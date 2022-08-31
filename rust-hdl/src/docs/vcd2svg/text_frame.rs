// Inspired heavily by dwfv

use crate::docs::vcd2svg::symbols;

#[derive(Debug, Clone, Default)]
pub struct TextFrame {
    buffers: Vec<Vec<char>>,
    columns: usize,
}

impl TextFrame {
    pub fn new(columns: usize) -> Self {
        Self {
            buffers: vec![],
            columns,
        }
    }
    pub fn row(&mut self, row: usize) -> &mut [char] {
        if row < self.buffers.len() {
            &mut self.buffers[row]
        } else {
            while self.buffers.len() <= row {
                self.buffers.push(vec![' '; self.columns])
            }
            &mut self.buffers[row]
        }
    }
    pub fn write(&mut self, row: usize, col: usize, msg: &str) {
        for (ndx, char) in msg.chars().enumerate() {
            if ndx + col < self.columns {
                self.row(row)[ndx + col] = char;
            }
        }
    }
    pub fn put(&mut self, row: usize, col: usize, x: char) {
        if col < self.columns {
            self.row(row)[col] = x;
        }
    }
}

impl ToString for TextFrame {
    fn to_string(&self) -> String {
        self.buffers
            .iter()
            .map(|x| x.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[test]
fn test_frame_draw_stuff() {
    let mut x = TextFrame::new(16);
    x.write(0, 0, "hello world!");
    assert_eq!(x.to_string(), "hello world!    ");
    x.write(1, 0, "foo!");
    assert_eq!(x.to_string(), "hello world!    \nfoo!            ");
    x.put(1, 1, symbols::HORIZONTAL);
    assert_eq!(x.to_string(), "hello world!    \nf─o!            ");
}
