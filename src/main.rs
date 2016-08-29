/*
== run-info == (Matthias Drescher, 2016)

This program shows the current cpu and memory load of the machine within a terminal UI.
Since it depends on linux-specific system files, it will only run on a linux machine.
*/

extern crate term;
use term::color;
use term::Attr;

extern crate clap;
use clap::{Arg, App};

extern crate time;

use std::io::prelude::*;
use std::mem;
use std::thread;

mod reader;
use reader::MemInfo;
use reader::{CPUInfo, CPULoad};

fn main()
{
	let matches = App::new("run-info")
						.version("0.1")
						.about("Shows current CPU and memory load")
						.arg(Arg::with_name("delay")
							.short("d")
							.long("delay")
							.help("Sets the delay between updating the information (in ms)")
							.takes_value(true))
						.arg(Arg::with_name("no-color")
							.short("c")
							.long("no-color")
							.help("Switches to monochrome mode"))
						.arg(Arg::with_name("simple")
							.short("s")
							.long("simple")
							.help("One-line display for logging"))
						.get_matches();
	let delay_str = matches.value_of("delay").unwrap_or("1500").to_owned();
	let color = matches.occurrences_of("no-color") == 0;
	let simple = matches.occurrences_of("simple") != 0;
	let mut valid_delay = true;
	let delay = match delay_str.parse::<usize>()
	{
		Ok(v) => v,
		Err(_) => {println!("Error: delay argument is not a number"); valid_delay = false; 0}
	};
	if valid_delay
	{
		main_loop(delay, color, simple);
	}
}

#[allow(unused_assignments)]
fn main_loop(delay: usize, color: bool, simple: bool)
{
	println!("");
	let mut term = term::stdout().expect("term is not available");
	let mut meminfo = MemInfo::new();
	let mut cpuinfo_old = CPUInfo::new();
	let mut cpuinfo_new = CPUInfo::new();
	let mut cpuinfo_delta = CPUInfo::new(); //The delta between the two time frames
	let _ = cpuinfo_new.update();

	loop 
	{
		match meminfo.update()  //we can just update the meminfo
		{
			Ok(_) => {},
			Err(_) => {
				println!("Error: Memory information is not available."); 
				println!("Maybe you are not running this program on a Linux OS?");
				break;}
		};

		mem::swap(&mut cpuinfo_new, &mut cpuinfo_old); //the new info is becoming the old info, and a new info is requested
		cpuinfo_new = CPUInfo::new();
		match cpuinfo_new.update()
		{
			Ok(_) => {},
			Err(_) => {
				println!("Error: CPU information is not available.");
				println!("Maybe you are not running this program on a Linux OS?");
				break;
			}
		};
		cpuinfo_delta = CPUInfo::new(); //reset delta
		CPUInfo::calculate_delta(&mut cpuinfo_delta, &cpuinfo_old, &cpuinfo_new); //calculate the difference

		if simple
		{
			print_simple(&mut term, color, &cpuinfo_delta, &meminfo);
		}
		else 
		{
			print(&mut term, color, &cpuinfo_delta, &meminfo);
		}

		thread::sleep(std::time::Duration::from_millis(delay as u64)); //wait until next update
	}
}

//Printing

fn print_simple(mut term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool, cpu: &CPUInfo, mem: &MemInfo)
{
	let time = time::now();
	let timestamp = format!("{}m/{}d/{}y-{}h:{}m:{}s", time.tm_mon+1, time.tm_mday, time.tm_year+1900, time.tm_hour, time.tm_min, time.tm_sec);
	let cpuload_string = format_float(calc_cpu_load_percentage(&cpu.total_load));
	let mem_string = format_gib(mem.total - mem.free - mem.cached);
	let swap_string = format_gib(mem.swap_used);

	let _ = write!(term, "{}\t\tCPU:", timestamp);
	print_highlighted(term, color, format!("{} %", cpuload_string));
	let _ = write!(term, "\t\tRAM: ");
	print_highlighted(term, color, format!("{} Gib", mem_string));
	if mem.swap_used != 0
	{
		let _ = write!(term, "\t\tSWAP: ");
		print_highlighted(term, color, format!("{} Gib", swap_string));
	}

	let _ = writeln!(term, "");
}

fn print(mut term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool, cpu: &CPUInfo, mem: &MemInfo)
{
	let mut lines_printed = 13;

	//CPU

	//"x processes on x cores"
	print_header(term, color, 57, String::from("CPU"));
	print_highlighted(term, color, format!("{}", cpu.processes));
	if cpu.processes > 1
	{
		let _ = write!(term, " processes on ");	
	}else 
	{
	   	let _ = write!(term, " process on ");
	}
	print_highlighted(term, color, format!("{}", cpu.cores));
	let _ = writeln!(term, " cores      ");
	let _ = writeln!(term, "");

	//print bars
	print_highlighted(term, color, String::from("TOTAL: "));
	let total_percentage = calc_cpu_load_percentage(&cpu.total_load);
	print_progress_bar(term, color, total_percentage, 40, color::RED);
	print_highlighted(term, color, format!(" {} %   ", format_float(total_percentage)));
	let _ = writeln!(term, "");
	
	let mut core_counter = 1;
	for core_load in &cpu.cores_load
	{
		let _ = write!(term, "CPU {}: ", core_counter);
		let core_percentage = calc_cpu_load_percentage(&core_load);
		print_progress_bar(term, color, core_percentage, 40, color::GREEN);
		let _ = write!(term, " {} %   ", format_float(core_percentage));
		let _ = writeln!(term, "");
		lines_printed += 1;
		core_counter += 1;
	}
	let _ = writeln!(term, "");

	//MEMORY

	print_header(term, color, 57, String::from("MEMORY"));
	let _ = writeln!(term, "");

	let memory_use: f64 = (mem.total - mem.free - mem.cached) as f64 / mem.total as f64;
	let swap_use: f64 = (mem.swap_total - mem.swap_free) as f64 / mem.swap_total as f64;

	let _ = write!(term, "  RAM: "); //RAM BAR
	print_progress_bar(term, color, memory_use, 40, color::GREEN);
	let _ = writeln!(term, "");
	print_highlighted(term, color, format!("             {}", format_gib(mem.total - mem.free - mem.cached)));
	let _ = write!(term, " GiB / ");
	print_highlighted(term, color, format!("{}", format_gib(mem.total)));
	let _ = write!(term, " GiB (");
	print_highlighted(term, color, format!(" {}% ", format_float(memory_use)));
	let _ = write!(term, ")");
	let _ = writeln!(term, "\n");

	let _ = write!(term, " SWAP: "); //SWAP BAR
	print_progress_bar(term, color, swap_use, 40, color::GREEN);
	let _ = writeln!(term, "");
	print_highlighted(term, color, format!("               {}", format_gib(mem.swap_used)));
	let _ = write!(term, " GiB / ");
	print_highlighted(term, color, format!("{}", format_gib(mem.swap_total)));
	let _ = write!(term, " GiB (");
	print_highlighted(term, color, format!(" {}% ", format_float(swap_use)));
	let _ = write!(term, ")");
	let _ = writeln!(term, "\n");



	for _ in 0..lines_printed
	{
		let _ = term.cursor_up();
	}
}

//UI Objects

fn print_progress_bar(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, enabled: bool, value: f64, size: usize, color: u16)
{
	let barsize = ((value * size as f64)) as usize;
	let _ = write!(term, "[");
	colorize(term, enabled, color);
	attribute(term, enabled, Attr::Bold);
	for i in 0..size
	{
		if i < barsize
		{
			let _ = write!(term, "=");
		}
		else 
		{
		 	let _ = write!(term, " ");   
		}
	}
	reset(term, enabled);
	let _ = write!(term, "]");
}

fn print_highlighted(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool, content: String)
{
	colorize(term, color, color::CYAN);
	attribute(term, color, Attr::Bold);
	let _ = write!(term, "{}", content);
	reset(term, color);
}

fn print_header(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool, size: usize, name: String)
{
	let halfsize = (size - name.len() - 2) / 2;
	let extend_first = halfsize * 2 + name.len() + 2 != size; //catch rounding errors
	for _ in 0..halfsize
	{
		let _ = write!(term, "=");
	}
	if extend_first
	{
		let _ = write!(term, "=");
	}
	colorize(term, color, color::YELLOW);
	attribute(term, color, Attr::Bold);
	let _ = write!(term, " {} ", name);
	reset(term, color);
	for _ in 0..halfsize
	{
		let _ = write!(term, "=");
	}
	let _ = writeln!(term, "");
}	

//HELPER FUNCTIONS

fn attribute(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool, attrib: Attr)
{
	if color
	{
		let _ = term.attr(attrib);
	}
}

fn reset(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool)
{
	if color
	{
		let _ = term.reset();
	}
}

fn colorize(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool, color_code: u16)
{
	if color
	{
		let _ = term.fg(color_code);
	}
}

fn format_float(float: f64) -> String
{
	let mut string = format!("{}", float*100.0);
	if string.contains(".")
	{
		string = string.split_at(string.find(".").unwrap() + 2).0.to_owned();
	}
	string
}

//takes kilobytes, transforms to gibibytes and crops the result according to format_float()
fn format_gib(kib: u64) -> String
{
	let gib = ((kib as f64 / 1024.0) / 1024.0) / 1024.0;

	let mut string = format!("{}", gib);
	if string.contains(".")
	{
		string = string.split_at(string.find(".").unwrap() + 2).0.to_owned();
	}
	string
}

fn calc_cpu_load_percentage(load: &CPULoad) -> f64
{
	let mut load_percentage: f64 = 0.0;
	if load.busy != 0
	{
		load_percentage = load.busy as f64 / (load.idle + load.busy) as f64;
	}
	load_percentage
}