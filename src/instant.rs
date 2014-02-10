/* Copyright Ian Daniher, 2013, 2014.
   Distributed under the terms of CC BY-NC-SA 4.0. */

extern mod native;
extern mod kpn;
extern mod bitfount;

use kpn::{Symbol, SourceConf};
use native::task::spawn;


pub enum Parts{
	Head (fn (Chan<Symbol>, &SourceConf) -> () ),
	Body (fn (Port<Symbol>, Chan<Symbol>, &SourceConf) -> () ),
	Tail (fn (Port<Symbol>, &SourceConf) -> () ),
}

enum Either{
	P(Port<Symbol>),
	C(Chan<Symbol>)
}

pub fn spinUp(fs: ~[Parts], conf: SourceConf) {
	let mut ps: ~[Either] = ~[];
	for _ in range(0, fs.len()) {
		let (p, c) = Chan::new();
		ps.push(C(c));
		ps.push(P(p));
	}
	for &f in fs.iter() {
		match (f, ps.shift()) {
			(Head(source), Some(C(c))) => {
				do spawn { source(c, &conf) } ;
			},
			(Body(manip), Some(P(p))) => {
				match ps.shift() {
					Some(C(c)) => do spawn { manip(p, c, &conf) },
					_ => ()
				}
			}
			(Tail(sink), Some(P(p))) => {
				do spawn { sink(p, &conf) } ;
			}
			_ => {}
		}
	}
}
