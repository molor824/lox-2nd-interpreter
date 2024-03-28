use std::{env, process::exit};

use parser::error::*;
use parser::*;

mod parser;
mod string_name;

fn main() {
    let mut args = env::args();
    let _ = args.next();

    let mut file: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut build = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "build" | "b" => build = true,
            "-o" => {
                out_path = match args.next() {
                    Some(path) => Some(path),
                    None => {
                        println!("No output path specified");
                        exit(-1);
                    }
                }
            }
            _ => {
                file = Some(arg);
                break;
            }
        }
    }

    let source = if let Some(path) = file {
        if let Ok(s) = std::fs::read_to_string(&path) {
            s
        } else {
            println!("Failed to read file '{}'", path);
            exit(-1);
        }
    } else {
        println!("No input file specified");
        exit(-1);
    };

    let mut parser = Parser::new(&source);
    let nodes = parser.parse();
    match nodes {
        Ok(nodes) => {
            for node in nodes {
                println!("{:?}", node);
            }
        }
        Err(err) => {
            let mut message = String::new();
            err.display(&source, &mut message).unwrap();
            eprintln!("{}", message);
        }
    }
}
