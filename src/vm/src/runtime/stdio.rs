use std::io::{Stderr, Stdin, Stdout, Write};

#[derive(Debug, Default)]
pub struct Stdio {
    pub stdin: Option<Stdin>,
    pub stdout: Option<Stdout>,
    pub stderr: Option<Stderr>,
}

impl Stdio {
    #[inline]
    pub const fn new() -> Self {
        Self {
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn write(&mut self, str: impl AsRef<str>) {
        self.stdout
            .get_or_insert_with(std::io::stdout)
            .write_all(str.as_ref().as_bytes())
            .expect("Failed to write to stdout.");
    }

    pub fn flush(&mut self) {
        self.stdout
            .get_or_insert_with(std::io::stdout)
            .flush()
            .expect("Failed to flush stdout.");
    }

    pub fn write_err(&mut self, str: impl AsRef<str>) {
        self.stderr
            .get_or_insert_with(std::io::stderr)
            .write_all(str.as_ref().as_bytes())
            .expect("Failed to write to stderr.");
    }

    pub fn flush_err(&mut self) {
        self.stderr
            .get_or_insert_with(std::io::stderr)
            .flush()
            .expect("Failed to flush stderr.");
    }

    pub fn read_line(&mut self) -> String {
        let mut buf = String::new();
        self.stdin
            .get_or_insert_with(std::io::stdin)
            .read_line(&mut buf)
            .expect("Failed to read from stdin.");
        buf
    }
}
