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

    let code = compiler::compile(&tree).unwrap();

    let mut runtime = vm::runtime::Runtime::new(std::io::stdout());
    vm::execute(&code, &mut runtime).unwrap();
}
