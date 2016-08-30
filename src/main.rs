/*
== run-info == (Matthias Drescher, 2016)

This program shows the current cpu and memory load of the machine within a terminal UI.
Since it depends on linux-specific system files, it will only run on a linux machine.
*/

extern crate term;

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
mod printer;

//Holds CLAP arguments
pub struct Settings
{
	delay: usize,
	enable_color: bool,
	enable_graph: bool,
	log_mode: bool
}

fn main()
{
	let matches = App::new("run-info")
						.version("0.3")
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
							.help("One-line display mode for logging"))
						.arg(Arg::with_name("no-graph")
							.short("g")
							.long("no-graph")
							.help("Hides the graph displayed under the CPU section in normal mode"))
						.get_matches();
	let delay_str = matches.value_of("delay").unwrap_or("1500").to_owned();
	let enable_color = matches.occurrences_of("no-color") == 0;
	let log_mode = matches.occurrences_of("log-mode") != 0;
	let enable_graph = matches.occurrences_of("no-graph") == 0;
	let mut valid_delay = true;
	let delay = match delay_str.parse::<usize>()
	{
		Ok(v) => v,
		Err(_) => {println!("Error: delay argument is not a number"); valid_delay = false; 0}
	};
	let settings = Settings{
		delay: delay,
		enable_color: enable_color,
		enable_graph: enable_graph,
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
			printer::print_log_mode(&mut term, &settings, &cpuinfo_delta, &meminfo);
		}
		else 
		{
			printer::print(&mut term, &settings, &cpuinfo_delta, &meminfo, &mut cpu_graph);
		}

		thread::sleep(std::time::Duration::from_millis(settings.delay as u64)); //wait until next update
	}
}