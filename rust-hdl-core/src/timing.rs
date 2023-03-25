#[derive(Clone, Debug)]
pub struct TimingInfo {
    pub name: String,
    pub clock: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}
