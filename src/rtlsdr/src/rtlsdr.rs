/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of the GPLv3. */
#![feature(libc)]
#![feature(collections)]
#![feature(core)]

extern crate num;
extern crate libc;

use num::complex;
use libc::{c_int, c_uint};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::vec;
use std::str;
use std::ffi;
use std::string;
use std::thread;
use std::sync::Arc;
use std::intrinsics;

#[link(name= "rtlsdr")]

#[repr(C)]
struct RtlSdrInternal;

#[derive(Clone)]
pub struct RtlSdrDev {
    bx: Arc<RtlSdrInternal>
}

extern "C" {
	fn rtlsdr_open(dev: &RtlSdrInternal, dev_index: u32) -> u32;
	fn rtlsdr_get_device_count() -> u32;
	fn rtlsdr_get_device_name(dev_index: u32) -> *const i8;
	fn rtlsdr_reset_buffer(dev: &RtlSdrInternal) -> c_int;
	fn rtlsdr_set_center_freq(dev: &RtlSdrInternal, freq: u32) -> c_int;
	fn rtlsdr_set_tuner_gain(dev: &RtlSdrInternal, gain: u32) -> c_int;
	fn rtlsdr_set_tuner_gain_mode(dev: &RtlSdrInternal, mode: u32) -> c_int;
	fn rtlsdr_read_sync(dev: &RtlSdrInternal, buf: *mut u8, len: u32, n_read: *mut c_int) -> c_int;
	fn rtlsdr_read_async(dev: &RtlSdrInternal, cb: extern "C" fn(*const u8, u32, &Sender<Vec<u8>>), chan: &Sender<Vec<u8>>, buf_num: u32, buf_len: u32) -> c_int;
	fn rtlsdr_cancel_async(dev: &RtlSdrInternal) -> c_int;
	fn rtlsdr_set_sample_rate(dev: &RtlSdrInternal, sps: u32) -> c_int;
	fn rtlsdr_get_sample_rate(dev: &RtlSdrInternal) -> u32;
	fn rtlsdr_close(dev: &RtlSdrInternal) -> c_int;
}


impl Drop for RtlSdrDev {
    fn drop (&mut self) {
        self.close();
    }
}

impl RtlSdrDev {
	pub fn close(&self) {
		unsafe {
			let success = rtlsdr_close(&(*self.bx));
			assert_eq!(success, 0);
		}
	}
	pub fn set_sample_rate(&self, sps: u32) {
		unsafe {
			let success = rtlsdr_set_sample_rate(&(*self.bx), sps);
			assert_eq!(success, 0);
			println!("actual sample rate: {}", rtlsdr_get_sample_rate(&(*self.bx)));
		}
	}
	pub fn open() -> RtlSdrDev {
		unsafe {
			let mut i: u32 = 0;
            let internal: RtlSdrInternal = intrinsics::init();
			'tryDevices: loop {
				let success = rtlsdr_open(&internal, i);
				if success == 0 {
					break 'tryDevices
				}
				if i > get_device_count() {
					panic!("no available devices");
				}
				i += 1;
			}
       return RtlSdrDev {bx: Arc::new(internal)};
	   }
	}
	pub fn clear_buffer(&self) {
		unsafe {
			let success = rtlsdr_reset_buffer(&(*self.bx));
			assert_eq!(success, 0);
		}
	}
	pub fn set_frequency(&self, freq: u32) {
		unsafe {
			let success = rtlsdr_set_center_freq(&(*self.bx), freq);
			assert_eq!(success, 0);
		}
	}
	pub fn set_gain(&self, v: u32) {
		unsafe {
			let success = rtlsdr_set_tuner_gain_mode(&(*self.bx), 1);
			assert_eq!(success, 0);
			let success = rtlsdr_set_tuner_gain(&(*self.bx), v);
			assert_eq!(success, 0);
		}
	}
	pub fn set_gain_auto(&self) {
		unsafe {
			let success = rtlsdr_set_tuner_gain_mode(&(*self.bx), 0);
			assert_eq!(success, 0);
		}
	}
	pub fn read_async(&self, block_size: u32) -> Receiver<Vec<u8>> {
		let (chan, port) = channel();
        let bx = self.bx.clone();
		thread::spawn(move || {
			unsafe{
				rtlsdr_read_async(&(*bx), rtlsdr_callback, &chan, 32, block_size*2);
			}
		});
		return port;
	}
	pub fn stop_async(&self) {
		unsafe {
			let success = rtlsdr_cancel_async(&(*self.bx));
			assert_eq!(success, 0);
		}
	}
	pub fn read_sync(&self, ct: c_uint) -> Vec<u8> {
		unsafe {
			let mut n_read: c_int = 0;
			let mut buffer = vec::Vec::with_capacity(512);
			let success = rtlsdr_read_sync(&(*self.bx), buffer.as_mut_ptr(), ct, &mut n_read);
			assert_eq!(success, 0);
			assert_eq!(ct as i32, n_read);
			return buffer;
		}
	}
}

extern fn rtlsdr_callback(buf: *const u8, len: u32, chan: &Sender<Vec<u8>>) {
	unsafe {
		let data = vec::Vec::from_raw_buf(buf, len as usize);
		chan.send(data).unwrap();
	}
}

pub fn get_device_count() -> u32 {
	unsafe {
		let x: u32 = rtlsdr_get_device_count();
		return x;
	}
}

pub fn get_device_name(dev_index: u32) -> string::String {
	unsafe {
		let device_string: *const i8 = rtlsdr_get_device_name(dev_index);
		return string::String::from_str(str::from_utf8(ffi::CStr::from_ptr(device_string).to_bytes()).unwrap());
	}
}
fn i2f(i: u8) -> f32 {i as f32/127.0 - 1.0}
pub fn data_to_samples(data: Vec<u8>) -> Vec<complex::Complex<f32>> {
	data[0..].chunks(2).map(|i| complex::Complex{re:i2f(i[0]), im:i2f(i[1])}).collect()
}
