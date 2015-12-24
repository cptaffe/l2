use std::io;
use std::io::prelude::*;
use std::sync::{ Arc, Mutex };
use std::sync::mpsc::{ channel, Sender, Receiver };
use std::thread;
use std::error;

// State Function
pub struct StateFn<E: error::Error, T>(pub fn(l: &mut Scanner<E, T>)->Result<Option<StateFn<E, T>>, E>);

pub trait StateMachine<E: error::Error, T> {
	fn start_state(self)->Result<Option<StateFn<E, T>>, E>;
}

#[derive(Clone, Debug)]
pub struct Pos {
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
			'\n' => {},
			_ => self.col -= 1
		}
	}
}

#[derive(Debug)]
pub struct Token<T> {
	pos: Pos,
	typ: T,
	val: String
}

pub trait Scanner<E: error::Error, T> {
	fn next(&mut self)->Result<char, E>;
	fn back(&mut self);
	fn emit(&mut self, typ: T);
}

// State Machine
pub struct ReadScanner<R: io::Read+Send, T: Send> {
	// Current position
	pos: Pos,
	// Input character stream
	input: Arc<Mutex<io::Chars<R>>>,
	// Back buffer
	backbuf: Vec<char>,
	// Unemitted buffer
	buf: String,
	// State machine position
	state: StateFn<io::CharsError, T>,
	// Sending handle to token channel
	send: Option<Sender<Token<T>>>,
	// Sending state machine thread handle
	handle: Option<thread::JoinHandle<Option<io::CharsError>>>
}

impl<R: io::Read+Send+'static, T: Send+'static> ReadScanner<R, T> {
	pub fn new<S: StateMachine<io::CharsError, T>>(reader: R, sstate: S)->ReadScanner<R, T> {
		let l = ReadScanner {
			pos: Pos::new(),
			input: Arc::new(Mutex::new(reader.chars())),
			buf: String::new(),
			backbuf: Vec::<char>::new(),
			state: sstate.start_state().unwrap().unwrap(),
			send: None,
			handle: None,
		};
		l
	}

	pub fn spawn(mut self)->Receiver<Token<T>> {
		let (tx, rx) = channel();
		self.send = Some(tx);
		self.handle = Some(thread::spawn(move || {
			loop {
				match (self.state.0)(&mut self) {
					Ok(s) => match s {
						Some(s) => self.state = s,
						None => return None,
					},
					Err(e) => return Some(e)
				}
			}
		}));
		rx
	}
}

impl<R: io::Read+Send, T: Send> Scanner<io::CharsError, T> for ReadScanner<R, T> {
	fn next(&mut self)->Result<char, io::CharsError> {
		let c = if self.backbuf.len() == 0 {
			try!(self.input.lock().unwrap().next().unwrap())
		} else {
			self.backbuf.pop().unwrap()
		};
		self.buf.push(c); // Push to unemitted buffer
		self.pos.next(c); // Update position
		Ok(c)
	}

	fn back(&mut self) {
		let c = self.buf.pop().unwrap();
		self.backbuf.push(c);
		self.pos.back(c);
	}

	fn emit(&mut self, typ: T) {
		let t = self.send.as_ref().unwrap().send(Token{
			pos: self.pos.clone(),
			typ: typ,
			val: self.buf.clone()
		}).unwrap();
		self.buf = String::new();
		t
	}
}
