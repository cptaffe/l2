
#![feature(io)]

use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::sync::{Arc, Mutex };
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
    fn emit(&self);
}

// State Machine
struct Lexer<R: io::Read+Send> {
    pos: Pos,
    buf: Arc<Mutex<io::Chars<R>>>,
    state: StateFn<io::CharsError>,
    send: Option<Sender<Token>>
}

fn start_state<E: std::error::Error>(l: &Lex<E>)->Option<StateFn<E>> {
    l.emit();
    None
}

impl<R: io::Read+Send+'static> Lexer<R> {
    fn new(reader: R)->Lexer<R> {
        Lexer {
            pos: Pos::new(),
            buf: Arc::new(Mutex::new(reader.chars())),
            state: StateFn {
                f: start_state
            },
            send: None
        }
    }

    fn lex(mut self)->(thread::JoinHandle<()>, Receiver<Token>) {
        let (tx, rx) = channel();
        self.send = Some(tx);
        (thread::spawn(move || {
            loop {
                match (self.state.f)(&self) {
                    Some(s) => { self.state = s; }
                    None => { return }
                }
            }
        }), rx)
    }
}

impl<R: io::Read+Send> Lex<io::CharsError> for Lexer<R> {
    fn next(&self)->Result<char, io::CharsError> {
        self.buf.lock().unwrap().next().unwrap()
    }

    fn emit(&self) {
        self.send.as_ref().unwrap().send(Token{
            pos: Pos::new(),
            typ: TokenType::Integer,
            val: "Test".to_string()
        }).unwrap();
    }
}

fn main() {
    let lexer = Lexer::new(BufReader::new(File::open("src.f").unwrap()));
    let (th, rx) = lexer.lex();
    println!("{:?}", rx.recv().unwrap());
    th.join().unwrap();
}
