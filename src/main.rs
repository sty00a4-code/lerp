extern crate lerp_lib;

use lerp_lib::{compiler::compile_program, parser::parse};
use std::{env, fs, process};

fn main() {
    let mut args = env::args().skip(1);
    let Some(input_path) = args.next() else {
        eprintln!("no input file provided");
        process::exit(1);
    };
    let Some(output_path) = args.next() else {
        eprintln!("no output file provided");
        process::exit(1);
    };
    let Ok(code) = fs::read_to_string(&input_path) else {
        eprintln!("couldn't open file {input_path:?}");
        process::exit(1);
    };
    let program = parse(&code)
        .map_err(|err| {
            eprintln!("Parse Error {input_path}:{err}");
            process::exit(1);
        })
        .unwrap();
    let program = compile_program(program)
        .map_err(|err| {
            eprintln!("Compilation Error {input_path}:{err}");
            process::exit(1);
        })
        .unwrap();
    fs::write(&output_path, program.to_string())
        .map_err(|err| {
            eprintln!("couldn't write assembly to {output_path:?}: {err}");
            process::exit(1);
        })
        .unwrap();
}
