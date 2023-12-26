use crate::frame::*;
use std::collections::HashMap;

pub const DEFAULT_BLOCK_NAME: &str = "";

pub fn rearrange_blocks(programs: Vec<Frame>) -> Vec<Frame> {
    let mut block_frames = HashMap::<String, Vec<Frame>>::new();
    let mut block_order = Vec::<String>::new();
    let mut current_block = DEFAULT_BLOCK_NAME.to_string();

    let start_frame = programs.first().unwrap();
    let end_frame = programs.last().unwrap();
    let extdefs = programs
        .iter()
        .filter(|frame| {
            matches!(
                frame.inner,
                FrameInner::Directive(directive::Directive::EXTDEF(_))
            )
        })
        .cloned()
        .collect::<Vec<Frame>>();
    let extrefs = programs
        .iter()
        .filter(|frame| {
            matches!(
                frame.inner,
                FrameInner::Directive(directive::Directive::EXTREF(_))
            )
        })
        .cloned()
        .collect::<Vec<Frame>>();

    let rerrangables = programs
        .iter()
        .filter(|frame| match frame.inner {
            FrameInner::Directive(ref d) => !matches!(
                d,
                directive::Directive::START(_)
                    | directive::Directive::END(_)
                    | directive::Directive::EXTDEF(_)
                    | directive::Directive::EXTREF(_)
            ),
            _ => true,
        })
        .cloned()
        .collect::<Vec<Frame>>();

    #[cfg(debug_assertions)]
    println!("current block: {}", &current_block);

    for frame in rerrangables {
        let switch_to = match frame.inner {
            FrameInner::Directive(directive::Directive::USE(ref u)) => Some(u.name.clone()),
            _ => None,
        };

        if let Some(name) = switch_to {
            current_block = name;
            #[cfg(debug_assertions)]
            println!("current block: {}", &current_block);
        } else {
            if !block_frames.contains_key(&current_block) {
                block_frames.insert(current_block.clone(), Vec::new());
            }
            if !block_order.contains(&current_block) {
                block_order.push(current_block.clone());
            }

            block_frames.get_mut(&current_block).unwrap().push(frame);
        }
    }

    let mut frames = vec![start_frame.clone()];
    frames.append(&mut extdefs.clone());
    frames.append(&mut extrefs.clone());
    for block in block_order {
        frames.append(&mut block_frames.get_mut(&block).unwrap().clone());
    }
    frames.push(end_frame.clone());

    frames
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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

        let frames = rearrange_blocks(frames);
        dbg!(&frames);

        assert_eq!(frames.len(), 45);
        match frames[13].inner {
            FrameInner::Instruction(ref i) => {
                match i {
                    instruction::Instruction::Format34(ref i) => {
                        // J
                        assert_eq!(i.opcode, 0x3C);
                    }
                    _ => panic!("Expected format 3/4 instruction"),
                }
            }
            _ => panic!("Expected instruction"),
        }
        match frames[14].inner {
            FrameInner::Instruction(ref i) => {
                match i {
                    instruction::Instruction::Format2(ref i) => {
                        // CLEAR
                        assert_eq!(i.opcode, 0xB4);
                    }
                    _ => panic!("Expected format 2 instruction"),
                }
            }
            _ => panic!("Expected instruction"),
        }
    }
}
