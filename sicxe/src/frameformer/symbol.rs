use std::collections::HashMap;

use crate::frame::directive::*;
use crate::frame::expression::*;
use crate::frame::record::*;
use crate::frame::*;

pub fn resolve_symbols(program: Vec<Frame>) -> Result<Vec<Frame>, String> {
    let mut program = program;

    // build symbol table
    let mut symtab = HashMap::<String, Box<Expression>>::new();

    // insert registers
    let register_a = Expression::Resolved(0);
    let register_x = Expression::Resolved(1);
    let register_l = Expression::Resolved(2);
    let register_b = Expression::Resolved(3);
    let register_s = Expression::Resolved(4);
    let register_t = Expression::Resolved(5);
    let register_f = Expression::Resolved(6);
    let register_pc = Expression::Resolved(8);
    let register_sw = Expression::Resolved(9);
    symtab.insert("A".to_string(), Box::new(register_a));
    symtab.insert("X".to_string(), Box::new(register_x));
    symtab.insert("L".to_string(), Box::new(register_l));
    symtab.insert("B".to_string(), Box::new(register_b));
    symtab.insert("S".to_string(), Box::new(register_s));
    symtab.insert("T".to_string(), Box::new(register_t));
    symtab.insert("F".to_string(), Box::new(register_f));
    symtab.insert("PC".to_string(), Box::new(register_pc));
    symtab.insert("SW".to_string(), Box::new(register_sw));

    // insert external references
    let extrefs = program
        .iter()
        .filter(|frame| {
            matches!(
                frame.inner,
                FrameInner::Directive(directive::Directive::EXTREF(_))
            )
        })
        .cloned()
        .collect::<Vec<Frame>>();
    for frame in extrefs {
        if let FrameInner::Directive(directive::Directive::EXTREF(ref extrefs)) = frame.inner {
            for extref in extrefs.names.clone() {
                let expr = Expression::Unsolved(UnsolvedExpression {
                    left: ExpressionOperand::Symbol("<EXTERNAL>".to_string()),
                    op: None,
                    right: None,
                });
                symtab.insert(extref, Box::new(expr));
            }
        }
    }

    // locctr maybe untrustable in some cases
    let mut start = 0;
    let mut locctr = Some(0);
    for frame in &mut program {
        let size = frame.size();
        let label = frame.label.clone();

        // resolve ORG expression
        if let FrameInner::Directive(Directive::ORG(ref mut org)) = frame.inner {
            if let Expression::Unsolved(ref mut e) = *org.address {
                if e.left == ExpressionOperand::Locctr {
                    e.left = ExpressionOperand::Value(locctr.unwrap() as i32);
                }
            }
            org.address = evaluate(org.address.clone(), &mut symtab);
            org.address.eval_and_update();
        }

        // set locctr to the address of START or ORG
        if let FrameInner::Directive(ref d) = frame.inner {
            if let directive::Directive::START(s) = d {
                locctr = Some(s.address);
                start = s.address;
            }
            if let directive::Directive::ORG(o) = d {
                locctr = Some(o.address.eval().unwrap() as u32);
            }
        }

        // resolve locctr
        if let FrameInner::Directive(directive::Directive::EQU(EQU { ref mut value })) = frame.inner
        {
            if let Expression::Unsolved(ref mut e) = **value {
                if e.left == ExpressionOperand::Locctr {
                    e.left = ExpressionOperand::Value(locctr.unwrap() as i32);
                }
            }
        }

        // insert label into symbol table
        if let FrameInner::Directive(directive::Directive::EQU(EQU { ref value })) = frame.inner {
            symtab.insert(label.unwrap(), value.clone());
        } else if let Some(label) = label {
            if let Some(locctr) = locctr {
                let expr = Expression::Resolved(locctr as i32);
                symtab.insert(label, Box::new(expr));
            }
        }

        // try to resolve expression in the symbol table
        resolve(&mut symtab);

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

    #[cfg(debug_assertions)]
    dbg!(&symtab);
    // #[cfg(debug_assertions)]
    // dbg!(&program);

    // replace EXTDEF with D record
    let mut extdefs = Vec::<Frame>::new();
    let mut i = 0;
    while i < program.len() {
        let frame = &program[i];
        if let FrameInner::Directive(directive::Directive::EXTDEF(EXTDEF { ref names })) =
            frame.inner
        {
            for name in names.clone() {
                let value = symtab.get(&name).unwrap().eval().unwrap() as u32 - start;
                let frame = Frame::from(
                    FrameInner::ObjectRecord(ObjectRecord::Define(DefineRecord { name, value })),
                    None,
                    frame,
                );
                extdefs.push(frame);
            }
            program.remove(i);
        } else {
            i += 1;
        }
    }
    // #[cfg(debug_assertions)]
    // dbg!(&extdefs);

    // replace EXTREF with R record
    let mut extrefs = Vec::<Frame>::new();
    let mut i = 0;
    while i < program.len() {
        let frame = &program[i];
        if let FrameInner::Directive(directive::Directive::EXTREF(EXTREF { ref names })) =
            frame.inner
        {
            for name in names.clone() {
                let frame = Frame::from(
                    FrameInner::ObjectRecord(ObjectRecord::Refer(ReferRecord { name })),
                    None,
                    frame,
                );
                extrefs.push(frame);
            }
            program.remove(i);
        } else {
            i += 1;
        }
    }
    // #[cfg(debug_assertions)]
    // dbg!(&extrefs);

    // replace symbols with values
    let mut i = 0;
    while i < program.len() {
        let frame = &mut program[i];
        match &mut frame.inner {
            FrameInner::Instruction(i) => match i {
                instruction::Instruction::Format2(ref mut i) => {
                    i.register1 = evaluate(i.register1.clone(), &mut symtab);
                    i.register2 = evaluate(i.register2.clone(), &mut symtab);
                }
                instruction::Instruction::Format34(i) => {
                    i.value = evaluate(i.value.clone(), &mut symtab);
                }
                _ => {}
            },
            FrameInner::Directive(d) => match d {
                Directive::END(ref mut end) => {
                    end.first = evaluate(end.first.clone(), &mut symtab);
                }
                Directive::WORD(ref mut word) => {
                    word.word = evaluate(word.word.clone(), &mut symtab);
                }
                Directive::RESB(ref mut resb) => {
                    resb.bytes = evaluate(resb.bytes.clone(), &mut symtab);
                }
                Directive::RESW(ref mut resw) => {
                    resw.words = evaluate(resw.words.clone(), &mut symtab);
                }
                Directive::ORG(ref mut org) => {
                    org.address = evaluate(org.address.clone(), &mut symtab);
                }
                Directive::BASE(ref mut base) => {
                    base.address = evaluate(base.address.clone(), &mut symtab);
                }
                _ => {}
            },
            _ => {}
        };

        if let FrameInner::Directive(directive::Directive::EQU(_)) = frame.inner {
            program.remove(i);
        } else {
            i += 1;
        }
    }

    // insert D and R records after start
    program.splice(1..1, extrefs);
    program.splice(1..1, extdefs);

    // #[cfg(debug_assertions)]
    // dbg!(&program);

    Ok(program)
}

fn resolve(symtab: &mut HashMap<String, Box<Expression>>) {
    let mut updated = true;
    while updated {
        updated = false;
        let cloned = symtab.clone();
        for (_key, value) in symtab.iter_mut() {
            if let Expression::Unsolved(ref mut expr) = **value {
                if let ExpressionOperand::Symbol(ref symbol) = expr.left {
                    if let Some(left) = cloned.get(symbol) {
                        if let Expression::Resolved(left) = **left {
                            expr.left = ExpressionOperand::Value(left);
                            updated = true;
                        }
                    }
                }

                if let Some(ExpressionOperand::Symbol(ref symbol)) = expr.right {
                    if let Some(right) = cloned.get(symbol) {
                        if let Expression::Resolved(right) = **right {
                            expr.right = Some(ExpressionOperand::Value(right));
                            updated = true;
                        }
                    }
                }
            }

            value.eval_and_update();
        }
    }
}

fn evaluate(
    mut expr: Box<Expression>,
    symtab: &mut HashMap<String, Box<Expression>>,
) -> Box<Expression> {
    if let Expression::Unsolved(ref mut expr) = *expr {
        if let ExpressionOperand::Symbol(ref symbol) = expr.left {
            if let Some(value) = symtab.get(symbol) {
                if let Expression::Resolved(value) = **value {
                    expr.left = ExpressionOperand::Value(value);
                }
            }
        }

        if let Some(ref mut right) = expr.right {
            if let ExpressionOperand::Symbol(ref symbol) = right {
                if let Some(value) = symtab.get(symbol) {
                    if let Expression::Resolved(value) = **value {
                        *right = ExpressionOperand::Value(value);
                    }
                }
            }
        }
    }
    expr.eval_and_update();
    expr
}

#[cfg(test)]
mod tests {
    use crate::frameformer::{
        block::rearrange_blocks, literal::dump_literals, section::split_into_sections,
    };

    use super::*;
    use std::fs;

    #[test]
    fn parse_code2() {
        let source = fs::read_to_string("../sample/code3.asm").unwrap();
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
        dbg!(&frames);

        assert_eq!(frames.len(), 24);
    }

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
        dbg!(&frames);

        assert_eq!(frames.len(), 46);
    }
}
