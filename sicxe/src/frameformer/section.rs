use crate::frame::{directive::*, expression::parse, *};

pub fn split_into_sections(program: Vec<Frame>) -> Vec<Vec<Frame>> {
    let mut main_program = Vec::<Frame>::new();
    let mut subroutines = Vec::<Vec<Frame>>::new();
    let mut is_subroutine = false;

    for frame in program {
        match frame.inner {
            FrameInner::Directive(ref d) => match d {
                directive::Directive::END(_) => {
                    main_program.push(frame);
                }
                directive::Directive::CSECT(s) => {
                    if !is_subroutine {
                        is_subroutine = true;
                    }

                    let subroutine_start = Frame::from(
                        FrameInner::Directive(Directive::START(START {
                            name: s.name.clone(),
                            address: 0,
                        })),
                        Some(s.name.clone()),
                        &frame,
                    );

                    let subroutine = vec![subroutine_start];
                    subroutines.push(subroutine);
                }
                _ => {
                    if is_subroutine {
                        let last_subroutine = subroutines.last_mut().unwrap();
                        last_subroutine.push(frame);
                    } else {
                        main_program.push(frame);
                    }
                }
            },
            FrameInner::Instruction(_) => {
                if is_subroutine {
                    let last_subroutine = subroutines.last_mut().unwrap();
                    last_subroutine.push(frame);
                } else {
                    main_program.push(frame);
                }
            }
            FrameInner::ObjectRecord(_) => panic!("Object record is not allowed in this stage"),
        }
    }

    let mut programs = Vec::<Vec<Frame>>::new();
    programs.push(main_program);
    for mut subroutine in subroutines {
        if subroutine.len() > 1 {
            let subroutine_end = Frame::from(
                FrameInner::Directive(Directive::END(END {
                    first: Box::new(parse(subroutine.first().unwrap().label.as_deref().unwrap()).unwrap()),
                })),
                None,
                subroutine.first().unwrap(),
            );

            subroutine.push(subroutine_end);
            programs.push(subroutine);
        }
    }

    programs
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

        let programs = split_into_sections(frames);
        dbg!(&programs);

        assert_eq!(programs.len(), 1);
        assert_eq!(programs[0].len(), 46);
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

        let programs = split_into_sections(frames);
        dbg!(&programs);

        assert_eq!(programs.len(), 3);
        assert_eq!(programs[0].len(), 23);
        assert_eq!(programs[1].len(), 19);
        assert_eq!(programs[2].len(), 12);
    }
}
