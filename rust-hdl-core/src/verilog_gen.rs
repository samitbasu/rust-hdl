use crate::code_writer::CodeWriter;

struct VerilogCodeGenerator {
    io: CodeWriter,
}

impl VerilogCodeGenerator {
    pub fn new() -> VerilogCodeGenerator {
        Self {
            io: CodeWriter::new(),
        }
    }
}
