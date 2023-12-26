pub mod directive;
pub mod expression;
pub mod instruction;
pub mod record;
pub mod tokenize;

use std::fmt::{Display, Formatter};

use directive::*;
use expression::*;
use instruction::*;
use record::*;
use tokenize::*;

pub trait FrameLike {
    /// Returns the size of the frame in bytes.
    /// Will be used to offset the location counter.
    /// If the frame is a directive, the size will be 0.
    /// If the frame size cannot be predicted, return None.
    fn size(&self) -> Option<i32>;

    /// Parses the operator, operand and label (only for directives), returns a Frame.
    /// If the operator is not recognized, return None.
    /// If the operator is recognized but the operand is invalid, return an error as a String.
    /// If the operator is recognized and the operand is valid, return the Frame.
    fn parse(
        operator: &str,
        operand: Option<&str>,
        label: Option<&str>,
    ) -> Option<Result<Self, String>>
    where
        Self: Sized;

    /// Access the epressions of a frame if any.
    fn expressions(&self) -> Option<Vec<&Expression>> {
        None
    }
}

/// Frame should be immutable
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    sources: Vec<FrameSource>,
    pub label: Option<String>,
    pub inner: FrameInner,
}

impl Frame {
    pub fn size(&self) -> Option<i32> {
        match &self.inner {
            FrameInner::Instruction(i) => i.size(),
            FrameInner::Directive(d) => d.size(),
            FrameInner::ObjectRecord(r) => r.size(),
        }
    }

    pub fn sources(&self) -> Vec<FrameSource> {
        self.sources.clone()
    }

    pub fn from(inner: FrameInner, label: Option<String>, other: &Frame) -> Frame {
        let mut sources = other.sources().clone();
        sources.push(FrameSource::Frame(other.clone()));

        Frame {
            sources,
            label,
            inner,
        }
    }

    pub fn from_source(source: &str, line: u32) -> Result<Option<Frame>, String> {
        let sources = vec![FrameSource::Source(source.to_string(), line)];
        let tokens = tokenize(source)?;

        match tokens.len() {
            0 => Ok(None),
            1 => {
                let operator = tokens[0].as_str();
                let operand = None;

                let inner = Frame::parse_inner(operator, operand, None);
                if let Ok(inner) = inner {
                    Ok(Some(Frame {
                        sources,
                        label: None,
                        inner,
                    }))
                } else {
                    Err(format!(
                        "{err}\n\tat {src}",
                        err = inner.err().unwrap(),
                        src = sources[0]
                    ))
                }
            }
            2 => {
                // case 1: operator and operand
                let mut label = None;
                let operator = tokens[0].as_str();
                let operand = Some(tokens[1].as_str());

                let mut inner = Frame::parse_inner(operator, operand, label.as_deref());
                // case 2: label and operator
                if inner.is_err() {
                    #[cfg(debug_assertions)]
                    println!("Failed to parse as operator and operand: {}, fallback to label and operator", &inner.err().unwrap());

                    label = Some(tokens[0].as_str().to_string());
                    let operator = tokens[1].as_str();
                    let operand = None;
                    inner = Frame::parse_inner(operator, operand, label.as_deref());
                }

                if let Ok(inner) = inner {
                    Ok(Some(Frame {
                        sources,
                        label,
                        inner,
                    }))
                } else {
                    Err(format!(
                        "{err}\n\tat {src}",
                        err = inner.err().unwrap(),
                        src = sources[0]
                    ))
                }
            }
            3 => {
                let label = Some(tokens[0].as_str().to_string());
                let operator = tokens[1].as_str();
                let operand = Some(tokens[2].as_str());

                let inner = Frame::parse_inner(operator, operand, label.as_deref());
                if let Ok(inner) = inner {
                    Ok(Some(Frame {
                        sources,
                        label,
                        inner,
                    }))
                } else {
                    Err(format!(
                        "{err}\n\tat {src}",
                        err = inner.err().unwrap(),
                        src = sources[0]
                    ))
                }
            }
            _ => Err(format!("Invalid number of tokens.\n\tat {}", sources[0]))?,
        }
    }

    pub fn parse_inner(
        operator: &str,
        operand: Option<&str>,
        label: Option<&str>,
    ) -> Result<FrameInner, String> {
        if let Some(result) = Instruction::parse(operator, operand, label) {
            return result.map(FrameInner::Instruction);
        }

        if let Some(result) = Directive::parse(operator, operand, label) {
            return result.map(FrameInner::Directive);
        }

        Err(format!("Invalid operator \"{}\"", operator))
    }

    /// get expressions from frame
    pub fn expressions(&self) -> Option<Vec<&Expression>> {
        match &self.inner {
            FrameInner::Instruction(i) => i.expressions(),
            FrameInner::Directive(d) => d.expressions(),
            FrameInner::ObjectRecord(r) => r.expressions(),
        }
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{inner}", inner = self.inner)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameSource {
    Source(String, u32),
    Frame(Frame),
}

impl Display for FrameSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameSource::Source(source, line) => write!(f, "Source {source} (Line {line})"),
            FrameSource::Frame(frame) => write!(f, "{frame}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameInner {
    Instruction(Instruction),
    Directive(Directive),
    ObjectRecord(ObjectRecord),
}

impl Display for FrameInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameInner::Instruction(i) => write!(f, "{i:?}"),
            FrameInner::Directive(d) => write!(f, "{d:?}"),
            FrameInner::ObjectRecord(r) => write!(f, "{r:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_sample2() {
        let source = fs::read_to_string("../sample/sample2.asm").unwrap();
        let mut frames = Vec::<Frame>::new();
        for (i, line) in source.lines().enumerate() {
            let frame = Frame::from_source(line, i as u32 + 1).unwrap();
            if let Some(frame) = frame {
                frames.push(frame);
            }
        }

        println!("{:#?}", frames);

        assert_eq!(frames.len(), 46);
    }

    #[test]
    fn parse_code2() {
        let source = fs::read_to_string("../sample/code2.asm").unwrap();
        let mut frames = Vec::<Frame>::new();
        for (i, line) in source.lines().enumerate() {
            let frame = Frame::from_source(line, i as u32 + 1).unwrap();
            if let Some(frame) = frame {
                frames.push(frame);
            }
        }

        println!("{:#?}", frames);

        assert_eq!(frames.len(), 51);
    }

    #[test]
    fn parse_code3() {
        let source = fs::read_to_string("../sample/code3.asm").unwrap();
        let mut frames = Vec::<Frame>::new();
        for (i, line) in source.lines().enumerate() {
            let frame = Frame::from_source(line, i as u32 + 1).unwrap();
            if let Some(frame) = frame {
                frames.push(frame);
            }
        }

        println!("{:#?}", frames);

        assert_eq!(frames.len(), 52);
    }
}
