use crate::Memory;
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub enum Instruction {
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
    GreaterThan(Value),
    LessThan(Value),
    GreaterThanEqual(Value),
    LessThanEqual(Value),
    BranchIfNotEqual(Value),
    BranchIfEqual(Value),
    Jump(Value),
    Exit(Value),
}

impl Instruction {
    pub fn u8_requires_value(instruction: u8) -> bool {
        match instruction {
            0x00 => false,
            0x01 => false,
            0x02 => false,
            0x03 => false,
            0x04 => false,
            _ => true,
        }
    }

    pub fn requires_value(&self) -> bool {
        match self {
            Self::Output => false,
            Self::CharacterOutput => false,
            Self::CharacterInput => false,
            Self::Dump => false,
            Self::Return => false,
            _ => true,
        }
    }

    pub fn get_value(&self) -> Value {
        match self {
            Instruction::SetAddress(value) => *value,
            Instruction::SetValue(value) => *value,
            Instruction::Add(value) => *value,
            Instruction::Subtract(value) => *value,
            Instruction::Multiply(value) => *value,
            Instruction::Divide(value) => *value,
            Instruction::Label(value) => *value,
            Instruction::Compare(value) => *value,
            Instruction::GreaterThan(value) => *value,
            Instruction::LessThan(value) => *value,
            Instruction::GreaterThanEqual(value) => *value,
            Instruction::LessThanEqual(value) => *value,
            Instruction::BranchIfNotEqual(value) => *value,
            Instruction::BranchIfEqual(value) => *value,
            Instruction::Jump(value) => *value,
            Instruction::Exit(value) => *value,
            _ => panic!("Variant {:?} does not have a value", self),
        }
    }

    pub fn from_string(string: &str, value: Option<Value>) -> Result<Self>
    where
        Self: Sized,
    {
        return Ok(match string {
            "OUT" => Instruction::Output,
            "CUT" => Instruction::CharacterOutput,
            "CIN" => Instruction::CharacterInput,
            "DMP" => Instruction::Dump,
            "RTN" => Instruction::Return,
            "SEA" => Instruction::SetAddress(value.expect("SEA instruction requires value")),
            "SET" => Instruction::SetValue(value.expect("SET instruction requires value")),
            "ADD" => Instruction::Add(value.expect("ADD instruction requires value")),
            "SUB" => Instruction::Subtract(value.expect("SUB instruction requires value")),
            "MUL" => Instruction::Multiply(value.expect("MUL instruction requires value")),
            "DIV" => Instruction::Divide(value.expect("DIV instruction requires value")),
            "LAB" => Instruction::Label(value.expect("LAB instruction requires value")),
            "CEQ" => Instruction::Compare(value.expect("CEQ instruction requires value")),
            "GTN" => Instruction::GreaterThan(value.expect("GTN instruction requires value")),
            "LTN" => Instruction::LessThan(value.expect("LTN instruction requires value")),
            "GTE" => Instruction::GreaterThanEqual(value.expect("GTE instruction requires value")),
            "LTE" => Instruction::LessThanEqual(value.expect("LTE instruction requires value")),
            "BNE" => Instruction::BranchIfNotEqual(value.expect("BNE instruction requires value")),
            "BEQ" => Instruction::BranchIfEqual(value.expect("BEQ instruction requires value")),
            "JMP" => Instruction::Jump(value.expect("JMP instruction requires value")),
            "EXT" => Instruction::Exit(value.expect("EXT instruction requires value")),
            _ => return Err(anyhow!("Unknown token {}", string)),
        });
    }

    pub fn from_u8(instruction: [u8; 2], raw_value: Option<u16>) -> Result<Self>
    where
        Self: Sized,
    {
        let mut value: Option<Value> = None;
        if let Some(v) = raw_value {
            value = Some(if instruction[0] > 0x00 {
                Value::Address(v)
            } else {
                Value::Literal(v)
            });
        }

        let instruction: u16 = instruction[1] as u16;

        Ok(match instruction {
            0x00 => Instruction::Output,
            0x01 => Instruction::CharacterOutput,
            0x02 => Instruction::CharacterInput,
            0x03 => Instruction::Dump,
            0x04 => Instruction::Return,
            0x05 => Instruction::SetAddress(value.expect("SEA instruction requires value")),
            0x06 => Instruction::SetValue(value.expect("SET instruction requires value")),
            0x07 => Instruction::Add(value.expect("ADD instruction requires value")),
            0x08 => Instruction::Subtract(value.expect("SUB instruction requires value")),
            0x09 => Instruction::Multiply(value.expect("MUL instruction requires value")),
            0x0A => Instruction::Divide(value.expect("DIV instruction requires value")),
            0x0B => Instruction::Label(value.expect("LAB instruction requires value")),
            0x0C => Instruction::Compare(value.expect("CEQ instruction requires value")),
            0x0D => Instruction::GreaterThan(value.expect("GTN instruction requires value")),
            0x0E => Instruction::LessThan(value.expect("LTN instruction requires value")),
            0x0F => Instruction::GreaterThanEqual(value.expect("GTE instruction requires value")),
            0x10 => Instruction::LessThanEqual(value.expect("LTE instruction requires value")),
            0x11 => Instruction::BranchIfNotEqual(value.expect("BNE instruction requires value")),
            0x12 => Instruction::BranchIfEqual(value.expect("BEQ instruction requires value")),
            0x13 => Instruction::Jump(value.expect("JMP instruction requires value")),
            0x14 => Instruction::Exit(value.expect("EXT instruction requires value")),
            _ => return Err(anyhow!("Unknown token {}", instruction)),
        })
    }

    pub fn to_u8(&self) -> Result<u8> {
        Ok(match self {
            Instruction::Output => 0x00,
            Instruction::CharacterOutput => 0x01,
            Instruction::CharacterInput => 0x02,
            Instruction::Dump => 0x03,
            Instruction::Return => 0x04,
            Instruction::SetAddress(_) => 0x05,
            Instruction::SetValue(_) => 0x06,
            Instruction::Add(_) => 0x07,
            Instruction::Subtract(_) => 0x08,
            Instruction::Multiply(_) => 0x09,
            Instruction::Divide(_) => 0x0A,
            Instruction::Label(_) => 0x0B,
            Instruction::Compare(_) => 0x0C,
            Instruction::GreaterThan(_) => 0x0D,
            Instruction::LessThan(_) => 0x0E,
            Instruction::GreaterThanEqual(_) => 0x0F,
            Instruction::LessThanEqual(_) => 0x10,
            Instruction::BranchIfNotEqual(_) => 0x11,
            Instruction::BranchIfEqual(_) => 0x12,
            Instruction::Jump(_) => 0x13,
            Instruction::Exit(_) => 0x14,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Literal(u16),
    Address(u16),
}

pub fn get_literal_value(val: &Value, memory: &mut Memory) -> u16 {
    match val {
        Value::Literal(literal) => *literal,
        Value::Address(address) => memory[*address as usize],
    }
}
