/*
Functions for printing output/layout
*/

use Settings;

use term::{
    color,
    self
};

use meminfo::MemInfo;
use cpuinfo::CPUInfo;
use graph::Graph;

use std::io::Stdout;

use printutils::*;

//so I don't have to write "let _ = write!(..)" all the time
macro_rules! p {
    ($t:expr, $($s:expr),+) => {
        let _ = write!($t, $($s),+);
    }
}

macro_rules! pl {
    ($t:expr, $($s:expr)+) => {
        let _ = writeln!($t, $($s,)+);
    }
}

//normal mode
pub fn print(mut term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings,
             cpu: &CPUInfo, mem: &MemInfo, graph: &mut Graph) {
	let mut lines_printed = 19;

	//CPU

	//"x processes on x cores"
	print_header(term, &settings, 57, String::from("CPU"));
	print_highlighted(term, &settings, format!("{}", cpu.processes));
	if cpu.processes > 1 {
		p!(term, " active processes on ");
	} else {
	   	p!(term, " active process on ");
	}
	print_highlighted(term, &settings, format!("{}", cpu.cores));
	pl!(term, " cores      ");
	pl!(term, "");

	//print bars
	print_highlighted(term, &settings, String::from("TOTAL: "));
	let total_percentage = calc_cpu_load_percentage(&cpu.total_load);
	graph.push(total_percentage); //push new data
	print_progress_bar(term, &settings, total_percentage, 40, color::RED);
	print_highlighted(term, &settings, format!(" {} %   ", format_float(total_percentage)));
	pl!(term, "");

	let mut core_counter = 1;
	for core_load in &cpu.cores_load {
		p!(term, "CPU {}: ", core_counter);
		let core_percentage = calc_cpu_load_percentage(&core_load);
		print_progress_bar(term, &settings, core_percentage, 40, color::GREEN);
		p!(term, " {} %   ", format_float(core_percentage));
		pl!(term, "");
		lines_printed += 1;
		core_counter += 1;
	}
	pl!(term, "");

	//print graph
	if settings.enable_graph {
    	print_graph(&mut term, &settings, graph);
	}
	else {
	    lines_printed -= 6;
	}

	//MEMORY

	print_header(term, &settings, 57, String::from("MEMORY"));
	pl!(term, "");

	let memory_use: f64 = mem.memory_use();
	let swap_use: f64 = mem.swap_use();

	p!(term, "  RAM: "); //RAM BAR
	print_progress_bar(term, &settings, memory_use, 40, color::GREEN);
	pl!(term, "");
	print_highlighted(term, &settings, format!("             {}",
         format_gib(mem.total - mem.free - mem.cached)));
	p!(term, " GiB / ");
	print_highlighted(term, &settings, format!("{}", format_gib(mem.total)));
	p!(term, " GiB (");
	print_highlighted(term, &settings, format!(" {}% ", format_float(memory_use)));
	p!(term, ")");
	pl!(term, "\n");

	p!(term, " SWAP: "); //SWAP BAR
	print_progress_bar(term, &settings, swap_use, 40, color::GREEN);
	pl!(term, "");
	print_highlighted(term, &settings, format!("               {}", format_gib(mem.swap_used)));
	p!(term, " GiB / ");
	print_highlighted(term, &settings, format!("{}", format_gib(mem.swap_total)));
	p!(term, " GiB (");
	print_highlighted(term, &settings, format!(" {}% ", format_float(swap_use)));
	p!(term, ")");
	pl!(term, "\n");

	for _ in 0..lines_printed {
		let _ = term.cursor_up();
	}
}

pub fn print_small_mode(mut term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings,
                        cpu: &CPUInfo, mem: &MemInfo) {
    let mut lines_printed = 4;
    //CPU
	print_highlighted(term, &settings, format!("TOTAL: "));
    let total_percentage = calc_cpu_load_percentage(&cpu.total_load);
    print_progress_bar(term, &settings, total_percentage, 40, color::RED);
    p!(term, " {} %   ", format_float(total_percentage));
    pl!(term, "");
    let mut core_counter = 1;
    for core_load in &cpu.cores_load {
        p!(term, "CPU {}: ", core_counter);
        let core_percentage = calc_cpu_load_percentage(&core_load);
        print_progress_bar(term, &settings, core_percentage, 40, color::GREEN);
        p!(term, " {} %   ", format_float(core_percentage));
        pl!(term, "");
        lines_printed += 1;
        core_counter += 1;
    }
    pl!(term, "");
    //MEM
	let memory_use: f64 = mem.memory_use();
	let swap_use: f64 = mem.swap_use();
    print_highlighted(term, &settings, format!("RAM:   "));
    print_progress_bar(term, &settings, memory_use, 40, color::YELLOW);
    p!(term, " {} %   ", format_float(memory_use));
    pl!(term, "");
    if swap_use > 0.0 {
        print_highlighted(term, &settings, format!("SWAP:  "));
        print_progress_bar(term, &settings, swap_use, 40, color::RED);
        p!(term, " {}   ", format_float(swap_use));
        pl!(term, "");
        lines_printed += 1;
    }
    pl!(term, "");
    
    for _ in 0..lines_printed {
        let _ = term.cursor_up();
    }
}

//a one-line version of print that can be used to log the data (-l flag)
pub fn print_log_mode(mut term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings,
                      cpu: &CPUInfo, mem: &MemInfo) {
	let seperator = "    ";
    
	let time = ::time::now();
	let timestamp = format!("{}m/{}d/{}y-{}h:{}m:{}s",
         time.tm_mon+1, time.tm_mday, time.tm_year+1900, time.tm_hour, time.tm_min, time.tm_sec);
	let cpuload_string = format_float(calc_cpu_load_percentage(&cpu.total_load));
	let mem_string = format_gib(mem.total - mem.free - mem.cached);
	let swap_string = format_gib(mem.swap_used);

	p!(term, "{}{}CPU:", timestamp, seperator);
	print_highlighted(term, &settings, format!("{}%", cpuload_string));
	p!(term, "{}RAM:", seperator);
	print_highlighted(term, &settings, format!("{}Gib", mem_string));
	if mem.swap_used != 0 {
		p!(term, "{}SWAP:", seperator);
		print_highlighted(term, &settings, format!("{}Gib", swap_string));
	}

	pl!(term, "");
}
