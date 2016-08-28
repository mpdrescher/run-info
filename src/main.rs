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

use std::io::prelude::*;
use std::mem;
use std::thread;
use std::time;

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
						.get_matches();
	let delay_str = matches.value_of("delay").unwrap_or("1500").to_owned();
	let color = matches.occurrences_of("no-color") == 0;
	let mut valid_delay = true;
	let delay = match delay_str.parse::<usize>()
	{
		Ok(v) => v,
		Err(_) => {println!("Error: delay argument is not a number"); valid_delay = false; 0}
	};
	if valid_delay
	{
		main_loop(delay, color);
	}
}

fn main_loop(delay: usize, color: bool)
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

		print(&mut term, color, &cpuinfo_delta, &meminfo);

		thread::sleep(time::Duration::from_millis(delay as u64)); //wait until next update
	}
}

fn print(mut term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool, cpu: &CPUInfo, mem: &MemInfo)
{
	let mut lines_printed = 13;


	//======================================
	//CPU
	//======================================


	let _ = write!(term, "========================== ");
	colorize(&mut term, color, color::YELLOW);
	attribute(&mut term, color, Attr::Bold);
	let _ = write!(term, "CPU");
	reset(&mut term, color);
	let _ = writeln!(term, " ==========================");

	colorize(&mut term, color, color::CYAN);
	attribute(&mut term, color, Attr::Bold);
	let _ = write!(term, "{}", cpu.processes);
	reset(&mut term, color);
	if cpu.processes > 1
	{
		let _ = write!(term, " processes on ");	
	}else 
	{
	   	let _ = write!(term, " process on ");
	}
	colorize(&mut term, color, color::CYAN);
	attribute(&mut term, color, Attr::Bold);
	let _ = write!(term, "{}", cpu.cores);
	reset(&mut term, color);
	let _ = writeln!(term, " cores      ");
	let _ = writeln!(term, "");

	print_cpu_load(&mut term, color, &cpu.total_load, String::from("TOTAL"), true);
	let mut core_counter = 1;
	for core_load in &cpu.cores_load
	{
		print_cpu_load(term, color, core_load, format!("CPU {}", core_counter), false);
		core_counter += 1;
		lines_printed += 1;
	}
	let _ = writeln!(term, "");


	//======================================
	//MEMORY
	//======================================


	let _ = write!(term, "========================= ");
	colorize(&mut term, color, color::YELLOW);
	attribute(&mut term, color, Attr::Bold);
	let _ = write!(term, "MEMORY");
	reset(&mut term, color);
	let _ = writeln!(term, " ========================");
	let _ = writeln!(term, "");

	let memory_use: f64 = (mem.total - mem.free - mem.cached) as f64 / mem.total as f64;
	let memory_bar_size = (memory_use * 40.0) as usize;
	let swap_use: f64 = (mem.swap_total - mem.swap_free) as f64 / mem.swap_total as f64;
	let swap_bar_size = (swap_use * 40.0) as usize;

	let _ = write!(term, "  RAM: ["); //RAM BAR
	attribute(&mut term, color, Attr::Bold);
	colorize(&mut term, color, color::GREEN);
	for i in 0..40
	{
		if i < memory_bar_size
		{
			let _ = write!(term, "=");
		}
		else 
		{
		    let _ = write!(term, " ");
		}
	}
	reset(&mut term, color);
	let _ = writeln!(term, "]");

	colorize(&mut term, color, color::CYAN);
	attribute(&mut term, color, Attr::Bold);
	let _ =  write!(term, "             {}", format_gib(mem.total - mem.free - mem.cached));
	reset(&mut term, color);
	let _ = write!(term, " GiB / ");
	colorize(&mut term, color, color::CYAN);
	attribute(&mut term, color, Attr::Bold);
	let _ =  write!(term, "{}", format_gib(mem.total));
	reset(&mut term, color);
	let _ = write!(term, " GiB (");
	colorize(&mut term, color, color::CYAN);
	attribute(&mut term, color, Attr::Bold);
	let _ = write!(term, " {}% ", format_float(memory_use));
	reset(&mut term, color);
	let _ = write!(term, ")");
	let _ = writeln!(term, "\n");

	let _ = write!(term, " SWAP: ["); //SWAP BAR
	attribute(&mut term, color, Attr::Bold);
	colorize(&mut term, color, color::GREEN);
	for i in 0..40
	{
		if i < swap_bar_size
		{
			let _ = write!(term, "=");
		}
		else 
		{
		    let _ = write!(term, " ");
		}
	}
	reset(&mut term, color);
	let _ = writeln!(term, "]");

	colorize(&mut term, color, color::CYAN);
	attribute(&mut term, color, Attr::Bold);
	let _ =  write!(term, "               {}", format_gib(mem.swap_used));
	reset(&mut term, color);
	let _ = write!(term, " GiB / ");
	colorize(&mut term, color, color::CYAN);
	attribute(&mut term, color, Attr::Bold);
	let _ =  write!(term, "{}", format_gib(mem.swap_total));
	reset(&mut term, color);
	let _ = write!(term, " GiB (");
	colorize(&mut term, color, color::CYAN);
	attribute(&mut term, color, Attr::Bold);
	let _ = write!(term, " {}% ", format_float(swap_use));
	reset(&mut term, color);
	let _ = write!(term, ")");
	let _ = writeln!(term, "\n");

	for _ in 0..lines_printed
	{
		let _ = term.cursor_up();
	}
}

fn print_cpu_load(mut term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, color: bool, load: &CPULoad, name: String, total: bool)
{
	if total
	{
		colorize(&mut term, color, color::CYAN);
		attribute(&mut term, color, Attr::Bold);
	}
	let _ = write!(term, "{}:", name);
	if total
	{
		reset(&mut term, color);
	}
	let _ = write!(term, " [");
	
	let mut load_percentage: f64 = 0.0;
	if load.busy != 0
	{
		load_percentage = load.busy as f64 / (load.idle + load.busy) as f64;
	}
	let barsize = (load_percentage * 40.0) as usize;

	if !total
	{
		colorize(&mut term, color, color::GREEN);
	}
	else 
	{
	 	colorize(&mut term, color, color::RED);
	}
	attribute(&mut term, color, Attr::Bold);

	for i in 0..40
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

	reset(&mut term, color);
	let _ = write!(term, "] ");
	reset(&mut term, color);
	if total
	{
		attribute(&mut term, color, Attr::Bold);
		colorize(&mut term, color, color::CYAN);
	}
	let _ = writeln!(term, "{} %   ", format_float(load_percentage));
	reset(&mut term, color);
}

fn attribute(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, enabled: bool, attrib: Attr)
{
	if enabled
	{
		let _ = term.attr(attrib);
	}
}

fn reset(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, enabled: bool)
{
	if enabled
	{
		let _ = term.reset();
	}
}

fn colorize(term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, enabled: bool, color: u16)
{
	if enabled
	{
		let _ = term.fg(color);
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