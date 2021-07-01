#[derive(Clone, Debug, PartialEq, Default)]
pub struct NamedPath {
    path: Vec<String>
}

impl NamedPath {
    pub fn push<T: ToString>(&mut self, x: T) {
        self.path.push(x.to_string());
    }

    pub fn pop(&mut self) {
        self.path.pop();
    }
}

impl ToString for NamedPath {
    fn to_string(&self) -> String {
        self.path.join("::")
    }
}
