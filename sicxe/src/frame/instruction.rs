use std::fmt::Display;

use super::expression::*;
use super::FrameLike;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Format1(Format1),
    Format2(Format2),
    Format34(Format34),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Format1 {
    pub opcode: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Format2 {
    pub opcode: u8,
    pub register1: Box<Expression>,
    pub register2: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Format34 {
    pub opcode: u8,
    pub nixbpe: u8,
    pub value: Box<Expression>,
}

impl Format34 {
    pub fn is_format4(&self) -> bool {
        self.nixbpe & 0b000001 != 0
    }

    pub fn is_immediate(&self) -> bool {
        self.nixbpe & 0b010000 != 0
    }

    pub fn is_indirect(&self) -> bool {
        self.nixbpe & 0b100000 != 0
    }

    pub fn is_indexed(&self) -> bool {
        self.nixbpe & 0b001000 != 0
    }

    pub fn set_base(&mut self) {
        self.nixbpe |= 0b000100;
        self.nixbpe &= 0b111101;
    }

    pub fn set_pc(&mut self) {
        self.nixbpe |= 0b000010;
        self.nixbpe &= 0b111011;
    }

    pub fn set_extended(&mut self) {
        self.nixbpe |= 0b000001;
    }
}

impl FrameLike for Instruction {
    fn size(&self) -> Option<i32> {
        match self {
            Instruction::Format1(_) => Some(1),
            Instruction::Format2(_) => Some(2),
            Instruction::Format34(f) => {
                if f.nixbpe & 0b000001 == 0 {
                    Some(3)
                } else {
                    Some(4)
                }
            }
        }
    }

    fn parse(
        operator: &str,
        operand: Option<&str>,
        _label: Option<&str>,
    ) -> Option<Result<Self, String>> {
        let is_format4 = operator.starts_with('+');
        let operator = operator.trim_start_matches('+');
        let format = match operator {
            "ADDR" | "CLEAR" | "COMPR" | "DIVR" | "MULR" | "RMO" | "SHIFTL" | "SHIFTR" | "SUBR"
            | "TIXR" => 2,
            "ADD" | "AND" | "COMP" | "DIV" | "J" | "JEQ" | "JGT" | "JLT" | "JSUB" | "LDA"
            | "LDCH" | "LDL" | "LDX" | "MUL" | "OR" | "RD" | "RSUB" | "STA" | "STCH" | "STL"
            | "STX" | "SUB" | "TD" | "TIX" | "WD" => 3,
            "LDB" | "LDS" | "LDT" | "STB" | "STS" | "STT" => 3,
            _ => 0,
        };

        match format {
            2 => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }

                let opcode = match operator {
                    "ADDR" => Some(0x90),
                    "CLEAR" => Some(0xB4),
                    "COMPR" => Some(0xA0),
                    "DIVR" => Some(0x9C),
                    "MULR" => Some(0x98),
                    "RMO" => Some(0xAC),
                    "SHIFTL" => Some(0xA4),
                    "SHIFTR" => Some(0xA8),
                    "SUBR" => Some(0x94),
                    "TIXR" => Some(0xB8),
                    _ => None,
                };
                opcode?;

                let mut operands = operand
                    .unwrap()
                    .split(',')
                    .collect::<Vec<&str>>()
                    .into_iter();
                let register1: &str;
                let mut register2 = "0";
                match operator {
                    "CLEAR" | "TIXR" => {
                        if operands.len() != 1 {
                            return Some(Err("Invalid number of operands".to_string()));
                        }

                        register1 = operands.next().unwrap().trim();
                    }
                    _ => {
                        if operands.len() != 2 {
                            return Some(Err("Invalid number of operands".to_string()));
                        }

                        register1 = operands.next().unwrap().trim();
                        register2 = operands.next().unwrap().trim();
                    }
                }

                let register1 = parse(register1).unwrap();
                let register2 = parse(register2).unwrap();
                let register1 = Box::new(register1);
                let register2 = Box::new(register2);

                Some(Ok(Instruction::Format2(Format2 {
                    opcode: opcode.unwrap(),
                    register1,
                    register2,
                })))
            }
            3 => {
                let has_operand = operand.is_some();
                let mut operand = operand;
                if operator == "RSUB" {
                    if has_operand {
                        return Some(Err("Invalid operand".to_string()));
                    }

                    operand = Some("0");
                } else if !has_operand {
                    return Some(Err("Missing operand".to_string()));
                }

                let operand = operand.unwrap();
                let is_indirect = operand.starts_with('@');
                let is_immediate = operand.starts_with('#');
                let is_indexed = operand.ends_with(",X");
                let operand = operand
                    .trim_start_matches('@')
                    .trim_start_matches('#')
                    .trim_end_matches(",X");

                let nixbpe = if is_format4 { 0b000001 } else { 0b000000 }
                    | if is_immediate { 0b010000 } else { 0b000000 }
                    | if is_indirect { 0b100000 } else { 0b000000 }
                    | if is_indexed { 0b001000 } else { 0b000000 };

                let value = parse(operand).ok()?;
                let value = Box::new(value);
                match operator {
                    "ADD" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x18,
                        nixbpe,
                        value,
                    }))),
                    "AND" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x40,
                        nixbpe,
                        value,
                    }))),
                    "COMP" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x28,
                        nixbpe,
                        value,
                    }))),
                    "DIV" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x24,
                        nixbpe,
                        value,
                    }))),
                    "J" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x3C,
                        nixbpe,
                        value,
                    }))),
                    "JEQ" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x30,
                        nixbpe,
                        value,
                    }))),
                    "JGT" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x34,
                        nixbpe,
                        value,
                    }))),
                    "JLT" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x38,
                        nixbpe,
                        value,
                    }))),
                    "JSUB" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x48,
                        nixbpe,
                        value,
                    }))),
                    "LDA" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x00,
                        nixbpe,
                        value,
                    }))),
                    "LDCH" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x50,
                        nixbpe,
                        value,
                    }))),
                    "LDL" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x08,
                        nixbpe,
                        value,
                    }))),
                    "LDX" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x04,
                        nixbpe,
                        value,
                    }))),
                    "MUL" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x20,
                        nixbpe,
                        value,
                    }))),
                    "OR" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x44,
                        nixbpe,
                        value,
                    }))),
                    "RD" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0xD8,
                        nixbpe,
                        value,
                    }))),
                    "RSUB" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x4C,
                        nixbpe,
                        value,
                    }))),
                    "STA" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x0C,
                        nixbpe,
                        value,
                    }))),
                    "STCH" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x54,
                        nixbpe,
                        value,
                    }))),
                    "STL" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x14,
                        nixbpe,
                        value,
                    }))),
                    "STX" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x10,
                        nixbpe,
                        value,
                    }))),
                    "SUB" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x1C,
                        nixbpe,
                        value,
                    }))),
                    "TD" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0xE0,
                        nixbpe,
                        value,
                    }))),
                    "TIX" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x2C,
                        nixbpe,
                        value,
                    }))),
                    "WD" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0xDC,
                        nixbpe,
                        value,
                    }))),
                    "LDB" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x68,
                        nixbpe,
                        value,
                    }))),
                    "LDS" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x6C,
                        nixbpe,
                        value,
                    }))),
                    "LDT" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x74,
                        nixbpe,
                        value,
                    }))),
                    "STB" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x78,
                        nixbpe,
                        value,
                    }))),
                    "STS" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x7C,
                        nixbpe,
                        value,
                    }))),
                    "STT" => Some(Ok(Instruction::Format34(Format34 {
                        opcode: 0x84,
                        nixbpe,
                        value,
                    }))),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn expressions(&self) -> Option<Vec<&Expression>> {
        match self {
            Instruction::Format1(_) => None,
            Instruction::Format2(f) => Some(vec![&f.register1, &f.register2]),
            Instruction::Format34(f) => Some(vec![&f.value]),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Format1(ref i) => write!(f, "op: 0x{:02X}", i.opcode),
            Instruction::Format2(ref i) => {
                let r1 = i.register1.to_string().parse::<u8>().unwrap();
                let r2 = i.register2.to_string().parse::<u8>().unwrap();
                write!(f, "op: 0x{:02X}, r1: {:01X}, r2: {:01X}", i.opcode, r1, r2)
            }
            Instruction::Format34(ref i) => {
                write!(
                    f,
                    "op: 0x{:02X}, nixbpe: 0b{:06b}, value: {}",
                    i.opcode, i.nixbpe, i.value
                )
            }
        }
    }
}
