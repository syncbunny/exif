mod jfif;
mod exif;
mod logger;

use std::env;
use std::fs::File;
use crate::jfif::*;
//use crate::logger;

fn main() {
	let args:Vec<String> = env::args().collect();
	let mut cnt:i32 = 0;
	for arg in args {
		if cnt > 0 {
			let f = match File::open(&arg) {
				Err(e) => {
					println!("cant open file [{}]: {}", arg, e);
				}
				Ok(mut f) => {
					if let Some(jfif) = JFIF::load(&mut f) {
/*
						println!("version: {:04X}", jfif.version);
						println!("unit: {}", jfif.unit);
						println!("DPUX: {}", jfif.dots_par_unit_x);
						println!("DPUY: {}", jfif.dots_par_unit_y);
						println!("Thumnail Width: {}", jfif.thumnail_width);
						println!("Thumnail Height: {}", jfif.thumnail_height);
*/
						if let Some(exif) = jfif.exif {
							for (key, value) in exif.values {
								println!("{}: {}", key, value);
							}
						}
					}
				}
			};
		}
		cnt += 1;
	}
}

