pub fn unescape_string(input: &str) -> String {
    let mut chars = input.chars().peekable();
    let mut output = String::with_capacity(input.len());
    while let Some(c) = chars.next() {
        let new_char = if c == '\\' {
            let next_char = chars.next().expect("Strings cannot end in a back-slash");
            match next_char {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '0' => '\0',
                _ => next_char,
            }
        } else {
            c
        };
        output.push(new_char);
    }

    output
}

pub fn escape_string(input: &str) -> String {
    let chars = input.chars();

    let mut output = String::with_capacity(input.len());
    for c in chars {
        match c {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\0' => output.push_str("\\0"),
            '\\' => output.push_str("\\\\"),
            _ => output.push(c),
        }
    }

    output
}

pub fn is_alpha(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '_')
}

pub fn is_digit(ch: char) -> bool {
    matches!(ch, '0'..='9')
}

pub fn is_alphanumeric(ch: char) -> bool {
    is_alpha(ch) || is_digit(ch)
}
