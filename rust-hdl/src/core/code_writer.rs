pub struct CodeWriter {
    lines: Vec<(usize, String)>,
    // Each line consists of an indent level and a string of text
    indent: usize,
    // CodeWriter provides a buffer for assembling lines too
    buffer: String,
}

impl CodeWriter {
    pub fn new() -> CodeWriter {
        CodeWriter {
            lines: vec![],
            indent: 0,
            buffer: String::new(),
        }
    }

    pub fn push(&mut self) {
        self.indent += 1
    }

    pub fn pop(&mut self) {
        self.indent -= 1
    }

    pub fn next(&mut self) {
        self.add_line("")
    }

    pub fn add_line<S: AsRef<str>>(&mut self, val: S) {
        self.lines.push((self.indent, String::from(val.as_ref())))
    }

    pub fn add<S: AsRef<str>>(&mut self, val: S) {
        let temp = String::from(val.as_ref());
        let pieces = temp.split_terminator("\n");
        for piece in pieces {
            self.add_line(&piece)
        }
    }

    pub fn write<S: AsRef<str>>(&mut self, val: S) {
        self.buffer += val.as_ref()
    }

    pub fn writeln<S: AsRef<str>>(&mut self, val: S) {
        self.write(val);
        self.flush();
    }

    pub fn flush(&mut self) {
        let line = self.buffer.clone();
        self.add(&line);
        self.buffer.clear()
    }
}

impl ToString for CodeWriter {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        for (indent, line) in &self.lines {
            buf += &"    ".repeat(*indent);
            buf += line;
            buf += &"\n"
        }
        buf
    }
}
