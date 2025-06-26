/*
A collection of functions used for printing the data
*/

use term::{
    color,
    Attr,
    self
};

use Settings;

use std::io::Stdout;

use cpuinfo::CPULoad;
use graph::Graph;

//UI Objects

//print a progress bar ( -> [======      ] ),
//where value ranges from 0.0 to 1.0 (panic otherwise)
pub fn print_progress_bar(term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings,
                          value: f64, size: usize, color: u32) {
    assert!(value >= 0.0 && value <= 1.0);
	let barsize = ((value * size as f64)) as usize;
	let _ = write!(term, "[");
	colorize(term, settings, color);
	attribute(term, settings, Attr::Bold);
	for i in 0..size {
		if i < barsize {
			let _ = write!(term, "=");
		}
		else {
		 	let _ = write!(term, " ");
		}
	}
	reset(term, settings);
	let _ = write!(term, "]");
}

pub fn print_highlighted(term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings,
                         content: String) {
	colorize(term, settings, color::CYAN);
	attribute(term, settings, Attr::Bold);
	let _ = write!(term, "{}", content);
	reset(term, settings);
}

pub fn print_header(term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings,
                    size: usize, name: String) {
	let halfsize = (size - name.len() - 2) / 2;
	let extend_first = halfsize * 2 + name.len() + 2 != size; //catch rounding errors
	for _ in 0..halfsize {
		let _ = write!(term, "=");
	}
	if extend_first {
		let _ = write!(term, "=");
	}
	colorize(term, settings, color::YELLOW);
	attribute(term, settings, Attr::Bold);
	let _ = write!(term, " {} ", name);
	reset(term, settings);
	for _ in 0..halfsize {
		let _ = write!(term, "=");
	}
	let _ = writeln!(term, "");
}

pub fn print_graph(term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings, graph: &Graph) {
    let graph_height_values = graph.height_values(10);
    for y in (0..5).rev() {
        let mut label = format!("{}%", y*25);
        while label.len() < 5 {
            label.push(' ');
        }
        label.push('|');
        let _ = write!(term, "{}", label);
        colorize(term, &settings, color::CYAN);
        attribute(term, &settings, Attr::Bold);
        for x in 0..graph_height_values.len() {
            let size = graph_height_values[x];
            if size < y*2 {
                let _ = write!(term, " ");
            }
            else if size < y*2 +1 {
                let _ = write!(term, ".");
            }
            else {
                let _ = write!(term, ":");
            }
        }
        reset(term, &settings);
        let _ = writeln!(term, "");
    }
    let _ = writeln!(term, "");
}

//HELPER FUNCTIONS

pub fn attribute(term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings, attrib: Attr) {
	if settings.enable_color {
		let _ = term.attr(attrib);
	}
}

pub fn reset(term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings) {
	if settings.enable_color {
		let _ = term.reset();
	}
}

pub fn colorize(term: &mut Box<term::Terminal<Output=Stdout> + Send>, settings: &Settings, color_code: u32) {
	if settings.enable_color {
		let _ = term.fg(color_code);
	}
}

pub fn format_float(float: f64) -> String {
	let mut string = format!("{}", float*100.0);
	if string.contains(".") {
		string = string.split_at(string.find(".").unwrap() + 2).0.to_owned();
	}
	string
}

pub fn pad_string(mut str: String, size: usize) -> String {
    while str.len() < size {
        str.push(' ');
    }
    str
}

//takes kilobytes, transforms to gibibytes and crops the result according to format_float()
pub fn format_gib(kib: u64) -> String {
	let gib = ((kib as f64 / 1024.0) / 1024.0) / 1024.0;

	let mut string = format!("{}", gib);
	if string.contains(".") {
		string = string.split_at(string.find(".").unwrap() + 2).0.to_owned();
	}
	string
}

pub fn calc_cpu_load_percentage(load: &CPULoad) -> f64 {
	let mut load_percentage: f64 = 0.0;
	if load.busy != 0 {
		load_percentage = load.busy as f64 / (load.idle + load.busy) as f64;
	}
	load_percentage
}
