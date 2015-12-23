
#![feature(io)]

use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::sync::{ Arc, Mutex, MutexGuard };
use std::sync::mpsc::{ channel, Sender, Receiver };
use std::thread;

// State Function
struct StateFn<E: std::error::Error>(fn(l: &mut Lex<E>)->Option<StateFn<E>>);

#[derive(Debug)]
struct Pos {
	pos: u64,
	line: u64,
	col: u64
}

// Position
impl Pos {
	fn new()->Pos {
		Pos {
			pos: 0,
			line: 0,
			col: 0
		}
	}

	fn next(&mut self, c: char) {
		self.pos += 1;
		match c {
			'\n' => {
				self.line += 1;
				self.col = 0;
			},
			_ => self.col += 1
		}
	}

	fn back(&mut self, c: char) {
		self.pos -= 1;
		match c {
			// Don't back over newlines.
			_ => self.col -= 1
		}
	}
}

#[derive(Debug)]
enum TokenType {
	Integer,
	Operator
}

#[derive(Debug)]
struct Token {
	pos: Pos,
	typ: TokenType,
	val: String
}

trait Lex<E: std::error::Error> {
	fn next(&mut self)->Result<char, E>;
	fn back(&mut self);
	fn emit(&mut self);
}

// State Machine
struct Lexer<R: io::Read+Send> {
	// Current position
	pos: Pos,
	// Input character stream
	input: Arc<Mutex<io::Chars<R>>>,
	// Back buffer
	backbuf: Vec<char>,
	// Unemitted buffer
	buf: Vec<char>,
	// State machine position
	state: StateFn<io::CharsError>,
	// Sending handle to token channel
	send: Option<Sender<Token>>,
	// Sending state machine thread handle
	handle: Option<thread::JoinHandle<()>>
}

fn start_state<E: std::error::Error>(l: &mut Lex<E>)->Option<StateFn<E>> {
	l.emit();
	None
}

impl<R: io::Read+Send+'static> Lexer<R> {
	fn new(reader: R)->Lexer<R> {
		let mut l = Lexer {
			pos: Pos::new(),
			input: Arc::new(Mutex::new(reader.chars())),
			buf: Vec::<char>::new(),
			backbuf: Vec::<char>::new(),
			state: StateFn(start_state),
			send: None,
			handle: None,
		};
		l
	}

	fn spawn(mut self)->Receiver<Token> {
		let (tx, rx) = channel();
		self.send = Some(tx);
		self.handle = Some(thread::spawn(move || {
			loop {
				match (self.state.0)(&mut self) {
					Some(s) => self.state = s,
					None => return
				}
			}
		}));
		rx
	}
}

impl<R: io::Read+Send> Lex<io::CharsError> for Lexer<R> {
	fn next(&mut self)->Result<char, io::CharsError> {
		let c = if self.backbuf.len() == 0 {
			'h'
			// self.input.lock().unwrap().next().unwrap()
		} else {
			self.backbuf.pop().unwrap()
		};
		self.buf.push(c); // Push to unemitted buffer
		self.pos.next(c); // Update position
		Ok(c)
	}

	fn back(&mut self) {
		self.buf.push(self.backbuf.pop().unwrap())
	}

	fn emit(&mut self) {
		self.send.as_ref().unwrap().send(Token{
			pos: Pos::new(),
			typ: TokenType::Integer,
			val: "Test".to_string()
		}).unwrap();
	}
}

fn main() {
	let lexer = Lexer::new(BufReader::new(File::open("src.f").unwrap()));
	let rx = lexer.spawn();
	println!("{:?}", rx.recv().unwrap());
}
