#[derive(Clone, Debug, PartialEq, Default)]
pub struct NamedPath {
    path: Vec<String>,
    namespace: Vec<String>,
}

impl NamedPath {
    pub fn push<T: ToString>(&mut self, x: T) {
        self.path.push(x.to_string());
    }

    pub fn pop(&mut self) {
        self.path.pop();
    }

    pub fn parent(&self) -> String {
        self.path[0..self.path.len() - 1]
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("$")
    }

    pub fn last(&self) -> String {
        self.path.last().unwrap().clone()
    }

    pub fn reset(&mut self) {
        self.path.clear();
    }

    pub fn flat(&self, sep: &str) -> String {
        self.path.join(sep)
    }

    pub fn len(&self) -> usize {
        self.path.len()
    }
}

impl ToString for NamedPath {
    fn to_string(&self) -> String {
        self.path.join("$")
    }
}
