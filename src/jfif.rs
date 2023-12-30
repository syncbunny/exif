use std::io;
use std::io::prelude::*;
use std::fs::File;
use crate::exif::*;
use crate::logger;

pub struct JFIF {
	pub version: u16,
	pub unit: u8,
	pub dots_par_unit_x: u16,
	pub dots_par_unit_y: u16,
	pub thumnail_width: u8,
	pub thumnail_height: u8,
	pub exif: Option<EXIF>,
}

enum JFIFState {
	INIT,
	FF,
	JFIF,
	JFIF_FF,
	ACCEPTED,
	IOERR,
	ERR
}

impl JFIF {
	pub fn load(f: &mut File) -> Option<JFIF> {
		let mut jfif = JFIF {
			version: 0,
			unit: 0,
			dots_par_unit_x: 0,
			dots_par_unit_y: 0,
			thumnail_width: 0,
			thumnail_height: 0,
			exif: None,
		};

		let mut state: JFIFState = JFIFState::INIT;
		while !matches!(state, JFIFState::ERR | JFIFState::ACCEPTED) {
			let mut inbyte = [0;1];
			let ret = f.read_exact(&mut inbyte);
			if ret.is_err() {
				state = JFIFState::IOERR;
				break;
			}
			match state {
				JFIFState::INIT => {
					match inbyte[0] {
						0xFF => {
							state = JFIFState::FF;
						}
						_ => {
							state = JFIFState::ERR;
						}
					}
				}
				JFIFState::FF => {
					match inbyte[0] {
						0xD8 => {
							state = JFIFState::JFIF;
						}
						_ => {
							state = JFIFState::ERR;
						}
					}
				}
				JFIFState::JFIF => {
					match inbyte[0] {
						0xFF => {
							state = JFIFState::JFIF_FF;
						}
						_ => {}
					}
				}
				JFIFState::JFIF_FF => {
					match inbyte[0] {
						0x00 => {  // data [0xFF]
							state = JFIFState::JFIF;	
						}
						0xD0..=0xD7 => { // restart
							state = JFIFState::JFIF;	
						}
						0xD9 => { // EOI
							logger::debug("ACCEPTED!");
							state = JFIFState::ACCEPTED;
						}
						0xE0 => { // APP0
							state = jfif.APP0(f, JFIFState::JFIF);
						}
						0xE1 => {
							state = jfif.APP1(f, JFIFState::JFIF);
						}
						_ => {
							logger::debug(format!("mark FF{:02X}", inbyte[0]).as_str());
							if jfif.mark(f) {
								state = JFIFState::JFIF;
							} else {
								logger::debug("ERR");
								state = JFIFState::ERR;
							}
						}
					}
				}
				JFIFState::ACCEPTED => {}
				JFIFState::IOERR => {}
				JFIFState::ERR => {}
			}
		}

		match state {
			JFIFState::ACCEPTED => {
				Some(jfif)
			}
			_ => {
				None
			}
		}	
	}

	fn mark(&mut self, f: &mut File) -> bool {
		let mut inbyte = [0;1];
		let mut len:u16 = 0;

		// length
		if let Some(l) = read_length(f) {
			len = l;
		} else {
			return false;
		}
		logger::debug(format!("length: {}", len).as_str());

		// skip
		for i in 0..len-2 {
			let ret = f.read_exact(&mut inbyte);
			if ret.is_err() {
				return false;
			}
		}

		return true;
	}

	fn APP0(&mut self, f: &mut File, ok_state: JFIFState) -> JFIFState {
		logger::debug("APP0");

		// Length
		let mut len: u16 = 0;
		if let Some(l) = read_length(f) {
			len = l;
		} else {
			return JFIFState::ERR;
		}
		logger::debug(format!("length: {}", len).as_str());

		// "JFIF\0"
		let mut buff = [0; 5];
		let ret = f.read_exact(&mut buff);
		if ret.is_err() {
			return JFIFState::ERR;
		}
		if buff[0] != 0x4A { // 'J'
			return JFIFState::ERR;
		}
		if buff[1] != 0x46 { // 'F'
			return JFIFState::ERR;
		}
		if buff[2] != 0x49 { // 'I'
			return JFIFState::ERR;
		}
		if buff[3] != 0x46 { // 'F'
			return JFIFState::ERR;
		}
		if buff[4] != 0x00 { // '\0'
			return JFIFState::ERR;
		}

		// version
		if let Some(version) = read_2bytes(f) {
			self.version = version;
		} else {
			return JFIFState::ERR;
		}

		// unit
		if let Some(unit) = read_1byte(f) {
			self.unit = unit;
		} else {
			return JFIFState::ERR;
		}

		// Dots Par Unit X/Y
		if let Some(dpu) = read_2bytes(f) {
			self.dots_par_unit_x = dpu;
		} else {
			return JFIFState::ERR;
		}
		if let Some(dpu) = read_2bytes(f) {
			self.dots_par_unit_y = dpu;
		} else {
			return JFIFState::ERR;
		}

		// thumnail size
		if let Some(size) = read_1byte(f) {
			self.thumnail_width = size;
		} else {
			return JFIFState::ERR;
		}
		if let Some(size) = read_1byte(f) {
			self.thumnail_height = size;
		} else {
			return JFIFState::ERR;
		}
			
		// skip thumnail image	
		if !skip(f, len-16 ) {
			return JFIFState::ERR;
		}

		return ok_state;
	}

	fn APP1(&mut self, f: &mut File, ok_state: JFIFState) -> JFIFState {
		logger::debug("APP1");
		let mut len: u16 = 0;
		if let Some(l) = read_length(f) {
			len = l;
		} else {
			return JFIFState::ERR;
		}
		logger::debug(format!("length: {}", len).as_str());
		
		// EXIF check
		let mut buff = [0; 6];
		let ret = f.read_exact(&mut buff);
		if ret.is_err() {
			return JFIFState::ERR;
		}
		if buff[0] == 0x45 && buff[1] == 0x78 && buff[2] == 0x69 && buff[3] == 0x66 && buff[4] == 0x00 && buff[5] == 0x00 {
			logger::debug("EXIF");
			let mut buff:Vec<u8> = Vec::with_capacity(len as usize -8);
			unsafe {
				buff.set_len(len as usize -8);
			}
			let ret = f.read_exact(&mut buff);
			if ret.is_err() {
				return JFIFState::ERR;
			}
			self.exif = EXIF::load(&mut buff.as_slice());
		} else {
			logger::debug("not EXIF");
			if !skip(f, len-2-6) {
				return JFIFState::ERR;
			}
		}
	
		return ok_state;
	}
}



fn read_length(f: &mut File) -> Option<u16> {
	if let Some(len) = read_2bytes(f) {
		if len < 2 {
			return None;
		} else {
			return Some(len);
		}
	} else {
		return None;
	}
}

fn read_1byte(f: &mut File) -> Option<u8> {
	let mut inbyte = [0;1];
	let mut r:u8 = 0;

	let ret = f.read_exact(&mut inbyte);
	if ret.is_err() {
		None
	} else {
		r = inbyte[0];
		Some(r)
	}
}

fn read_2bytes(f: &mut File) -> Option<u16> {
	let mut inbyte = [0;2];
	let mut r:u16 = 0;

	let ret = f.read_exact(&mut inbyte);
	if ret.is_err() {
		None
	} else {
		r = inbyte[0] as u16;
		r <<= 8;
		r |= inbyte[1] as u16;
		Some(r)
	}
}

fn skip(f: &mut File, len: u16) -> bool {
	let mut buff = [0;1];

	for i in 0..len {	
		let ret = f.read_exact(&mut buff);
		if ret.is_err() {
			return false;
		}
	}
	return true;
}
