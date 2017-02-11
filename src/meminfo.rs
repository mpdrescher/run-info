use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Result;

/*
Reads key/value pairs from /proc/meminfo
*/

pub struct MemInfo
{
	pub total: u64,
	pub free: u64,
	pub cached: u64,
	pub used: u64,

	pub swap_total: u64,
	pub swap_free: u64,
	pub swap_used: u64
}

impl MemInfo
{
	pub fn new() -> MemInfo
	{
		MemInfo {
			total: 0,
			free: 0,
			cached: 0,
			used: 0,

			swap_total: 0,
			swap_free: 0,
			swap_used: 0
		}
	}

	pub fn update(&mut self) -> Result<()>
	{
		let map = try!(MemInfo::read_mem());

		self.total = map.get("MemTotal:").expect("MemTotal not found").to_owned();
		self.free = map.get("MemFree:").expect("MemFree not found").to_owned();
		self.cached = map.get("Cached:").expect("Cached not found").to_owned();
		self.used = self.total - (self.free + self.cached);

		self.swap_total = map.get("SwapTotal:").expect("SwapTotal not found").to_owned();
		self.swap_free = map.get("SwapFree:").expect("SwapFree not found").to_owned();
		self.swap_used = self.swap_total - self.swap_free;

		Ok(())
	}

	//parse stats in /proc/meminfo into a HashMap
	fn read_mem() -> Result<HashMap<String, u64>>
	{
		let plain = try!(read_file("/proc/meminfo"));
		let mut mem_map = HashMap::new();

		for line in plain.lines()
		{
			let mut name = String::new();
			let mut value: u64 = 0;

			let mut col_count = 0;
			for info in line.split_whitespace()
			{
				match col_count
				{
					0 => {name = info.to_owned();},
					1 => {value = info.to_owned().parse::<u64>().expect("invalid value");},
					2 => {value *= 1024},
					_ => {}
				}
				col_count += 1;
			}

			mem_map.insert(name, value);
		}

		Ok(mem_map)
	}
}

fn read_file(location: &str) -> Result<String>
{
	let mut file = try!(File::open(location));
	let mut ret_val = String::new();
	try!(file.read_to_string(&mut ret_val));
	Ok(ret_val)
}