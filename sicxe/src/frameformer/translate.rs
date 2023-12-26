use crate::frame::{
    expression::{Expression, ExpressionOperand},
    record::{EndRecord, HeaderRecord, ModificationRecord, ObjectRecord, TextRecord},
    *,
};

pub fn translate_to_record(program: Vec<Frame>) -> Result<Vec<ObjectRecord>, String> {
    let r_records = program
        .iter()
        .filter(|frame| {
            matches!(
                frame.inner,
                FrameInner::ObjectRecord(ObjectRecord::Refer(_))
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let d_records = program
        .iter()
        .filter(|frame| {
            matches!(
                frame.inner,
                FrameInner::ObjectRecord(ObjectRecord::Define(_))
            )
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut program = program
        .into_iter()
        .filter(|frame| !matches!(frame.inner, FrameInner::ObjectRecord(_)))
        .collect::<Vec<_>>();

    let mut t_records = Vec::<Frame>::new();
    let mut m_records = Vec::<Frame>::new();
    let mut h_record: Option<Frame> = None;
    let mut e_record: Option<Frame> = None;

    let mut start = 0;
    let mut locctr = Some(0);
    let mut base: Option<u32> = None;
    for frame in &mut program {
        let size = frame.size();
        // set locctr to the address of START or ORG
        if let FrameInner::Directive(ref d) = frame.inner {
            match d {
                directive::Directive::START(s) => {
                    locctr = Some(s.address);
                    start = s.address;
                }
                directive::Directive::ORG(o) => {
                    locctr = Some(o.address.eval().unwrap() as u32);
                }
                directive::Directive::BASE(b) => {
                    base = Some(b.address.eval().unwrap() as u32);
                }
                directive::Directive::NOBASE(_) => {
                    base = None;
                }
                _ => {}
            }
        }

        // translate H, E, T and M records
        match frame.inner {
            FrameInner::Instruction(ref i) => {
                match i {
                    instruction::Instruction::Format1(i) => {
                        t_records.push(Frame::from(
                            FrameInner::ObjectRecord(ObjectRecord::Text(TextRecord {
                                start: locctr.unwrap(),
                                length: 1,
                                data: vec![i.opcode],
                            })),
                            None,
                            frame,
                        ));
                    }
                    instruction::Instruction::Format2(i) => {
                        let r1 = i.register1.eval().unwrap() as u8;
                        let r2 = i.register2.eval().unwrap() as u8;
                        let operand = r1 << 4 | r2;
                        t_records.push(Frame::from(
                            FrameInner::ObjectRecord(ObjectRecord::Text(TextRecord {
                                start: locctr.unwrap(),
                                length: 2,
                                data: vec![i.opcode, operand],
                            })),
                            None,
                            frame,
                        ));
                    }
                    instruction::Instruction::Format34(i) => {
                        // RSUB
                        if i.opcode == 0x4C {
                            t_records.push(Frame::from(
                                FrameInner::ObjectRecord(ObjectRecord::Text(TextRecord {
                                    start: locctr.unwrap(),
                                    length: 3,
                                    data: vec![0x4C | 0b11, 0, 0],
                                })),
                                None,
                                frame,
                            ));
                        } else {
                            // IT'A A MESS!! I DON'T KNOW WHAT I'M DOING!!
                            let mut data = vec![];
                            let length = i.is_format4() as u32 + 3;
                            let operand = i.value.eval();
                            let (operand, external) = match operand {
                                Some(v) => (v, false),
                                _ => {
                                    // external reference involved
                                    // create modification record
                                    let mut expr = *i.value.clone();
                                    if let Expression::Unsolved(ref mut u) = expr {
                                        if let ExpressionOperand::Symbol(s) = &u.left {
                                            m_records.push(Frame::from(
                                                FrameInner::ObjectRecord(
                                                    ObjectRecord::Modification(
                                                        ModificationRecord {
                                                            start: locctr.unwrap() + 1,
                                                            length: if i.is_format4() {
                                                                5
                                                            } else {
                                                                3
                                                            },
                                                            symbol: format!("+{}", s),
                                                        },
                                                    ),
                                                ),
                                                None,
                                                frame,
                                            ));
                                            u.left = ExpressionOperand::Value(0);
                                        }

                                        let op = &u.op;

                                        if let Some(ExpressionOperand::Symbol(s)) = &u.right {
                                            m_records.push(Frame::from(
                                                FrameInner::ObjectRecord(
                                                    ObjectRecord::Modification(
                                                        ModificationRecord {
                                                            start: locctr.unwrap() + 1,
                                                            length: if i.is_format4() {
                                                                5
                                                            } else {
                                                                3
                                                            },
                                                            symbol: format!(
                                                                "{}{}",
                                                                op.clone().unwrap(),
                                                                s
                                                            ),
                                                        },
                                                    ),
                                                ),
                                                None,
                                                frame,
                                            ));
                                            let val = match op.clone().unwrap() {
                                                expression::ExpressionOperator::Add
                                                | expression::ExpressionOperator::Subtract => 0,
                                                expression::ExpressionOperator::Multiply
                                                | expression::ExpressionOperator::Divide => 1,
                                            };
                                            u.right = Some(ExpressionOperand::Value(val));
                                        }
                                    }

                                    (expr.eval().unwrap(), true)
                                }
                            };

                            let source = &frame.sources()[0];
                            let is_number = if let FrameSource::Source(src, _) = source {
                                // find the next character of '#' is a number, then it is a number
                                let mut chars = src.chars();
                                let mut is_number = false;
                                while let Some(c) = chars.next() {
                                    if c == '#' {
                                        if let Some(c) = chars.next() {
                                            is_number = c.is_numeric();
                                        }
                                        break;
                                    }
                                }
                                is_number
                            } else {
                                false
                            };

                            let pc = locctr.unwrap() + length;
                            if i.is_format4() {
                                let nixbpe = if is_number {
                                    i.nixbpe
                                } else {
                                    i.nixbpe | 0b110000
                                };
                                data.push(i.opcode | (nixbpe & 0b110000) >> 4);
                                data.push(nixbpe << 4 | (operand >> 16) as u8);
                                data.push((operand >> 8) as u8);
                                data.push(operand as u8);

                                if start == 0 && (i.nixbpe & 0b010000) == 0 && !external {
                                    m_records.push(Frame::from(
                                        FrameInner::ObjectRecord(ObjectRecord::Modification(
                                            ModificationRecord {
                                                start: locctr.unwrap() + 1,
                                                length: 5,
                                                symbol: String::new(),
                                            },
                                        )),
                                        frame.label.clone(),
                                        frame,
                                    ));
                                }
                            } else if i.is_immediate() {
                                if is_number {
                                    let nixbpe = i.nixbpe;
                                    let disp = operand as i32;
                                    data.push(i.opcode | (nixbpe & 0b110000) >> 4);
                                    data.push(nixbpe << 4 | ((disp & 0b111100000000) >> 8) as u8);
                                    data.push(disp as u8);
                                } else {
                                    let nixbpe = i.nixbpe | 0b000010;
                                    let disp = operand as i32 - pc as i32;
                                    data.push(i.opcode | (nixbpe & 0b110000) >> 4);
                                    data.push(nixbpe << 4 | ((disp & 0b111100000000) >> 8) as u8);
                                    data.push(disp as u8);
                                }
                            // } else if let Some(base) = base {
                            //     let disp = operand as i32 - base as i32;
                            //     if disp >= 0 && disp <= 4095 {
                            //         let nixbpe = i.nixbpe | 0b110100;
                            //         data.push(i.opcode | (nixbpe & 0b110000) >> 4);
                            //         data.push(nixbpe << 4 | ((disp & 0b111100000000) >> 8) as u8);
                            //         data.push(disp as u8);
                            //     } else {
                            //         // calculate displacement
                            //         let mut reachable = false;
                            //         // try PC relative first
                            //         let disp = operand as i32 - pc as i32;
                            //         if disp >= -2048 && disp <= 2047 {
                            //             let nixbpe = i.nixbpe | 0b110010;
                            //             data.push(i.opcode | (nixbpe & 0b110000) >> 4);
                            //             data.push(nixbpe << 4 | ((disp & 0b111100000000) >> 8) as u8);
                            //             data.push(disp as u8);
                            //             reachable = true;
                            //         }

                            //         if !reachable {
                            //             return Err(format!("Operand out of range: {}", i.value));
                            //         }
                            //     }
                            } else if i.is_indirect() {
                                let nixbpe = i.nixbpe | 0b000010;
                                let disp = operand as i32 - pc as i32;
                                data.push(i.opcode | (nixbpe & 0b110000) >> 4);
                                data.push(nixbpe << 4 | ((disp & 0b111100000000) >> 8) as u8);
                                data.push(disp as u8);
                            } else {
                                // calculate displacement
                                let mut reachable = false;
                                // try PC relative first
                                let disp = operand as i32 - pc as i32;
                                if (-2048..=2047).contains(&disp) {
                                    let nixbpe = i.nixbpe | 0b110010;
                                    data.push(i.opcode | (nixbpe & 0b110000) >> 4);
                                    data.push(nixbpe << 4 | ((disp & 0b111100000000) >> 8) as u8);
                                    data.push(disp as u8);
                                    reachable = true;
                                } else if let Some(base) = base {
                                    // try BASE relative
                                    let disp = operand as i32 - base as i32;
                                    if (0..=4095).contains(&disp) {
                                        let nixbpe = i.nixbpe | 0b110100;
                                        data.push(i.opcode | (nixbpe & 0b110000) >> 4);
                                        data.push(
                                            nixbpe << 4 | ((disp & 0b111100000000) >> 8) as u8,
                                        );
                                        data.push(disp as u8);
                                        reachable = true;
                                    }
                                }

                                if !reachable {
                                    return Err(format!("Operand out of range: {}", i.value));
                                }
                            }

                            t_records.push(Frame::from(
                                FrameInner::ObjectRecord(ObjectRecord::Text(TextRecord {
                                    start: locctr.unwrap(),
                                    length,
                                    data,
                                })),
                                None,
                                frame,
                            ));
                        }
                    }
                }
            }
            FrameInner::Directive(ref d) => match d {
                directive::Directive::BYTE(b) => {
                    t_records.push(Frame::from(
                        FrameInner::ObjectRecord(ObjectRecord::Text(TextRecord {
                            start: locctr.unwrap(),
                            length: b.data.len() as u32,
                            data: b.data.clone(),
                        })),
                        None,
                        frame,
                    ));
                }
                directive::Directive::WORD(w) => {
                    let mut data = vec![];
                    let value = w.word.eval();
                    let mut value = match value {
                        Some(v) => v,
                        _ => {
                            // external reference involved
                            // create modification record
                            let mut expr = *w.word.clone();
                            if let Expression::Unsolved(ref mut u) = expr {
                                if let ExpressionOperand::Symbol(s) = &u.left {
                                    m_records.push(Frame::from(
                                        FrameInner::ObjectRecord(ObjectRecord::Modification(
                                            ModificationRecord {
                                                start: locctr.unwrap() + 1,
                                                length: 6,
                                                symbol: format!("+{}", s),
                                            },
                                        )),
                                        None,
                                        frame,
                                    ));
                                    u.left = ExpressionOperand::Value(0);
                                }

                                let op = &u.op;

                                if let Some(ExpressionOperand::Symbol(s)) = &u.right {
                                    m_records.push(Frame::from(
                                        FrameInner::ObjectRecord(ObjectRecord::Modification(
                                            ModificationRecord {
                                                start: locctr.unwrap() + 1,
                                                length: 6,
                                                symbol: format!("{}{}", op.clone().unwrap(), s),
                                            },
                                        )),
                                        None,
                                        frame,
                                    ));
                                    let val = match op.clone().unwrap() {
                                        expression::ExpressionOperator::Add
                                        | expression::ExpressionOperator::Subtract => 0,
                                        expression::ExpressionOperator::Multiply
                                        | expression::ExpressionOperator::Divide => 1,
                                    };
                                    u.right = Some(ExpressionOperand::Value(val));
                                }
                            }

                            expr.eval().unwrap()
                        }
                    };
                    for _ in 0..3 {
                        data.push((value & 0xFF) as u8);
                        value >>= 8;
                    }
                    t_records.push(Frame::from(
                        FrameInner::ObjectRecord(ObjectRecord::Text(TextRecord {
                            start: locctr.unwrap(),
                            length: 3,
                            data,
                        })),
                        None,
                        frame,
                    ));
                }
                directive::Directive::START(s) => {
                    h_record = Some(Frame::from(
                        FrameInner::ObjectRecord(ObjectRecord::Header(HeaderRecord {
                            name: s.name.clone(),
                            start: s.address,
                            length: 0,
                        })),
                        None,
                        frame,
                    ));
                }
                directive::Directive::END(e) => {
                    e_record = Some(Frame::from(
                        FrameInner::ObjectRecord(ObjectRecord::End(EndRecord {
                            start: e.first.eval().unwrap() as u32,
                        })),
                        None,
                        frame,
                    ));
                }
                _ => {}
            },
            _ => {}
        }

        // advance locctr
        if locctr.is_some() {
            let loc = locctr.unwrap();
            if let Some(size) = size {
                locctr = Some(loc + size as u32);
            } else {
                locctr = None;
            }
        }
    }

    if let Some(h) = h_record.as_mut() {
        if let FrameInner::ObjectRecord(ObjectRecord::Header(ref mut h)) = h.inner {
            h.length = locctr.unwrap() - start;
        }
    }

    let mut records = vec![];
    if let Some(h) = h_record {
        records.push(h);
    }
    records.append(&mut r_records.clone());
    records.append(&mut d_records.clone());
    records.append(&mut t_records.clone());
    records.append(&mut m_records.clone());
    if let Some(e) = e_record {
        records.push(e);
    }

    #[cfg(debug_assertions)]
    {
        for record in &records {
            println!("{:?}", record);
        }
    }

    Ok(records
        .into_iter()
        .map(|frame| {
            if let FrameInner::ObjectRecord(r) = frame.inner {
                r
            } else {
                panic!("Expected object record");
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::frameformer::{
        block::rearrange_blocks, literal::dump_literals, section::split_into_sections,
        symbol::resolve_symbols,
    };

    use super::*;
    use std::fs;

    #[test]
    fn parse_base() {
        let source = fs::read_to_string("../sample/base.asm").unwrap();
        let mut frames = Vec::<Frame>::new();
        for (i, line) in source.lines().enumerate() {
            let frame = Frame::from_source(line, i as u32 + 1).unwrap();
            if let Some(frame) = frame {
                frames.push(frame);
            }
        }

        let programs = split_into_sections(frames);
        let first = programs[0].clone();
        let frames = rearrange_blocks(first);
        let frames = dump_literals(frames);
        let frames = resolve_symbols(frames).unwrap();
        let records = translate_to_record(frames).unwrap();
        dbg!(&records);
        for record in &records {
            println!("{}", record);
        }

        assert_eq!(records.len(), 45);
    }
}
