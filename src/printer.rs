/*
Functions for printing output/layout
*/

use Settings;

use term::{
    color,
    Attr,
    self
};

use meminfo::MemInfo;
use cpuinfo::CPUInfo;

use std::io::Stdout;

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
use printutils::pad_string as pad_string;

//so I don't have to write pl!(); all the time
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
             cpu: &CPUInfo, mem: &MemInfo, graph: &mut Vec<f64>) {
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
	graph.remove(0); //dequeue old data
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
		let graph_sizes = transform_to_graphsize(&graph);
		for y in (0..5).rev() {
			let mut label = format!("{}%", y*25);
			while label.len() < 5 {
				label.push(' ');
			}
			label.push('|');
			p!(term, "{}", label);
			colorize(term, &settings, color::CYAN);
			attribute(term, &settings, Attr::Bold);
			for x in 0..51 {
				let size = graph_sizes.get(x).unwrap();
				if size < &(y*2) {
					p!(term, " ");
				}
				else if size < &(y*2 +1) {
					p!(term, ".");
				}
				else {
			    	p!(term, ":");
				}
			}
			reset(term, &settings);
			pl!(term, "");
		}
		pl!(term, "");
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
    let mut lines_printed = 9;

    print_header(term, &settings, 28, "CPU:".to_owned());

    //CPU

    pl!(term, "");
   	let cpuload_string = format_float(calc_cpu_load_percentage(&cpu.total_load));
    print_highlighted(term, &settings, format!("{}%  \n", cpuload_string));
    let mut core_counter = 1;
    for core_load in &cpu.cores_load {
		let core_percentage = calc_cpu_load_percentage(&core_load);
		p!(term, "{}", &pad_string(format!("{}% ", format_float(core_percentage)), 7));
        if core_counter == 4 {
            pl!(term, "");
            lines_printed += 1;
        }
        core_counter += 1;
	}
    pl!(term, "");

    //Memory

    print_header(term, &settings, 28, "RAM:".to_owned());
    pl!(term, "");

	let memory_use: f64 = mem.memory_use();
	let swap_use: f64 = mem.swap_use();

    print_highlighted(term, &settings, format_gib(mem.total - mem.free - mem.cached));
    p!(term, " GiB ( ");
    print_highlighted(term, &settings, format!("{}%", format_float(memory_use)));
    p!(term, " ) + \n");
    print_highlighted(term, &settings, format_gib(mem.swap_used));
    p!(term, " GiB Swap ( ");
    print_highlighted(term, &settings, format!("{}%", format_float(swap_use)));
    p!(term, " )");
    pl!(term, "");

    pl!(term, "");
    for _ in 0..lines_printed {
        let _ = term.cursor_up();
    }
}

//a simpler version of print (-l flag)
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
