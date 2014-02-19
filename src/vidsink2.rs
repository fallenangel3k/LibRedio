extern crate sdl2;
extern crate extra;
extern crate dsputils;
extern crate native;
extern crate kpn;

use kpn::{Token, SourceConf, Packet, Dbl};
use extra::time;
use std::comm;
use native::task::spawn;

pub fn drawVectorAsBarPlot (renderer: &sdl2::render::Renderer, mut data: ~[f32]){
	// downsample to 800px if needbe
	let (sw, sh) = renderer.get_output_size().unwrap();
	let len: uint = data.len() as uint;
	let px: uint = sw as uint;
	let decimateFactor = (px as f32 - (0.5f32*(data.len() as f32/px as f32)) ) / data.len() as f32;
	data = data.iter().enumerate().filter_map(|(x, &y)|
		if ((x as f32*decimateFactor) - (x as f32*decimateFactor).floor()) < decimateFactor { Some(y) } else { None }
	 ).to_owned_vec();
	//data = data.iter().enumerate().filter(|&(x, &y)| (x % (len/px + 1)) == 0).map(|(x, &y)| y).collect();
	// black screen background
	renderer.set_draw_color(sdl2::pixels::RGB(0, 0, 0));
	renderer.clear();
	// calculate bar width
	let width: f32 = sw as f32 / (data.len() as f32);
	let height: f32 = sh as f32;
	// find max value
	let &dmax: &f32 = data.iter().max().unwrap();
	let &dmin: &f32 = data.iter().min().unwrap();
	// calculate height scale value
	let scale: f32 = height / (2f32*(dmax-dmin));
	assert!(width > 1.0);
	for i in range(0, data.len()) {
		let x = data[i];
		let mut yf = height*0.5f32;
		let mut hf = scale*x;
		if x > 0f32 {yf -= x*scale;}
		if x < 0f32 {hf = -1f32*hf;}
		let r = sdl2::rect::Rect (
			((sw as f32) - width*(i as f32 + 1.0)) as i32,
			yf as i32,
			width as i32,
			hf as i32);
		println!("{:?}", &r);
		renderer.set_draw_color(sdl2::pixels::RGB(0, 127, 0));
		renderer.fill_rect(&r);
	};
}

pub fn doWorkWithPEs (pDataC: comm::Port<~[f32]>) {
	let mut lastDraw: u64 = 0;
	sdl2::init([sdl2::InitVideo]);
	let window =  match sdl2::video::Window::new("rust-sdl2 demo: Video", sdl2::video::PosCentered, sdl2::video::PosCentered, 1300, 600, [sdl2::video::OpenGL]) {
		Ok(window) => window,
		Err(err) => fail!("")
	};
	let renderer =  match sdl2::render::Renderer::from_window(window, sdl2::render::DriverAuto, [sdl2::render::Accelerated]){
		Ok(renderer) => renderer,
		Err(err) => fail!("")
	};
	'main : loop {
		match sdl2::event::poll_event() {
			sdl2::event::QuitEvent(_) => break 'main,
			_ => {}
		}
		match pDataC.try_recv() {
			comm::Data(d) => {
				drawVectorAsBarPlot(renderer, d);
				renderer.present()
			}
			_ => ()
		}
	}
	sdl2::quit();
}

pub fn spawnVectorVisualSink() -> (comm::Chan<~[f32]>){
	let (pData, cData): (comm::Port<~[f32]>, comm::Chan<~[f32]>) = comm::Chan::new();
	spawn(proc(){ doWorkWithPEs(pData)});
	return cData;
}

pub fn vidSink(U: Port<Token>, S: SourceConf) {
	let c = spawnVectorVisualSink();
	let mut x: ~[f32] = ~[0.0f32, ..1000];
	loop {
		match U.recv() {
			Packet(p) => {x = p.move_iter().filter_map(|x| match x { Dbl(x) => Some(x as f32), _ => None }).to_owned_vec()},
			Dbl(d)  => {x.pop(); x.unshift(d as f32)},
			_ => (),
		}
		c.send(x.clone());
	}
}
