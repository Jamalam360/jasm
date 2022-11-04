use std::{fs::File, io::Write, time::Instant};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

mod compiler;
mod instructions;
mod interpreter;
mod lexer;
use crate::{
    compiler::compile,
    instructions::Value,
    interpreter::interpret,
    lexer::{lex_bin, lex_str},
};

#[derive(Parser)]
#[command(name = "JASM")]
#[command(author = "Jamalam")]
#[command(version = "1.0.0")]
#[command(about = "A CLI for the JASM language, supporting compiling JASM, and running both JASM and JASMB files.", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Run { path: String },
    Compile { path: String },
}

pub const ADDRESS_ADDRESS: usize = 65534;
pub const COMPARISON_ADDRESS: usize = 65533;
pub type Memory = [u16; 65535];

fn main() -> Result<()> {
    let args = Args::parse();

    println!("--- JASM");

    let res = match args.command {
        Some(Commands::Compile { path }) => {
            if path.ends_with(".jasmb") {
                return Err(anyhow!("Cannot compile .jasmb"));
            }

            println!("--- Compiling file");
            let before_lex = Instant::now();
            let res = lex_str(&path)?;
            let buf = compile(res)?;
            println!(
                "--- Compiled in {}ms ({} microseconds)",
                before_lex.elapsed().as_millis(),
                before_lex.elapsed().as_micros(),
            );

            println!("--- Writing to File");
            let mut file = File::create(&path.replace(".jasm", ".jasmb"))?;
            file.write(&buf)?;
            println!("--- Compiled Successfully");
            Ok(())
        }
        Some(Commands::Run { path }) => {
            println!("--- Lexing file");
            let before_lex = Instant::now();

            let mut res = if path.ends_with(".jasm") {
                lex_str(&path)
            } else if path.ends_with(".jasmb") {
                lex_bin(&path)
            } else {
                Err(anyhow!("Invalid file extension"))
            }?;

            println!(
                "--- Lexed {} Instructions in {}ms ({} microseconds)",
                res.len(),
                before_lex.elapsed().as_millis(),
                before_lex.elapsed().as_micros(),
            );

            println!("--- Interpreting Instructions");
            println!("--- Program Output Begins Here");

            let before_interpret = Instant::now();
            let memory: &mut Memory = &mut [0u16; 65535];
            let exit_code = interpret(memory, &mut res)?;

            println!("");
            println!("Program exited with code {}", exit_code);
            println!(
                "--- Interpretation finished in {}ms ({} microseconds)",
                before_interpret.elapsed().as_millis(),
                before_interpret.elapsed().as_micros(),
            );

            Ok(())
        }
        None => Err(anyhow!("No subcommand specified")),
    };

    res
}
