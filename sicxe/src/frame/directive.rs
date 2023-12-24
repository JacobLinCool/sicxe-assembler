use std::fmt::Display;

use super::expression::*;
use super::FrameLike;

#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    START(START),
    END(END),
    BYTE(BYTE),
    WORD(WORD),
    RESB(RESB),
    RESW(RESW),
    ORG(ORG),
    BASE(BASE),
    NOBASE(NOBASE),
    EQU(EQU),
    LTORG(LTORG),
    USE(USE),
    CSECT(CSECT),
    EXTREF(EXTREF),
    EXTDEF(EXTDEF),
}

#[derive(Debug, Clone, PartialEq)]
pub struct START {
    pub name: String,
    pub address: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct END {
    pub first: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BYTE {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WORD {
    pub word: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RESB {
    pub bytes: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RESW {
    pub words: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ORG {
    pub address: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BASE {
    pub address: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NOBASE;

#[derive(Debug, Clone, PartialEq)]
pub struct EQU {
    pub value: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LTORG;

#[derive(Debug, Clone, PartialEq)]
pub struct USE {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CSECT {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EXTREF {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EXTDEF {
    pub names: Vec<String>,
}

// Implement FrameLike for Directive
impl FrameLike for Directive {
    fn size(&self) -> Option<i32> {
        match self {
            Directive::START(_) => Some(0),
            Directive::END(_) => Some(0),
            Directive::BYTE(d) => Some(d.data.len() as i32),
            Directive::WORD(_) => Some(3),
            Directive::RESB(d) => d.bytes.eval(),
            Directive::RESW(d) => {
                let words = d.words.eval();
                words.map(|words| words * 3)
            }
            Directive::ORG(_) => Some(0),
            Directive::BASE(_) => Some(0),
            Directive::NOBASE(_) => Some(0),
            Directive::EQU(_) => Some(0),
            Directive::LTORG(_) => Some(0),
            Directive::USE(_) => Some(0),
            Directive::CSECT(_) => Some(0),
            Directive::EXTREF(_) => Some(0),
            Directive::EXTDEF(_) => Some(0),
        }
    }

    fn parse(
        operator: &str,
        operand: Option<&str>,
        label: Option<&str>,
    ) -> Option<Result<Self, String>>
    where
        Self: Sized,
    {
        match operator {
            "START" => {
                if label.is_none() {
                    return Some(Err("Missing label".to_string()));
                }
                let label = label.unwrap();

                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let address = u32::from_str_radix(operand, 16)
                    .map_err(|_| "Failed to parse address".to_string())
                    .ok()?;
                Some(Ok(Directive::START(START {
                    name: label.to_string(),
                    address,
                })))
            }
            "END" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let first = parse(operand).ok()?;
                let first = Box::new(first);
                Some(Ok(Directive::END(END { first })))
            }
            "BYTE" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();
                let data = literal_to_data(operand).ok()?;
                Some(Ok(Directive::BYTE(BYTE { data })))
            }
            "WORD" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let word = parse(operand).ok()?;
                let word = Box::new(word);
                Some(Ok(Directive::WORD(WORD { word })))
            }
            "RESB" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let bytes = parse(operand).ok()?;
                let bytes = Box::new(bytes);
                Some(Ok(Directive::RESB(RESB { bytes })))
            }
            "RESW" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let words = parse(operand).ok()?;
                let words = Box::new(words);
                Some(Ok(Directive::RESW(RESW { words })))
            }
            "ORG" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let address = parse(operand).ok()?;
                let address = Box::new(address);
                Some(Ok(Directive::ORG(ORG { address })))
            }
            "BASE" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let address = parse(operand).ok()?;
                let address = Box::new(address);
                Some(Ok(Directive::BASE(BASE { address })))
            }
            "NOBASE" => {
                if operand.is_some() {
                    return Some(Err("Unexpected operand".to_string()));
                }
                Some(Ok(Directive::NOBASE(NOBASE)))
            }
            "EQU" => {
                if label.is_none() {
                    return Some(Err("Missing label".to_string()));
                }
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let value = parse(operand).ok()?;
                let value = Box::new(value);
                Some(Ok(Directive::EQU(EQU { value })))
            }
            "LTORG" => {
                if operand.is_some() {
                    return Some(Err("Unexpected operand".to_string()));
                }
                Some(Ok(Directive::LTORG(LTORG)))
            }
            "USE" => {
                let operand = operand.unwrap_or("");

                Some(Ok(Directive::USE(USE {
                    name: operand.to_string(),
                })))
            }
            "CSECT" => {
                if label.is_none() {
                    return Some(Err("Missing label".to_string()));
                }
                let label = label.unwrap();

                Some(Ok(Directive::CSECT(CSECT {
                    name: label.to_string(),
                })))
            }
            "EXTREF" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let names = operand.split(',').map(|s| s.to_string()).collect();
                Some(Ok(Directive::EXTREF(EXTREF { names })))
            }
            "EXTDEF" => {
                if operand.is_none() {
                    return Some(Err("Missing operand".to_string()));
                }
                let operand = operand.unwrap();

                let names = operand.split(',').map(|s| s.to_string()).collect();
                Some(Ok(Directive::EXTDEF(EXTDEF { names })))
            }
            _ => None,
        }
    }
}

impl Display for Directive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Directive::START(s) => write!(f, "{name:<6}\tSTART\t{address:04X}", name = s.name, address = s.address),
            Directive::END(e) => write!(f, "      \tEND\t{}", e.first),
            Directive::BYTE(b) => {
                let data = b
                    .data
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<String>>()
                    .join("");
                write!(f, "      \tBYTE\t{data}")
            }
            Directive::WORD(w) => write!(f, "      \tWORD\t{}", w.word),
            Directive::RESB(r) => write!(f, "      \tRESB\t{}", r.bytes),
            Directive::RESW(r) => write!(f, "      \tRESW\t{}", r.words),
            Directive::ORG(o) => write!(f, "      \tORG\t{}", o.address),
            Directive::BASE(b) => write!(f, "      \tBASE\t{}", b.address),
            Directive::NOBASE(_) => write!(f, "      \tNOBASE"),
            Directive::EQU(e) => write!(f, "      \tEQU\t{}", e.value),
            Directive::LTORG(_) => write!(f, "      \tLTORG"),
            Directive::USE(u) => write!(f, "      \tUSE\t{}", u.name),
            Directive::CSECT(c) => write!(f, "{name:<6}\tCSECT", name = c.name),
            Directive::EXTREF(e) => {
                let names = e.names.join(",");
                write!(f, "      \tEXTREF\t{names}")
            }
            Directive::EXTDEF(e) => {
                let names = e.names.join(",");
                write!(f, "      \tEXTDEF\t{names}")
            }
        }
    }
}

pub fn literal_to_data(operand: &str) -> Result<Vec<u8>, String> {
    if operand.starts_with("C'") && operand.ends_with('\'') {
        let data = operand[2..operand.len() - 1].as_bytes().to_vec();
        Ok(data)
    } else if operand.starts_with("X'") && operand.ends_with('\'') {
        let operand = if operand.len() % 2 == 0 {
            let mut operand = operand[2..operand.len() - 1].to_string();
            operand.insert(0, '0');
            operand
        } else {
            operand[2..operand.len() - 1].to_string()
        };

        let data = operand
            .as_bytes()
            .chunks(2)
            .map(|chunk| {
                let hex_str = std::str::from_utf8(chunk).unwrap();
                u8::from_str_radix(hex_str, 16).unwrap()
            })
            .collect::<Vec<u8>>();
        Ok(data)
    } else {
        Err("Invalid literal".to_string())
    }
}
