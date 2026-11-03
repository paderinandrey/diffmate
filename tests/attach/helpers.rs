use std::io::{BufRead, BufReader, Read};
use std::process::{Child, ChildStderr, Output};

pub struct Waiting {
    pub child: Child,
    pub lines: String,
    stderr: BufReader<ChildStderr>,
}

impl Waiting {
    pub fn new(mut child: Child) -> Self {
        let mut stderr = BufReader::new(child.stderr.take().unwrap());
        let mut lines = String::new();
        loop {
            let mut line = String::new();
            if stderr.read_line(&mut line).unwrap() == 0 {
                break;
            }
            lines.push_str(&line);
            if line.starts_with("waiting:") {
                break;
            }
        }
        Self {
            child,
            lines,
            stderr,
        }
    }

    pub fn finish(mut self) -> Output {
        self.stderr.read_to_string(&mut self.lines).unwrap();
        let mut output = self.child.wait_with_output().unwrap();
        output.stderr = self.lines.into_bytes();
        output
    }
}

pub fn succeeded(output: &Output) {
    assert!(output.status.success(), "{output:?}");
}
