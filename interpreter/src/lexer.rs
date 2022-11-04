use crate::Value;
use anyhow::{anyhow, Result};
use std::{
    fs::File,
    io::{self, BufRead, Read},
    path::Path,
};

use crate::instructions::Instruction;

pub fn lex_str(path: &String) -> Result<Vec<Instruction>> {
    let mut instructions: Vec<Instruction> = Vec::new();
    let lines = read_lines(path)?;
    for line in lines {
        let mut found_comment = false;
        let line: String = line?
            .trim_start()
            .chars()
            .filter(|c| {
                if c == &';' {
                    found_comment = true;
                }

                !found_comment
            })
            .collect();

        if line.trim().is_empty() {
            continue;
        }

        let mut tokens: Vec<&str> = line.split_whitespace().collect();
        let instruction: &str;
        let mut value: Option<Value> = None;

        if tokens.len() == 0 {
            return Err(anyhow!("No tokens found"));
        }

        instruction = tokens[0];

        if tokens.len() > 1 {
            let is_address = tokens[1].starts_with("*");
            tokens[1] = tokens[1].trim_start_matches('*');

            if tokens[1].starts_with("0x") {
                let hex = u16::from_str_radix(tokens[1].trim_start_matches("0x"), 16)?;
                value = Some(if is_address {
                    Value::Address(hex)
                } else {
                    Value::Literal(hex)
                });
            } else if tokens[1].starts_with("0b") {
                let bin = u16::from_str_radix(tokens[1].trim_start_matches("0b"), 2)?;
                value = Some(if is_address {
                    Value::Address(bin)
                } else {
                    Value::Literal(bin)
                });
            } else {
                return Err(anyhow!("Unknown number literal {}", tokens[1]));
            }
        }

        instructions.push(Instruction::from_string(instruction, value)?);
    }

    Ok(instructions)
}

pub fn lex_bin(path: &String) -> Result<Vec<Instruction>> {
    let mut instructions: Vec<Instruction> = Vec::new();
    let file = File::open(path)?;
    let mut reader = io::BufReader::new(file);

    let mut buffer = [0; 2];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        let mut value: Option<u16> = None;
        let mut instruction_buf = buffer;
        if Instruction::u8_requires_value(buffer[1]) {
            instruction_buf = buffer.clone();
            let n = reader.read(&mut buffer)?;
            if n != 2 {
                return Err(anyhow!("Unexpected end of file"));
            }

            value = Some(u16::from_be_bytes(buffer));
        }
        let instruction = Instruction::from_u8(instruction_buf, value)?;

        instructions.push(instruction);
    }

    Ok(instructions)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
