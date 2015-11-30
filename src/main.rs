
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{ channel, Sender, Receiver };
use std::thread;

// State Function
struct StateFn<E: std::error::Error> {
    f: fn(l: &Lex<E>)->Option<StateFn<E>>
}

#[derive(Debug)]
struct Pos {
    line: u64,
    col: u64
}

impl Pos {
    fn new()->Pos {
        Pos {
            line: 0,
            col: 0
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
    fn next(&self)->Result<char, E>;
}

// State Machine
struct Lexer<R: io::Read> {
    pos: Pos,
    buf: Arc<Mutex<io::Chars<R>>>,
    state: StateFn<io::CharsError>,
    send: Sender<Token>,
    recv: Arc<Mutex<Receiver<Token>>>
}

fn startState<E: std::error::Error>(l: &Lex<E>)->Option<StateFn<E>> {
    None
}

impl<R: io::Read> Lexer<R> {
    fn new(reader: R)->Lexer<R> {
        let (tx, rx) = channel();
        Lexer {
            pos: Pos::new(),
            buf: Arc::new(Mutex::new(reader.chars())),
            state: StateFn {
                f: startState
            },
            send: tx,
            recv: Arc::new(Mutex::new(rx))
        }
    }

    fn lex(&self) {
        thread::spawn(move || {
            loop {
                match (self.state.f)(self) {
                    Some(s) => { self.state = s; }
                    None => { return }
                }
            }
        });
    }
}

impl<R: io::Read> Lex<io::CharsError> for Lexer<R> {
    fn next(&self)->Result<char, io::CharsError> {
        try!(self.buf.lock().unwrap().next())
    }
}

fn main() {
    let mut l = Lexer::new(BufReader::new(try!(File::open("src.f"))));
    println!("{:?}\n", l.recv.lock().unwrap().recv().unwrap());

}
