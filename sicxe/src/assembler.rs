use std::collections::VecDeque;

use crate::frame::record::ObjectRecord;
use crate::frame::*;
use crate::frameformer::block::rearrange_blocks;
use crate::frameformer::literal::dump_literals;
use crate::frameformer::section::split_into_sections;
use crate::frameformer::symbol::resolve_symbols;
use crate::frameformer::translate::translate_to_record;

pub fn assemble(source: &str) -> Result<String, String> {
    #[cfg(debug_assertions)]
    dbg!(&source);

    let mut frames = Vec::<Frame>::new();
    for (i, line) in source.lines().enumerate() {
        let frame = Frame::from_source(line, i as u32 + 1)?;
        if let Some(frame) = frame {
            frames.push(frame);
        }
    }

    let mut result = String::new();

    let programs = split_into_sections(frames);
    for program in programs {
        let frames = rearrange_blocks(program);
        let frames = dump_literals(frames);
        let frames = resolve_symbols(frames)?;
        let records = translate_to_record(frames)?;

        #[cfg(debug_assertions)]
        for record in &records {
            println!("{}", record);
        }

        let optimized = optimize(records);
        result.push_str(&format!("{}\n", optimized));
    }

    Ok(result)
}

pub fn optimize(records: Vec<ObjectRecord>) -> String {
    let mut optimized = String::new();

    let headers = records
        .iter()
        .filter(|r| matches!(r, ObjectRecord::Header(_)));
    let defines = records
        .iter()
        .filter(|r| matches!(r, ObjectRecord::Define(_)));
    let refers = records
        .iter()
        .filter(|r| matches!(r, ObjectRecord::Refer(_)));
    let texts = records
        .iter()
        .filter(|r| matches!(r, ObjectRecord::Text(_)));
    let modifications = records
        .iter()
        .filter(|r| matches!(r, ObjectRecord::Modification(_)));
    let ends = records.iter().filter(|r| matches!(r, ObjectRecord::End(_)));

    // insert headers
    for header in headers {
        if let ObjectRecord::Header(header) = header {
            optimized.push_str(&format!(
                "H{: <6}{:06X}{:06X}\n",
                header.name, header.start, header.length
            ));
        }
    }

    // try to merge defines, up to 6 defines per line
    let mut defines = defines.collect::<Vec<_>>();
    while !defines.is_empty() {
        let mut line = "D".to_string();
        for _ in 0..6 {
            if let Some(ObjectRecord::Define(define)) = defines.pop() {
                line.push_str(&format!("{: <6}{:06X}", define.name, define.value));
            }
        }
        optimized.push_str(&format!("{}\n", line));
    }

    // merge refers, up to 12 refers per line
    let mut refers = refers.collect::<Vec<_>>();
    while !refers.is_empty() {
        let mut line = "R".to_string();
        for _ in 0..12 {
            if let Some(ObjectRecord::Refer(refer)) = refers.pop() {
                line.push_str(&format!("{: <6}", refer.name));
            }
        }
        optimized.push_str(&format!("{}\n", line));
    }

    // merge texts, up to 60 bytes per line
    let texts = texts.collect::<Vec<_>>();
    let mut current_line = String::new();
    let mut current_start = 0;
    for r in texts {
        if let ObjectRecord::Text(r) = r {
            if current_start + current_line.len() / 2 != r.start as usize {
                if !current_line.is_empty() {
                    let len = current_line.len() / 2;
                    optimized.push_str(&format!(
                        "T{:06X}{:02X}{}\n",
                        current_start, len, current_line
                    ));
                }
                current_line = String::new();
                current_start = r.start as usize;
            }

            let mut data = VecDeque::from(r.data.clone());
            while !data.is_empty() {
                let max: i32 = (60 - current_line.len() as i32) / 2;
                let wrote = max.min(data.len() as i32);

                if wrote > 0 {
                    for _ in 0..wrote {
                        if let Some(byte) = data.pop_front() {
                            current_line.push_str(&format!("{:02X}", byte));
                        }
                    }
                }

                #[cfg(debug_assertions)]
                if current_line.len() > 60 {
                    panic!("Line length exceeds 60 bytes, max = {max}, wrote = {wrote}");
                }

                if current_line.len() == 60 {
                    optimized.push_str(&format!(
                        "T{:06X}{:02X}{}\n",
                        current_start,
                        current_line.len() / 2,
                        current_line
                    ));
                    current_line = String::new();
                    current_start += 30;
                }
            }
        }
    }

    if !current_line.is_empty() {
        optimized.push_str(&format!(
            "T{:06X}{:02X}{}\n",
            current_start,
            current_line.len() / 2,
            current_line
        ));
    }

    // insert modifications
    for modification in modifications {
        if let ObjectRecord::Modification(modification) = modification {
            optimized.push_str(&format!(
                "M{:06X}{:02X}{}\n",
                modification.start, modification.length, modification.symbol
            ));
        }
    }

    // insert ends
    for end in ends {
        if let ObjectRecord::End(end) = end {
            optimized.push_str(&format!("E{:06X}\n", end.start));
        }
    }

    optimized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_base() {
        let source = fs::read_to_string("../sample/base.asm").unwrap();
        let result = assemble(&source).unwrap();
        println!("{}", result);

        let fixture = fs::read_to_string("../sample/base.obj").unwrap();

        // trim
        let result = result.trim();
        let fixture = fixture.trim();

        // compare line by line
        let result = result.lines().collect::<Vec<_>>();
        let fixture = fixture.lines().collect::<Vec<_>>();
        assert_eq!(result.len(), fixture.len());
        for (i, (result, fixture)) in result.iter().zip(fixture.iter()).enumerate() {
            assert_eq!(result, fixture, "line {}", i + 1);
        }
    }

    // #[test]
    // fn parse_code2() {
    //     let source = fs::read_to_string("../sample/code2.asm").unwrap();
    //     let result = assemble(&source).unwrap();
    //     println!("{}", result);

    //     let fixture = fs::read_to_string("../sample/code2.obj").unwrap();

    //     // trim
    //     let result = result.trim();
    //     let fixture = fixture.trim();

    //     // compare line by line
    //     let result = result.lines().collect::<Vec<_>>();
    //     let fixture = fixture.lines().collect::<Vec<_>>();
    //     assert_eq!(result.len(), fixture.len());
    //     for (i, (result, fixture)) in result.iter().zip(fixture.iter()).enumerate() {
    //         assert_eq!(result, fixture, "line {}", i + 1);
    //     }
    // }

    #[test]
    fn parse_code3() {
        let source = fs::read_to_string("../sample/code3.asm").unwrap();
        let result = assemble(&source).unwrap();
        println!("{}", result);

        let fixture = fs::read_to_string("../sample/code3.obj").unwrap();

        // trim
        let result = result.trim();
        let fixture = fixture.trim();

        // compare line by line
        let result = result.lines().collect::<Vec<_>>();
        let fixture = fixture.lines().collect::<Vec<_>>();
        assert_eq!(result.len(), fixture.len());
        for (i, (result, fixture)) in result.iter().zip(fixture.iter()).enumerate() {
            assert_eq!(result, fixture, "line {}", i + 1);
        }
    }
}
