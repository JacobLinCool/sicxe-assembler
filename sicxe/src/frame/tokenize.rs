pub const LITERAL_NOT_CLOSED: &str = "Literal not closed";

pub fn tokenize(line: &str) -> Result<Vec<String>, String> {
    let mut matched: Vec<String> = vec![];
    let mut current = String::new();

    let mut prev = '\0';
    let mut literal = false;

    for c in line.chars() {
        if !literal {
            if c.is_whitespace() {
                if !current.is_empty() {
                    matched.push(current.clone());
                    current.clear();
                }

                prev = c;
                continue;
            }

            if c == '.' || c == '\n' || c == '\r' {
                break;
            }

            if c == '\'' {
                literal = true;
            }

            current.push(c);
        } else {
            if c == '\'' && prev != '\\' {
                literal = false;
            }

            current.push(c);
        }

        prev = c;
    }

    if literal {
        return Err(LITERAL_NOT_CLOSED.to_string());
    }

    if !current.is_empty() {
        matched.push(current.clone());
    }

    #[cfg(debug_assertions)]
    dbg!(line, &matched);

    Ok(matched)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_mnem_with_comment() {
        let res = tokenize("PROG START .comment here").unwrap();
        assert_eq!(res, vec!["PROG".to_string(), "START".to_string()]);
    }

    #[test]
    fn label_mnem_operand_with_comment() {
        let res = tokenize("VAR RESW 10 .comment here").unwrap();
        assert_eq!(
            res,
            vec!["VAR".to_string(), "RESW".to_string(), "10".to_string()]
        );
    }

    #[test]
    fn label_mnem_operand() {
        let res = tokenize("VAR RESW 10").unwrap();
        assert_eq!(
            res,
            vec!["VAR".to_string(), "RESW".to_string(), "10".to_string()]
        );
    }

    #[test]
    fn label_mnem() {
        let res = tokenize("PROG START").unwrap();
        assert_eq!(res, vec!["PROG".to_string(), "START".to_string()]);
    }

    #[test]
    fn mnem_operand() {
        let res = tokenize("    LDA   #0").unwrap();
        assert_eq!(res, vec!["LDA".to_string(), "#0".to_string()]);
    }

    #[test]
    fn mnem() {
        let res = tokenize("    END").unwrap();
        assert_eq!(res, vec!["END".to_string()]);
    }

    #[test]
    fn literal_operand() {
        let res = tokenize("MYVAR   WORD    C'IT\\'S A STRING'").unwrap();
        assert_eq!(
            res,
            vec![
                "MYVAR".to_string(),
                "WORD".to_string(),
                "C'IT\\'S A STRING'".to_string()
            ]
        );
    }

    #[test]
    fn too_many_token() {
        let res = tokenize("ABC DEF GHI JKL").unwrap();
        assert_eq!(
            res,
            vec![
                "ABC".to_string(),
                "DEF".to_string(),
                "GHI".to_string(),
                "JKL".to_string()
            ]
        )
    }

    #[test]
    fn literal_not_closed() {
        let err = tokenize("ABC C'IT\\'S A STRING").unwrap_err();
        assert_eq!(err, LITERAL_NOT_CLOSED.to_string())
    }

    #[test]
    fn comment_only() {
        let res = tokenize("    .comment here 123").unwrap();
        assert_eq!(res, Vec::<String>::new())
    }
}
