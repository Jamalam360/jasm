use anyhow::Result;

use crate::instructions::{Instruction, Value};

pub fn compile(instructions: Vec<Instruction>) -> Result<Vec<u8>> {
    let mut compiled: Vec<u8> = Vec::new();

    for instruction in instructions {
        if instruction.requires_value() {
            push_instruction_with_value(&mut compiled, &instruction);
            push_value(&mut compiled, &instruction.get_value());
        } else {
            compiled.push(0x00);
            compiled.push(instruction.to_u8()?);
        }
    }

    Ok(compiled)
}

fn push_instruction_with_value(compiled: &mut Vec<u8>, instruction: &Instruction) {
    let value = instruction.get_value();

    compiled.push(match value {
        Value::Literal(_) => 0x00,
        Value::Address(_) => 0x10,
    });

    compiled.push(instruction.to_u8().unwrap());
}

fn push_value(compiled: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Literal(val) => {
            compiled.push((val >> 8) as u8);
            compiled.push(*val as u8);
        }
        Value::Address(val) => {
            compiled.push((val >> 8) as u8);
            compiled.push(*val as u8);
        }
    };
}
