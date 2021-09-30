#[derive(Clone, Debug)]
pub struct Designator {
    pub kind: DesignatorKind,
    pub index: Option<u64>,
}

#[derive(Clone, Copy, Debug)]
pub enum DesignatorKind {
    Resistor,
    Capacitor,
    Inductor,
    Diode,
    VoltageRegulator,
    Connector,
    IntegratedCircuit,
    MountingHole,
    Switch,
}
