use crate::frame::{
    directive::{literal_to_data, Directive, BYTE},
    expression::parse,
    instruction::Format34,
    *,
};
use std::collections::HashMap;

pub fn dump_literals(program: Vec<Frame>) -> Vec<Frame> {
    let mut program = program;
    let mut literal_pool = HashMap::<String, String>::new();

    let mut literal_count = 0;
    let mut i = 0;
    while i < program.len() {
        let frame = &mut program[i];
        match &mut frame.inner {
            FrameInner::Directive(d) => match d {
                Directive::LTORG(_) => {
                    let bytes = dump_pool(&literal_pool, frame);
                    program.splice(i..=i, bytes);
                    literal_pool.clear();
                }
                Directive::END(_) => {
                    let bytes = dump_pool(&literal_pool, frame);
                    program.splice(i..i, bytes);
                    literal_pool.clear();
                }
                _ => {}
            },
            FrameInner::Instruction(i) => match i {
                instruction::Instruction::Format34(ref mut i) => {
                    if let Some(literal) = get_literal(i) {
                        let reference =
                            get_literal_reference(&mut literal_pool, literal, &mut literal_count);
                        i.value = Box::new(parse(&reference).unwrap());
                    }
                }
                _ => {}
            },
            _ => {}
        }

        i += 1;
    }

    program
}

fn dump_pool(literal_pool: &HashMap<String, String>, ltorg: &Frame) -> Vec<Frame> {
    let mut frames = Vec::new();

    for (value, symbol) in literal_pool {
        let data = literal_to_data(value).unwrap();
        let frame = Frame::from(
            FrameInner::Directive(Directive::BYTE(BYTE { data })),
            Some(symbol.clone()),
            ltorg,
        );
        frames.push(frame);
    }

    frames
}

fn get_literal(frame: &Format34) -> Option<String> {
    match *frame.value {
        expression::Expression::Literal(ref l) => Some(l.clone()),
        _ => None,
    }
}

fn get_literal_reference(
    literal_pool: &mut HashMap<String, String>,
    content: String,
    i: &mut u32,
) -> String {
    // if the literal is already in the pool, return the reference
    if let Some(reference) = literal_pool.get(&content) {
        return reference.clone();
    }

    // otherwise, add the literal to the pool and return the reference
    let key = format!("_L{i:04X}");
    *i += 1;
    literal_pool.insert(content, key.clone());
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_literal() {
        let source = fs::read_to_string("../sample/literals1.asm").unwrap();
        let mut frames = Vec::<Frame>::new();
        for (i, line) in source.lines().enumerate() {
            let frame = Frame::from_source(line, i as u32 + 1).unwrap();
            if let Some(frame) = frame {
                frames.push(frame);
            }
        }

        let frames = dump_literals(frames);
        dbg!(&frames);

        assert_eq!(frames.len(), 10);
        match frames[1].inner {
            FrameInner::Instruction(ref i) => match i {
                instruction::Instruction::Format34(ref i) => {
                    assert_eq!(i.value.to_string(), "_L0000".to_string());
                }
                _ => panic!("Expected format 3/4 instruction"),
            },
            _ => panic!("Expected instruction"),
        }
        match frames[2].inner {
            FrameInner::Instruction(ref i) => match i {
                instruction::Instruction::Format34(ref i) => {
                    assert_eq!(i.value.to_string(), "_L0001".to_string());
                }
                _ => panic!("Expected format 3/4 instruction"),
            },
            _ => panic!("Expected instruction"),
        }

        if frames[3].label == Some("_L0000".to_string()) {
            match frames[3].inner {
                FrameInner::Directive(ref d) => match d {
                    Directive::BYTE(ref b) => {
                        assert_eq!(b.data, vec![0xF1]);
                    }
                    _ => panic!("Expected BYTE directive"),
                },
                _ => panic!("Expected directive"),
            }
            match frames[4].inner {
                FrameInner::Directive(ref d) => match d {
                    Directive::BYTE(ref b) => {
                        assert_eq!(b.data, vec![0x45, 0x4F, 0x46]);
                    }
                    _ => panic!("Expected BYTE directive"),
                },
                _ => panic!("Expected directive"),
            }
        } else if frames[3].label == Some("_L0001".to_string()) {
            match frames[3].inner {
                FrameInner::Directive(ref d) => match d {
                    Directive::BYTE(ref b) => {
                        assert_eq!(b.data, vec![0x45, 0x4F, 0x46]);
                    }
                    _ => panic!("Expected BYTE directive"),
                },
                _ => panic!("Expected directive"),
            }
            match frames[4].inner {
                FrameInner::Directive(ref d) => match d {
                    Directive::BYTE(ref b) => {
                        assert_eq!(b.data, vec![0xF1]);
                    }
                    _ => panic!("Expected BYTE directive"),
                },
                _ => panic!("Expected directive"),
            }
        } else {
            panic!("Expected literal");
        }
    }
}
