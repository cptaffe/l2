#![feature(io)]
mod scanner;

use std::io;
use std::fs::File;

fn start_state<E: std::error::Error>(l: &mut scanner::Scanner<E>)->Option<scanner::StateFn<E>> {
	while match l.next().unwrap() {
		' ' => false,
		'\n' => false,
		_ => true
	} {}
	l.emit();
	None
}

fn main() {
	let lexer = scanner::ReadScanner::new(io::BufReader::new(File::open("src.f").unwrap()), scanner::StateFn(start_state));
	let rx = lexer.spawn();
	println!("{:?}", rx.recv().unwrap());
}
