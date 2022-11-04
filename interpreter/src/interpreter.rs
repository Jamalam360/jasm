use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    io::{self, Write},
    num::Wrapping,
};

use crate::{
    instructions::{get_literal_value, Instruction},
    Memory, ADDRESS_ADDRESS, COMPARISON_ADDRESS,
};

pub fn interpret(
    instructions: &Vec<Instruction>,
    memory: &mut Memory,
    labels: &mut HashMap<u16, usize>,
) -> Result<u16> {
    let mut instruction_index = 0;
    let mut caller_stack: Vec<usize> = Vec::new();
    let term = console::Term::stdout();

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
                let char = console::Term::read_char(&term)?;
                memory[memory[ADDRESS_ADDRESS] as usize] = char as u32 as u16;
                print!("{}", char);
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

                memory[COMPARISON_ADDRESS] =
                    (memory[memory[ADDRESS_ADDRESS] as usize] == value) as u16;
            }
            Instruction::BranchIfNotEqual(label) => {
                if memory[COMPARISON_ADDRESS] == 0 {
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
                if memory[COMPARISON_ADDRESS] == 1 {
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

    Ok(0)
}
