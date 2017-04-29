/*
== run-info == (Matthias Drescher, 2016/2017)

This program shows the current cpu and memory load of the machine within a terminal UI.
Since it depends on linux-specific system files, it will only run on a linux machine.
*/

extern crate term;

extern crate clap;
use clap::{
    Arg, App
};

extern crate time;

use std::mem;
use std::thread;

mod graph;
mod meminfo;
mod cpuinfo;
mod printutils;
mod printer;

use cpuinfo::CPUInfo;
use meminfo::MemInfo;
use graph::Graph;

//Holds CLAP arguments
pub struct Settings {
	delay: usize,
	enable_color: bool,
	enable_graph: bool,
	mode: Mode
}

pub enum Mode {
    Normal,
    Log,
    Small
}

fn main() {
	let matches = App::new("run-info")
						.version("0.5.1")
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
							.help("One-line display mode for logging")
                            .conflicts_with("short-mode"))
                        .arg(Arg::with_name("small-mode")
                            .short("s")
                            .long("small")
                            .help("Like the default printing style, but for smaller windows")
                            .conflicts_with("log-mode"))
						.arg(Arg::with_name("no-graph")
							.short("g")
							.long("no-graph")
							.help("Hides the graph displayed under the CPU section in normal mode"))
						.get_matches();
	let delay_str = matches.value_of("delay").unwrap_or("1500").to_owned();
	let enable_color = matches.occurrences_of("no-color") == 0;
    let mut mode = Mode::Normal;
    if  matches.occurrences_of("log-mode") > 0 {
        mode = Mode::Log;
    }
    if matches.occurrences_of("small-mode") > 0 {
        mode = Mode::Small;
    }
	let enable_graph = matches.occurrences_of("no-graph") == 0;
	let delay = match delay_str.parse::<usize>() {
		Ok(v) => v,
		Err(_) => {
            println!("error: delay argument is not a valid number.");
            return;
        }
	};
	let settings = Settings {
		delay: delay,
		enable_color: enable_color,
		enable_graph: enable_graph,
	    mode: mode
	};
	main_loop(settings);
}

#[allow(unused_assignments)]
fn main_loop(settings: Settings) {
	println!("");
	let mut term = term::stdout().expect("term is not available.");
	let mut meminfo = MemInfo::new();
	let mut cpuinfo_old = CPUInfo::new();
	let mut cpuinfo_new = CPUInfo::new();
	let mut cpuinfo_delta = CPUInfo::new(); //The delta between the two time frames
	let _ = cpuinfo_new.update();

	let mut cpu_graph = Graph::new();

	loop {
		match meminfo.update() {  //we can just update the meminfo
			Ok(_) => {},
			Err(_) => {
				println!("error: Memory information is not available.");
				println!("maybe you are not running this program on a Linux OS?");
				break;}
		};

		//the new info is becoming the old info, and a new info is requested
        mem::swap(&mut cpuinfo_new, &mut cpuinfo_old);
        cpuinfo_new = CPUInfo::new();
		match cpuinfo_new.update() {
			Ok(_) => {},
			Err(_) => {
				println!("error: CPU information is not available.");
				println!("maybe you are not running this program on a Linux OS?");
				break;
			}
		};
		cpuinfo_delta = CPUInfo::new(); //reset delta
		CPUInfo::calculate_delta(&mut cpuinfo_delta, &cpuinfo_old, &cpuinfo_new); //calculate the difference

		match settings.mode {
            Mode::Normal => {
                printer::print(&mut term, &settings, &cpuinfo_delta, &meminfo, &mut cpu_graph)
            },
            Mode::Log => {
                printer::print_log_mode(&mut term, &settings, &cpuinfo_delta, &meminfo)
            },
		    Mode::Small => {
                printer::print_small_mode(&mut term, &settings, &cpuinfo_delta, &meminfo)
            }
		}

		thread::sleep(std::time::Duration::from_millis(settings.delay as u64)); //wait until next update
	}
}
