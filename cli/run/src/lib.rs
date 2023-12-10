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

    let (tree, err) = parser::parse(&tokens, buf_str.len()..buf_str.len());
    if !err.is_empty() {
        for e in err {
            println!("{e:?}");
        }
        return;
    }
    let Some(tree) = tree else {
        println!("No tree");
        return;
    };

    let code = match compiler::compile(&tree) {
        Ok(x) => x,
        Err(e) => {
            println!("Compilation error: {:?}", e);
            let (start, end) = match get_line_column_range(buf_str, e.span) {
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
    span: std::ops::Range<usize>,
) -> Option<((usize, usize), (usize, usize))> {
    let source_len = source.chars().count();

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

    let calc_line_column = |offset: usize| {
        let line = line_start_offsets
            .iter()
            .position(|&line_start_offset| line_start_offset > offset)
            .unwrap_or(line_start_offsets.len() - 1);
        let column = offset - line_start_offsets[line - 1];
        (line, column)
    };

    let start = calc_line_column(span.start);
    let end = calc_line_column(span.end);
    Some((start, end))
}
