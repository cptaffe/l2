use std::io;
use std::io::prelude::*;
use std::sync::{ Arc, Mutex };
use std::sync::mpsc::{ channel, Sender, Receiver };
use std::thread;
use std::error;

// State Function
pub struct StateFn<E: error::Error>(pub fn(l: &mut Scanner<E>)->Option<StateFn<E>>);

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

pub trait Scanner<E: error::Error> {
	fn next(&mut self)->Result<char, E>;
	fn back(&mut self);
	fn emit(&mut self);
}

// State Machine
pub struct ReadScanner<R: io::Read+Send> {
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

impl<R: io::Read+Send+'static> ReadScanner<R> {
	pub fn new(reader: R, sstate: StateFn<io::CharsError>)->ReadScanner<R> {
		let l = ReadScanner {
			pos: Pos::new(),
			input: Arc::new(Mutex::new(reader.chars())),
			buf: Vec::<char>::new(),
			backbuf: Vec::<char>::new(),
			state: sstate,
			send: None,
			handle: None,
		};
		l
	}

	pub fn spawn(mut self)->Receiver<Token> {
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

impl<R: io::Read+Send> Scanner<io::CharsError> for ReadScanner<R> {
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
