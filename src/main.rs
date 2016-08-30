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

use std::mem;
use std::thread;

mod meminfo;
use meminfo::MemInfo;

mod cpuinfo;
use cpuinfo::CPUInfo;

mod printutils;
use printutils::print_progress_bar as print_progress_bar;
use printutils::print_highlighted as print_highlighted;
use printutils::print_header as print_header;
use printutils::attribute as attribute;
use printutils::reset as reset;
use printutils::colorize as colorize;
use printutils::format_float as format_float;
use printutils::format_gib as format_gib;
use printutils::calc_cpu_load_percentage as calc_cpu_load_percentage;
use printutils::transform_to_graphsize as transform_to_graphsize;

//Holds CLAP arguments
pub struct Settings
{
	delay: usize,
	enable_color: bool,
	log_mode: bool
}

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
						.arg(Arg::with_name("log-mode")
							.short("l")
							.long("log")
							.help("One-line display for logging"))
						.get_matches();
	let delay_str = matches.value_of("delay").unwrap_or("1500").to_owned();
	let enable_color = matches.occurrences_of("no-color") == 0;
	let log_mode = matches.occurrences_of("log-mode") != 0;
	let mut valid_delay = true;
	let delay = match delay_str.parse::<usize>()
	{
		Ok(v) => v,
		Err(_) => {println!("Error: delay argument is not a number"); valid_delay = false; 0}
	};
	let settings = Settings{
		delay: delay,
		enable_color: enable_color,
		log_mode: log_mode
	};
	if valid_delay
	{
		main_loop(settings);
	}
}

#[allow(unused_assignments)]
fn main_loop(settings: Settings)
{
	println!("");
	let mut term = term::stdout().expect("term is not available");
	let mut meminfo = MemInfo::new();
	let mut cpuinfo_old = CPUInfo::new();
	let mut cpuinfo_new = CPUInfo::new();
	let mut cpuinfo_delta = CPUInfo::new(); //The delta between the two time frames
	let _ = cpuinfo_new.update();
	
	let mut cpu_graph: Vec<f64> = Vec::new();
	for _ in 0..51
	{
		cpu_graph.push(0.0);
	}

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

		if settings.log_mode
		{
			print_log_mode(&mut term, &settings, &cpuinfo_delta, &meminfo);
		}
		else 
		{
			print(&mut term, &settings, &cpuinfo_delta, &meminfo, &mut cpu_graph);
		}

		thread::sleep(std::time::Duration::from_millis(settings.delay as u64)); //wait until next update
	}
}

//Printing

//a simpler version of print (-s flag)
fn print_log_mode(mut term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, settings: &Settings, cpu: &CPUInfo, mem: &MemInfo)
{
	let time = time::now();
	let timestamp = format!("{}m/{}d/{}y-{}h:{}m:{}s", time.tm_mon+1, time.tm_mday, time.tm_year+1900, time.tm_hour, time.tm_min, time.tm_sec);
	let cpuload_string = format_float(calc_cpu_load_percentage(&cpu.total_load));
	let mem_string = format_gib(mem.total - mem.free - mem.cached);
	let swap_string = format_gib(mem.swap_used);

	let _ = write!(term, "{}\t\tCPU:", timestamp);
	print_highlighted(term, &settings, format!("{} %", cpuload_string));
	let _ = write!(term, "\t\tRAM: ");
	print_highlighted(term, &settings, format!("{} Gib", mem_string));
	if mem.swap_used != 0
	{
		let _ = write!(term, "\t\tSWAP: ");
		print_highlighted(term, &settings, format!("{} Gib", swap_string));
	}

	let _ = writeln!(term, "");
}

fn print(mut term: &mut Box<term::Terminal<Output=std::io::Stdout> + Send>, settings: &Settings, cpu: &CPUInfo, mem: &MemInfo, graph: &mut Vec<f64>)
{
	let mut lines_printed = 19;

	//CPU

	//"x processes on x cores"
	print_header(term, &settings, 57, String::from("CPU"));
	print_highlighted(term, &settings, format!("{}", cpu.processes));
	if cpu.processes > 1
	{
		let _ = write!(term, " active processes on ");	
	}else 
	{
	   	let _ = write!(term, " active process on ");
	}
	print_highlighted(term, &settings, format!("{}", cpu.cores));
	let _ = writeln!(term, " cores      ");
	let _ = writeln!(term, "");

	//print bars
	print_highlighted(term, &settings, String::from("TOTAL: "));
	let total_percentage = calc_cpu_load_percentage(&cpu.total_load);
	graph.push(total_percentage); //push new data
	graph.remove(0); //dequeue old data
	print_progress_bar(term, &settings, total_percentage, 40, color::RED);
	print_highlighted(term, &settings, format!(" {} %   ", format_float(total_percentage)));
	let _ = writeln!(term, "");
	
	let mut core_counter = 1;
	for core_load in &cpu.cores_load
	{
		let _ = write!(term, "CPU {}: ", core_counter);
		let core_percentage = calc_cpu_load_percentage(&core_load);
		print_progress_bar(term, &settings, core_percentage, 40, color::GREEN);
		let _ = write!(term, " {} %   ", format_float(core_percentage));
		let _ = writeln!(term, "");
		lines_printed += 1;
		core_counter += 1;
	}
	let _ = writeln!(term, "");

	//print graph
	let graph_sizes = transform_to_graphsize(&graph);
	for y in (0..5).rev()
	{
		let mut label = format!("{}%", y*25);
		while label.len() < 5
		{
			label.push(' ');
		}
		label.push('|');
		let _ = write!(term, "{}", label);
		colorize(term, &settings, color::CYAN);
		attribute(term, &settings, Attr::Bold);
		for x in 0..51
		{
			let size = graph_sizes.get(x).unwrap();
			if size < &(y*2)
			{
				let _ = write!(term, " ");
			}
			else if size < &(y*2 +1)
			{
				let _ = write!(term, ".");
			}
			else 
			{
			    let _ = write!(term, ":");
			}
		}
		reset(term, &settings);
		let _ = writeln!(term, "");
	}
	let _ = writeln!(term, "");

	//MEMORY

	print_header(term, &settings, 57, String::from("MEMORY"));
	let _ = writeln!(term, "");

	let memory_use: f64 = (mem.total - mem.free - mem.cached) as f64 / mem.total as f64;
	let swap_use: f64 = (mem.swap_total - mem.swap_free) as f64 / mem.swap_total as f64;

	let _ = write!(term, "  RAM: "); //RAM BAR
	print_progress_bar(term, &settings, memory_use, 40, color::GREEN);
	let _ = writeln!(term, "");
	print_highlighted(term, &settings, format!("             {}", format_gib(mem.total - mem.free - mem.cached)));
	let _ = write!(term, " GiB / ");
	print_highlighted(term, &settings, format!("{}", format_gib(mem.total)));
	let _ = write!(term, " GiB (");
	print_highlighted(term, &settings, format!(" {}% ", format_float(memory_use)));
	let _ = write!(term, ")");
	let _ = writeln!(term, "\n");

	let _ = write!(term, " SWAP: "); //SWAP BAR
	print_progress_bar(term, &settings, swap_use, 40, color::GREEN);
	let _ = writeln!(term, "");
	print_highlighted(term, &settings, format!("               {}", format_gib(mem.swap_used)));
	let _ = write!(term, " GiB / ");
	print_highlighted(term, &settings, format!("{}", format_gib(mem.swap_total)));
	let _ = write!(term, " GiB (");
	print_highlighted(term, &settings, format!(" {}% ", format_float(swap_use)));
	let _ = write!(term, ")");
	let _ = writeln!(term, "\n");


	for _ in 0..lines_printed
	{
		let _ = term.cursor_up();
	}
}