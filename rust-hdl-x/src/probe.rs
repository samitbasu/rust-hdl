struct Thing {
    pub thing_1: Thing1,
    pub thing_2: Thing2,
}

struct ThingState {}

struct ThingInput {
    trigger: bool,
}

struct ThingOutput {
    enable: bool,
}

struct TraceId(usize);

struct ThingTrace {
    id_i: TraceId,
    id_o: TraceId,
    id_d: TraceId,
    id_q: TraceId,
    id_thing_1: Thing1Trace,
    id_thing_2: Thing2Trace,
}

struct Thing1 {
    pub counter: u16,
}

struct Thing1Input {
    pub signal: bool,
}

struct Thing1Output {}

struct Thing1Trace {}

struct Thing2 {
    pub level: bool,
}

struct Thing2Output {
    pub count: u16,
}

struct Thing2Trace {}
struct Thing2Input {}

struct Thing2State {}
struct Thing1State {}

impl Test for Thing {
    type Input = ThingInput;
    type Output = ThingOutput;
    type State = ThingState;
    type Trace = ThingTrace;

    fn setup(name: &'static str, tracer: impl Tracer) -> Self::Trace {
        tracer.enter_module("thing1");
        let id_i = Thing1Input::allocate("i", tracer);
        let id_o = Thing1Output::allocate("o", tracer);
        let id_d = Thing1State::allocate("d", tracer);
        let id_q = Thing1State::allocate("q", tracer);
        let id_thing_1 = <Thing1 as Test>::setup("thing_1", tracer);
        let id_thing_2 = <Thing2 as Test>::setup("thing_2", tracer);
        tracer.exit_module();
        ThingTrace {
            id_i,
            id_o,
            id_d,
            id_q,
            id_thing_1,
            id_thing_2,
        }
    }

    fn update(
        &self,
        q: Self::State,
        i: Self::Input,
        t: impl Tracer,
    ) -> (Self::Output, Self::State) {
        let (o, d) = self.thing_1.update(q.d, i, t);
        let (o, q) = self.thing_2.update(q.q, i, t);
        (o, ThingState { d, q })
    }
}

trait Test {
    type Input;
    type Output;
    type State;
    const ID: &'static str;

    fn setup(tracer: impl Tracer);
    fn call_update(
        &self,
        q: Self::State,
        i: Self::Input,
        t: impl Tracer,
    ) -> (Self::Output, Self::State);
    fn update(
        &self,
        q: Self::State,
        i: Self::Input,
        t: impl Tracer,
    ) -> (Self::Output, Self::State) {
    }
    fn default_output(&self) -> Self::Output;
}

trait Tracer {}

/*
 One option is to "opt in" to tracing, but with a single simple type.
*/

struct Thing4 {
    other_stuff: u32,
    thing: Thing,
    trace_id: TraceId, // <-- this is the only thing that needs to be added
}

impl Test for Thing4 {
    type Input = ThingInput;
    type Output = ThingOutput;
    type State = ThingState;
    const ID: &'static str = "thing4";

    fn setup(tracer: impl Tracer) {
        tracer.enter_module(Self::ID);
        // Tell the tracer about our 3 signal types
        Self::Input::allocate_input(Self::ID, tracer);
        Self::Output::allocate_output(Self::ID, tracer);
        Self::State::allocate_state(Self::ID, tracer);
        // Repeat for sub things
        Thing::setup(tracer);
    }
    fn call_update(
        &self,
        q: Self::State,
        i: Self::Input,
        t: impl Tracer,
    ) -> (Self::Output, Self::State) {
        t.enter_scope(Self::ID);
        t.record_input(Self::ID, i);
        t.record_state(Self::ID, q);
        let (o, d) = self.update(q, i, t);
        t.record_output(Self::ID, o);
        t.record_new_state(Self::ID, d);
        t.exit_scope(Self::ID);
        (o, d)
    }
    fn update(
        &self,
        q: Self::State,
        i: Self::Input,
        t: impl Tracer,
    ) -> (Self::Output, Self::State) {
        let (o, d) = self.thing_1.update(q.d, i, t);
        let (o, q) = self.thing_2.update(q.q, i, t);
        (o, ThingState { d, q })
    }
}

// Each element in simple tracer has a path and an ID
struct SimpleTracer {
    path: Vec<&'static str>,
    map: HashMap<Vec<&'static str>, Vec<TraceValues>>,
}
