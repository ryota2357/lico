use std::path::PathBuf;

pub fn start(file: &PathBuf) {
    let buf = std::fs::read_to_string(file).unwrap();
    let buf_str = buf.as_str();

    let (tokens, err) = lexer::parse(buf_str);
    if !err.is_empty() {
        for e in err {
            println!("{e:?}");
        }
        return;
    }

    let (tree, err) = parser::parse(&tokens);
    if !err.is_empty() {
        for e in err {
            println!("{e:?}");
        }
        return;
    }

    let code = match compiler::compile(&tree) {
        Ok(x) => x,
        Err(e) => {
            println!("Compilation error: {:?}", e);
            let (start, end) = match get_line_column_range(buf_str, e.span.to_range()) {
                Some(x) => x,
                None => {
                    println!("Invalid span");
                    return;
                }
            };
            println!("Positon: {}:{} ~ {}:{}", start.0, start.1, end.0, end.1);
            return;
        }
    };

    let mut runtime = vm::runtime::Runtime::new();
    vm::execute(&code, &mut runtime).unwrap();
}

fn get_line_column_range(
    source: &str,
    span: std::ops::Range<u32>,
) -> Option<((u32, u32), (u32, u32))> {
    let source_len = {
        let len = source.chars().count();
        if len > u32::MAX as usize {
            return None;
        }
        len as u32
    };

    // Return None if the start and end positions of the range are outside the document range
    if span.start >= source_len || span.end > source_len || span.start > span.end {
        return None;
    }

    let line_start_offsets = {
        let mut offsets = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                offsets.push(i + 1);
            }
        }
        offsets
    };

    let calc_line_column = |offset: u32| {
        let line = line_start_offsets
            .iter()
            .position(|&line_start_offset| line_start_offset as u32 > offset)
            .unwrap_or(line_start_offsets.len() - 1);
        let column = offset - line_start_offsets[line - 1] as u32;
        (line as u32, column)
    };

    let start = calc_line_column(span.start);
    let end = calc_line_column(span.end);
    Some((start, end))
}
