use std::fmt::Display;

use super::FrameLike;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectRecord {
    Header(HeaderRecord),
    Text(TextRecord),
    End(EndRecord),
    Modification(ModificationRecord),
    Define(DefineRecord),
    Refer(ReferRecord),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderRecord {
    pub name: String,
    pub start: u32,
    pub length: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextRecord {
    pub start: u32,
    pub length: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EndRecord {
    pub start: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModificationRecord {
    pub start: u32,
    pub length: u32,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefineRecord {
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReferRecord {
    pub name: String,
}

impl FrameLike for ObjectRecord {
    fn size(&self) -> Option<i32> {
        match self {
            ObjectRecord::Header(_) => Some(0),
            ObjectRecord::Text(r) => Some(r.data.len() as i32),
            ObjectRecord::End(_) => Some(0),
            ObjectRecord::Modification(_) => Some(0),
            ObjectRecord::Define(_) => Some(0),
            ObjectRecord::Refer(_) => Some(0),
        }
    }

    fn parse(
        _operator: &str,
        _operand: Option<&str>,
        _label: Option<&str>,
    ) -> Option<Result<Self, String>>
    where
        Self: Sized,
    {
        // since this frame should not be directly parsed from the source code,
        // this function should never return a result
        None
    }
}

impl Display for ObjectRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectRecord::Header(r) => {
                write!(f, "H {: <6} {:06X} {:06X}", r.name, r.start, r.length)
            }
            ObjectRecord::Text(r) => {
                let data = r
                    .data
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<String>>()
                    .join("");
                write!(f, "T {:06X} {:02X} {}", r.start, r.length, data)
            }
            ObjectRecord::End(r) => write!(f, "E {:06X}", r.start),
            ObjectRecord::Modification(r) => {
                write!(f, "M {:06X} {:02X} {}", r.start, r.length, r.symbol)
            }
            ObjectRecord::Define(r) => write!(f, "D {} {:06X}", r.name, r.value),
            ObjectRecord::Refer(r) => write!(f, "R {}", r.name),
        }
    }
}
