#![feature(io)]
mod scanner;

use std::io::BufReader;
use std::fs::File;
use std::error;

fn start_state<E: error::Error>(l: &mut scanner::Scanner<E>)->Option<scanner::StateFn<E>> {
	l.emit();
	None
}

fn main() {
	let lexer = scanner::ReadScanner::new(BufReader::new(File::open("src.f").unwrap()), scanner::StateFn(start_state));
	let rx = lexer.spawn();
	println!("{:?}", rx.recv().unwrap());
}
