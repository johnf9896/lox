#![feature(associated_type_defaults)]
#![feature(never_type)]
#![feature(exhaustive_patterns)]
#![allow(clippy::diverging_sub_expression)]

extern crate failure;
#[macro_use]
extern crate failure_derive;

mod array;
mod callable;
mod class;
mod constants;
pub mod error;
mod eval;
mod expr;
mod lexer;
mod location;
mod parser;
mod resolver;
mod scriptable;
mod stmt;
#[cfg(test)]
mod test_utils;
mod utils;
mod value;

use ansi_term::Color::{Blue, Cyan, Green, Purple, Yellow};
use ansi_term::Style;
use error::print_err;
use eval::Interpreter;
use failure::{Fallible, ResultExt};
use lexer::Scanner;
use parser::Parser;
use resolver::Resolver;
use rustyline::{config::Configurer, error::ReadlineError, Editor};
use std::ffi::OsStr;
use std::io::{stdin, Read};
use std::path::Path;
use stmt::{Stmt, StmtKind};
use value::Value;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Lox {
    inter: Interpreter,
}

impl Lox {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Lox {
            inter: Interpreter::new(),
        }
    }

    pub fn run(&mut self, input: &str) -> Fallible<()> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;

        let mut parser = Parser::new(&tokens);
        let stmts = parser.parse()?;

        let mut resolver = Resolver::new(&mut self.inter);
        resolver.resolve(&stmts)?;

        error::print_warns(&resolver.warnings);

        self.inter.interpret(&stmts)?;

        Ok(())
    }

    pub fn run_file(&mut self, path: &OsStr) -> Fallible<()> {
        let content = if path == "-" {
            let mut content = String::new();
            stdin()
                .lock()
                .read_to_string(&mut content)
                .context("Could not read from stdin")?;
            content
        } else {
            let path = Path::new(path);
            let context = format!("Could not read '{}'", path.display());
            std::fs::read_to_string(path).context(context)?
        };

        self.run(&content)
    }

    pub fn run_prompt(&mut self) -> Fallible<()> {
        let mut rl = Editor::<()>::new();
        rl.set_auto_add_history(true);

        println!("Lox {}", VERSION);
        println!("Press Ctrl+D to exit\n");

        let prompt = format!("{}> ", Blue.bold().paint("lox"));

        loop {
            match rl.readline(&prompt) {
                Ok(line) if line.is_empty() => (),
                Ok(line) => match self.run_prompt_line(&line) {
                    Ok(()) => (),
                    Err(err) => print_err(&err),
                },
                Err(ReadlineError::Interrupted) => (),
                Err(ReadlineError::Eof) => break,
                Err(err) => return Err(err.into()),
            }
        }

        Ok(())
    }

    fn run_prompt_line(&mut self, input: &str) -> Fallible<()> {
        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens()?;

        let mut parser = Parser::new(&tokens);
        parser.allow_expression = true;
        let stmts = parser.parse()?;

        let mut resolver = Resolver::new(&mut self.inter);
        resolver.resolve(&stmts)?;

        error::print_warns(&resolver.warnings);

        if stmts.len() == 1 {
            match stmts.first().unwrap() {
                Stmt {
                    kind: StmtKind::Expression(expr),
                    ..
                } => {
                    let val = self.inter.evaluate(expr)?;
                    print!("=> ");
                    print_value(&val);
                    println!();
                }
                _ => {
                    self.inter.interpret(&stmts)?;
                }
            }
        } else {
            self.inter.interpret(&stmts)?;
        }

        Ok(())
    }
}

fn print_value(val: &Value) {
    let output = match val {
        Value::Integer(int) => Blue.paint(int.to_string()),
        Value::Float(float) => Cyan.paint(float.to_string()),
        Value::Str(string) => Green.paint(format!("\"{}\"", utils::escape_string(&string))),
        Value::Boolean(boolean) => Purple.paint(boolean.to_string()),
        Value::Nil => Purple.paint("nil"),
        Value::Callable(callable) => Yellow.paint(callable.to_string()),
        Value::Instance(instance) => Yellow.paint(instance.borrow().to_string()),
        Value::Array(array) => {
            let len = array.borrow().len();
            print!("[");
            for (i, el) in array.borrow().iter().enumerate() {
                print_value(el);
                if i < len - 1 {
                    print!(", ");
                }
            }

            Style::new().paint("]")
        }
    };

    print!("{}", output);
}
