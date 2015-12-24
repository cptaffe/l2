#![feature(io)]
mod scanner;

use std::io;
use std::fs::File;
use std::error;

#[derive(Debug)]
enum SpaceTokenType {
	Token,
	Space
}

fn space_state<E: std::error::Error>(l: &mut scanner::Scanner<E, SpaceTokenType>)->Result<Option<scanner::StateFn<E, SpaceTokenType>>, E> {
	while match l.next() {
		Ok(c) => match c {
			' ' | '\n' | '\t' => true,
			_ => false
		},
		Err(e) => return Err(e)
	} {}
	l.back();
	l.emit(SpaceTokenType::Space);
	Ok(Some(scanner::StateFn(start_state)))
}

fn start_state<E: std::error::Error>(l: &mut scanner::Scanner<E, SpaceTokenType>)->Result<Option<scanner::StateFn<E, SpaceTokenType>>, E> {
	while match l.next() {
		Ok(c) => match c {
		' ' | '\n' | '\t' => false,
		_ => true
		},
		Err(e) => return Err(e)
	} {}
	l.back();
	l.emit(SpaceTokenType::Token);
	Ok(Some(scanner::StateFn(space_state)))
}

// State Machine
struct StateMachine<E: error::Error, T>(scanner::StateFn<E, T>);

// SpaceStateMachine remains generic,
// but this specialization allows us to select a startstate
// for SpaceTokenType.
impl<E: error::Error> StateMachine<E, SpaceTokenType> {
	fn new()->StateMachine<E, SpaceTokenType> {
		StateMachine(scanner::StateFn(start_state))
	}
}

impl<E: error::Error, T> scanner::StateMachine<E, T> for StateMachine<E, T> {
	fn start_state(self)->Result<Option<scanner::StateFn<E, T>>, E> {
		return Ok(Some(self.0));
	}
}

fn main() {
	let lexer = scanner::ReadScanner::new(io::BufReader::new(File::open("src.f").unwrap()), StateMachine::<_, SpaceTokenType>::new());
	let rx = lexer.spawn();
	println!("{:?}", match rx.recv().as_ref() {
		Ok(k) => k as &std::fmt::Debug,
		Err(e) => e as &std::fmt::Debug
	});
}
