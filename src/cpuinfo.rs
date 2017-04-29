/*
Read /proc/stat to get the cpu usage
Calculate difference between two datasets to get momentary load
*/

use std::fs::File;
use std::io::Read;
use std::io::Result;

pub struct CPULoad {
	pub busy: u64,
	pub idle: u64
}

pub struct CPUInfo {
	pub cores: usize,
	pub cores_load: Vec<CPULoad>,
	pub total_load: CPULoad,
	pub processes: usize
}

impl CPUInfo {
	pub fn new() -> CPUInfo {
		CPUInfo {
			cores: 0,
			cores_load: Vec::new(),
			total_load: CPULoad{busy: 0, idle: 0},
			processes: 0
		}
	}

	//parse stats in /proc/stat
	pub fn update(&mut self) -> Result<()> {
		let plain = try!(read_file("/proc/stat"));

		//filling out cores_load and processes
		for line in plain.lines() {
            //we only want core information for now
			if line.starts_with("cpu") && line.starts_with("cpu ") == false {
				//pushing all information within the line to a stack and getting the values by position
				let mut info_vec = Vec::new();
				for info in line.split_whitespace() {
					info_vec.push(info);
				}
				let mut busy: u64 = 0;
				let mut idle: u64 = 0;

				busy += info_vec.get(1).expect("missing cpu information")
                    .parse::<u64>().expect("incorrect cpu information format");//user time
				busy += info_vec.get(2).expect("missing cpu information")
                    .parse::<u64>().expect("incorrect cpu information format");//niced time
				busy += info_vec.get(3).expect("missing cpu information")
                    .parse::<u64>().expect("incorrect cpu information format");//system time
				idle += info_vec.get(4).expect("missing cpu information")
                    .parse::<u64>().expect("incorrect cpu information format");//idle time

				self.cores_load.push(CPULoad {
					busy: busy,
					idle: idle
				});
			}
			else if line.starts_with("procs_running") {
				//parsing the no. of processes (2nd entry in the "processes" line)
			    self.processes = line.split_whitespace().nth(1)
			    	.expect("incorrect cpu information format").to_owned()
                    .parse::<usize>().expect("expected a number (processes)");
			}
		}

		//getting the number of cores from the length of the coreinfo list
		self.cores = self.cores_load.len();

		//sum the core information for total
		let mut total_busy = 0;
		let mut total_idle = 0;
		for core in &self.cores_load {
			total_busy += core.busy;
			total_idle += core.idle;
		}
		self.total_load = CPULoad {
			busy: total_busy,
			idle: total_idle
		};

		Ok(())
	}

	//calculate the difference between the last two datasets on the cpu usage
	//this needs to be done since the file /proc/stat only holds the difference to boot time
	pub fn calculate_delta(delta: &mut CPUInfo, old: &CPUInfo, new: &CPUInfo) {
		delta.cores = new.cores; //core number and processes stay the same
		delta.processes = new.processes;

		delta.total_load.busy = new.total_load.busy - old.total_load.busy;
		delta.total_load.idle = new.total_load.idle - old.total_load.idle;

		for core in 0..delta.cores {
			let delta_idle = new.cores_load.get(core).unwrap().idle - old.cores_load.get(core).unwrap().idle;
			let delta_busy = new.cores_load.get(core).unwrap().busy - old.cores_load.get(core).unwrap().busy;
			delta.cores_load.push(CPULoad {
				idle: delta_idle,
				busy: delta_busy
			})
		}
	}
}

fn read_file(location: &str) -> Result<String> {
	let mut file = try!(File::open(location));
	let mut ret_val = String::new();
	try!(file.read_to_string(&mut ret_val));
	Ok(ret_val)
}
