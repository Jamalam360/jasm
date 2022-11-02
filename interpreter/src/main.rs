use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, Read, Write},
    num::Wrapping,
    path::Path,
    time::Instant,
};

use anyhow::{anyhow, Result};
use clap::Parser;

#[derive(Debug)]
enum Instruction {
    Output,
    CharacterOutput,
    CharacterInput,
    Dump,
    Return,
    SetAddress(Value),
    SetValue(Value),
    Add(Value),
    Subtract(Value),
    Multiply(Value),
    Divide(Value),
    Label(Value),
    Compare(Value),
    BranchIfNotEqual(Value),
    BranchIfEqual(Value),
    Jump(Value),
}

#[derive(Debug)]
enum Value {
    Literal(u16),
    Address(u16),
}

#[derive(Default, Parser)]
#[clap(about, version)]
pub struct Args {
    #[clap()]
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("--- JASM Interpreter");
    println!("--- Lexing Source");

    let before_lex = Instant::now();
    let mut num_lines = 0;
    let mut instructions: Vec<Instruction> = Vec::new();
    let lines = read_lines(args.path)?;
    for line in lines {
        num_lines += 1;
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

        match instruction {
            "OUT" => instructions.push(Instruction::Output),
            "CUT" => instructions.push(Instruction::CharacterOutput),
            "CIN" => instructions.push(Instruction::CharacterInput),
            "DMP" => instructions.push(Instruction::Dump),
            "RTN" => instructions.push(Instruction::Return),
            "SEA" => instructions.push(Instruction::SetAddress(
                value.expect("SEA instruction requires value"),
            )),
            "SET" => instructions.push(Instruction::SetValue(
                value.expect("SET instruction requires value"),
            )),
            "ADD" => instructions.push(Instruction::Add(
                value.expect("ADD instruction requires value"),
            )),
            "SUB" => instructions.push(Instruction::Subtract(
                value.expect("SUB instruction requires value"),
            )),
            "MUL" => instructions.push(Instruction::Multiply(
                value.expect("MUL instruction requires value"),
            )),
            "DIV" => instructions.push(Instruction::Divide(
                value.expect("DIV instruction requires value"),
            )),
            "LAB" => instructions.push(Instruction::Label(
                value.expect("LAB instruction requires value"),
            )),
            "CEQ" => instructions.push(Instruction::Compare(
                value.expect("CEQ instruction requires value"),
            )),
            "BNE" => instructions.push(Instruction::BranchIfNotEqual(
                value.expect("BNE instruction requires value"),
            )),
            "BEQ" => instructions.push(Instruction::BranchIfEqual(
                value.expect("BEQ instruction requires value"),
            )),
            "JMP" => instructions.push(Instruction::Jump(
                value.expect("JMP instruction requires value"),
            )),
            _ => return Err(anyhow!("Unknown token {}", instruction)),
        }
    }

    println!(
        "--- Lexed {} Lines in {}ms ({} microseconds)",
        num_lines,
        before_lex.elapsed().as_millis(),
        before_lex.elapsed().as_micros(),
    );

    println!("--- Interpreting Instructions");
    println!("--- Program Output Begins Here");

    let mut labels: HashMap<u16, usize> = instructions
        .iter()
        .enumerate()
        .filter_map(|(i, instruction)| match instruction {
            Instruction::Label(Value::Literal(literal)) => Some((*literal, i)),
            _ => None,
        })
        .collect();

    let memory: &mut [u16; 65535] = &mut [0u16; 65535];
    interpret(&instructions, memory, &mut labels)?;

    println!("");
    println!("--- Interpretation finished");

    Ok(())
}

const ADDRESS_ADDRESS: usize = 65534;
const COMPARE_ADDRESS: usize = 65533;

fn interpret(
    instructions: &Vec<Instruction>,
    memory: &mut [u16; 65535],
    labels: &mut HashMap<u16, usize>,
) -> Result<()> {
    let mut instruction_index = 0;
    let mut caller_stack: Vec<usize> = Vec::new();

    while instruction_index < instructions.len() {
        let instruction = &instructions[instruction_index];
        match instruction {
            Instruction::Output => {
                print!("{}", memory[memory[ADDRESS_ADDRESS] as usize]);
                io::stdout().flush()?;
            }
            Instruction::CharacterOutput => {
                print!(
                    "{}",
                    char::from_u32(memory[memory[ADDRESS_ADDRESS] as usize] as u32)
                        .expect("Invalid character")
                );
                io::stdout().flush()?;
            }
            Instruction::CharacterInput => {
                let mut character = [0];
                io::stdin().read(&mut character)?;
                memory[memory[ADDRESS_ADDRESS] as usize] = character[0] as u32 as u16;
            }
            Instruction::Dump => println!("{:?}", memory),
            Instruction::Return => {
                if caller_stack.len() == 0 {
                    return Err(anyhow!("No caller to return to"));
                }

                instruction_index = caller_stack.pop().unwrap();
            }
            Instruction::SetAddress(new_address) => {
                memory[ADDRESS_ADDRESS] = get_literal_value(new_address, memory)
            }
            Instruction::SetValue(value) => {
                memory[memory[ADDRESS_ADDRESS] as usize] = get_literal_value(value, memory)
            }
            Instruction::Add(other) => {
                memory[memory[ADDRESS_ADDRESS] as usize] =
                    (Wrapping(memory[memory[ADDRESS_ADDRESS] as usize])
                        + Wrapping(get_literal_value(other, memory)))
                    .0
            }
            Instruction::Subtract(other) => {
                memory[memory[ADDRESS_ADDRESS] as usize] =
                    (Wrapping(memory[memory[ADDRESS_ADDRESS] as usize])
                        - Wrapping(get_literal_value(other, memory)))
                    .0
            }
            Instruction::Multiply(other) => {
                memory[memory[ADDRESS_ADDRESS] as usize] =
                    (Wrapping(memory[memory[ADDRESS_ADDRESS] as usize])
                        * Wrapping(get_literal_value(other, memory)))
                    .0
            }
            Instruction::Divide(other) => {
                memory[memory[ADDRESS_ADDRESS] as usize] =
                    (Wrapping(memory[memory[ADDRESS_ADDRESS] as usize])
                        / Wrapping(get_literal_value(other, memory)))
                    .0
            }
            Instruction::Compare(other) => {
                let value = get_literal_value(other, memory);

                memory[COMPARE_ADDRESS] =
                    (memory[memory[ADDRESS_ADDRESS] as usize] == value) as u16;
            }
            Instruction::BranchIfNotEqual(label) => {
                if memory[COMPARE_ADDRESS] == 0 {
                    instruction_index = *labels.get(&get_literal_value(label, memory)).expect(
                        format!(
                            "Use of undeclared label {}",
                            get_literal_value(label, memory)
                        )
                        .as_str(),
                    );
                    continue;
                }
            }
            Instruction::BranchIfEqual(label) => {
                if memory[COMPARE_ADDRESS] == 1 {
                    instruction_index = *labels.get(&get_literal_value(label, memory)).expect(
                        format!(
                            "Use of undeclared label {}",
                            get_literal_value(label, memory)
                        )
                        .as_str(),
                    );
                    continue;
                }
            }
            Instruction::Jump(label) => {
                caller_stack.push(instruction_index);

                instruction_index = *labels.get(&get_literal_value(label, memory)).expect(
                    format!(
                        "Use of undeclared label {}",
                        get_literal_value(label, memory)
                    )
                    .as_str(),
                );

                continue;
            }
            _ => {}
        }

        instruction_index += 1;
    }

    Ok(())
}

fn get_literal_value(val: &Value, memory: &mut [u16; 65535]) -> u16 {
    match val {
        Value::Literal(literal) => *literal,
        Value::Address(address) => memory[*address as usize],
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
