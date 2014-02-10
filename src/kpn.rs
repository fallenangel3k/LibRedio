/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-NC-SA 4.0. */
extern mod extra;
extern mod dsputils;

use std::comm::{Chan, Port};

pub struct SourceConf {
	Freq: f64,
	Rate: f64,
	Period: f64
}

#[deriving(Eq, Clone, DeepClone)]
pub enum Symbol {
	Chit(uint),
	Break(~str),
	Dur(uint, f64),
	Run(uint, uint),
}


// run length encoding
pub fn rle(U: Port<Symbol>, V: Chan<Symbol>, S: &SourceConf){
	let mut x: Symbol = U.recv();
	let mut i: uint = 1;
	loop {
		let y = U.recv();
		if y != x {
			match x {
				Chit(v) => V.send( Run (v,i)),
				_ => ()
			}
			i = 1;
		}
		else {i = i + 1}
		x = y;
	}
}

// accept input infinite sequence of runs, convert counts to duration by dividing by sample rate
pub fn dle(U: Port<Symbol>, V: Chan<Symbol>, S: &SourceConf) {
	loop {
		match U.recv() {
			Run(v, ct) => V.send( Dur ( v, ct as f64 / S.Rate) ),
			_ => (),
		}
	}
}

// duration length decoding
pub fn dld(U: Port<Symbol>, V: Chan<Symbol>, S: &SourceConf) {
	loop {
		match U.recv() {
			Dur(v, dur) => V.send( Run ( v, (dur * S.Rate) as uint)),
			_ => (),
		}
	}
}

// run length decoding
pub fn rld(U: Port<Symbol>, V: Chan<uint>, S: &SourceConf) {
	loop {
		match U.recv() {
			Run(v, ct) => for _ in range(0, ct){V.send(v)},
			_ => (),
		}
	}
}


// temperature sensor pulse duration modulated binary protocol symbol matcher
pub fn validSymbolTemp(U: Port<Symbol>, V: Chan<Symbol>, S: &SourceConf) {
	loop {
		match U.recv() {
			Dur(va, dura) => {
				if (va == 1) && (4.4e-4 < dura) && (dura < 6e-4) {
					match U.recv() {
						Dur(_, durb) => {
							if (1.7e-3 < durb) && (durb < 2.2e-3) {V.send(Chit(0))}
							else if (3.6e-3 < durb) && (durb < 4.2e-3) {V.send(Chit(1))}
							else if durb > 8.7e-3 {V.send(Break(~"silence"))}
						},
						_ => ()
					}
				}
			}
			_=> ()
		}
	}
}

// manchester 1/2 pulse duration to state matching
pub fn validSymbolManchester(U: Port<Symbol>, V: Chan<Symbol>, S: &SourceConf) {
	loop {
		match U.recv() {
			Dur(v, dur) => {
				if ((0.7*S.Period) < dur) && ( dur < (1.3*S.Period)) { V.send(Chit(v))}
				else if (1.7*S.Period < dur) && (dur < (2.3*S.Period)) { V.send(Chit(v)); V.send(Chit(v));}
				else if (dur > 3.0*S.Period) && (v == 0) { V.send(Break(~"silence"))}
			},
			_ => ()
		}
	}
}


// manchester state-pair to symbol decoding
pub fn manchesterd(U: Port<Symbol>, V: Chan<Symbol>, S: &SourceConf) {
	let mut x = U.recv();
	let mut y = U.recv();
	loop {
		let msg = match (x, y.clone()) {
			(Chit(1),Chit(0)) => Chit(1),
			(Chit(0),Chit(1)) => Chit(0),
			(Chit(a), Chit(b)) if a == b => Break(~"manchester collision"),
			(Break(b), _) =>  Break(b),
			(_, Break(b)) =>  Break(b),
			_ => Break(~"else")
		};
		if msg == Break(~"manchester collision") {
			x = y;
			y = U.recv();
		}
		else {
			x = U.recv();
			y = U.recv();
		}
		V.send(msg);
	}
}

#[deriving(Eq)]
enum state {
	matching,
	matched
}

// basic convolutional detector, accepts an infinite sequence, passes all symbols after a match until a 1,0 symbol
pub fn detector(U: Port<Symbol>, V: Chan<Symbol>, W: ~[uint]) {
	// surprisingly useless unless implemented in hardware
	let mut m: ~[uint] = range(0,W.len()).map(|_| 0).to_owned_vec();
	let mut state = matching;
	loop {
		match U.recv() {
			Chit(x) if state == matching => {m.push(x);m.shift();},
			Chit(x) if state == matched => {V.send(Chit(x));m.push(x);m.shift();},
			Break(x) => {state = matching; V.send(Break(x)); m = range(0,W.len()).map(|_| 0).to_owned_vec();},
			_ => (),
		}
		if m == W {
			state = matched;
			let x = Break(~"preamble match");
			println!("{:?}", x);
			V.send(x);
			m = range(0,W.len()).map(|_| 0).to_owned_vec();
		}
	}
}

// duplicates infinite sequences
pub fn tuplicator(U: Port<Symbol>, V: Chan<Symbol>, W: Chan<Symbol>){
	loop {
		let y = U.recv();
		V.send(y.clone());
		W.send(y);
	}
}