extern crate libc;
extern crate core;
use libc::{c_int, c_void, size_t};
use std::ptr::null;
use core::mem::transmute;
use std::vec;
use std::comm;
use std::task;
use std::num;


// opaque struct
struct pa_simple;

struct pa_sample_spec {
	format: c_int,
	rate: u32,
	channels: u8
}

#[link (name="pulse")]
#[link (name="pulse-simple")]

extern "C" {
	fn pa_simple_new(
		server: *c_void,
		name: *i8,
		dir: c_int,
		dev: *c_void,
		stream_name: *i8,
		ss: *pa_sample_spec,
		pa_channel_map: *c_void,
		pa_buffer_attr: *c_void,
		error: *c_int
	) -> *pa_simple;
	fn pa_simple_read(s: *pa_simple, data: *mut c_void, bytes: size_t, error: *c_int) -> c_int;
	fn pa_simple_write(s: *pa_simple, data: *c_void, bytes: size_t, error: *c_int) -> c_int;
	fn pa_simple_flush(s: *pa_simple, error: *c_int) -> c_int;
	fn pa_simple_get_latency(s: *pa_simple, error: *c_int) -> u64;
}

pub fn pulseSource(cData: comm::Sender<Vec<f32>>, sRate: uint, bSize: uint) {
	
	let ss = pa_sample_spec { format: 3, rate: sRate as u32, channels: 1 };
	// pa_stream_direction_t -> enum, record = 2, playback = 1
	unsafe {
		let error: c_int = 0;
		let s: *pa_simple = pa_simple_new(null(), "rust-pa-simple-source".to_c_str().unwrap(), 2, null(), "pa-source".to_c_str().unwrap(), &ss, null(), null(), &error);
		assert_eq!(error, 0);
		'main : loop {
			let mut buffer: Vec<i16> = vec::Vec::from_elem(bSize, 0i16);
			pa_simple_read(s, transmute(buffer.as_mut_ptr()), (bSize*2) as u64, &error);
			assert_eq!(error, 0);
			let f32Buffer: Vec<f32> = buffer.iter().map(|&i| (i as f32)).collect();
			cData.send(f32Buffer);
		}
		}
}

pub fn pulseSink(pData: Receiver<Vec<f32>>, sRate: uint) {
	let ss = pa_sample_spec { format: 5, rate: sRate as u32, channels: 1 };
	let error: c_int = 0;
	unsafe {
		let s: *pa_simple = pa_simple_new(null(), "rust-pa-simple-sink".to_c_str().unwrap(), 1, null(), "pa-sink".to_c_str().unwrap(), &ss, null(), null(), &error);
		println!("{:?}", pa_simple_get_latency(s, &error));
		'main : loop {
			let samps = pData.recv();
			if (samps.length() == 0) { break 'main }
			let size: size_t = (samps.len() as u64)*4;
			pa_simple_write(s, transmute(samps.as_ptr()), size, &error);
		}
	}
}
