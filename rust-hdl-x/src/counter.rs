use crate::synchronous::{NoTrace, Synchronous, Tracer};

struct Counter {}

impl Synchronous for Counter {
    type State = u32;
    type Input = bool;
    type Output = u32;
    fn update(&self, t: impl Tracer, q: u32, enable: bool) -> (u32, u32) {
        let module = t.module("counter");
        let d = if enable { q + 1 } else { q };
        (q, d)
    }

    fn default_output(&self) -> Self::Output {
        0
    }
}

// Count to 1e9
#[test]
fn test_count_to_1e9() {
    let mut state = 0_u32;
    let mut output = 0_u32;
    let now = std::time::Instant::now();
    let counter = Counter {};
    let tracer = NoTrace {};
    for cycle in 0..1_000_000_000 {
        (output, state) = counter.update(&tracer, state, cycle % 2 == 0);
    }
    println!(
        "Final state: {state:?}, elapsed time {}",
        now.elapsed().as_millis()
    );
}
