use rust_hdl_x::{
    basic_logger_builder, synchronous::Synchronous, LogBuilder, Loggable, Logger, TagID,
};
use rust_hdl_x_macro::Loggable;

pub mod counter;
pub mod reg_fifo;
pub mod shot;
//pub mod strobe;

#[derive(Debug)]
struct Bar {
    counter: u16,
    tag_input: TagID<u16>,
    tag_output: TagID<bool>,
    tag_state: TagID<u16>,
    tag_next_state: TagID<u16>,
}

impl Bar {
    fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        let tag_state = builder.tag("state");
        let tag_next_state = builder.tag("next_state");
        Self {
            counter: 0,
            tag_input,
            tag_output,
            tag_state,
            tag_next_state,
        }
    }
}

impl Synchronous for Bar {
    type Input = u16;
    type Output = bool;
    type State = u16;

    fn compute(
        &self,
        mut trace: impl Logger,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        let next_state = state + inputs;
        let output = next_state % 2 == 0;
        trace.log(self.tag_input, inputs);
        trace.log(self.tag_output, output);
        trace.log(self.tag_state, state);
        trace.log(self.tag_next_state, next_state);
        (output, next_state)
    }
}

#[derive(Debug)]
struct Foo {
    sub1: Bar,
    sub2: Bar,
    tag_input: TagID<u16>,
    tag_output: TagID<MoreJunk>,
    tag_state: TagID<u16>,
    tag_next_state: TagID<u16>,
}

impl Foo {
    fn new(mut builder: impl LogBuilder) -> Self {
        let tag_input = builder.tag("input");
        let tag_output = builder.tag("output");
        let tag_state = builder.tag("state");
        let tag_next_state = builder.tag("next_state");
        Self {
            sub1: Bar::new(builder.scope("sub1")),
            sub2: Bar::new(builder.scope("sub2")),
            tag_input,
            tag_output,
            tag_state,
            tag_next_state,
        }
    }
}

impl Synchronous for Foo {
    type Input = u16;
    type Output = MoreJunk;
    type State = u16;

    fn compute(
        &self,
        mut logger: impl Logger,
        state: Self::State,
        inputs: Self::Input,
    ) -> (Self::Output, Self::State) {
        // Update the submodules
        logger.log(self.tag_input, inputs);
        logger.log(self.tag_state, state);
        let (sub1_out, sub1_state) = self.sub1.compute(&mut logger, state, inputs);
        let (sub2_out, sub2_state) = self.sub2.compute(&mut logger, state, inputs);
        // Do our own update
        let output = MoreJunk::default();
        let state = sub1_state + sub2_state;
        logger.log(self.tag_output, output);
        logger.log(self.tag_next_state, state);
        (output, state)
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
enum State {
    #[default]
    Boot,
    Running,
}

impl Loggable for State {
    fn allocate<L: Loggable>(tag: TagID<L>, builder: impl LogBuilder) {
        builder.allocate(tag, 0)
    }
    fn record<L: Loggable>(&self, tag: TagID<L>, mut logger: impl Logger) {
        match self {
            State::Boot => logger.write_string(tag, "Boot"),
            State::Running => logger.write_string(tag, "Running"),
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Loggable, PartialEq)]
struct DeepJunk {
    x: u32,
    y: u16,
}

#[derive(Default, Clone, Copy, Debug, Loggable, PartialEq)]
struct Junk {
    a: bool,
    b: u8,
    c: State,
    d: DeepJunk,
}

#[derive(Default, Copy, Clone, Debug, Loggable, PartialEq)]
struct MoreJunk {
    a: Junk,
    b: Junk,
}

#[test]
fn test_trace_setup() {
    let mut logger_builder = basic_logger_builder::BasicLoggerBuilder::default();
    let foo = Foo::new(&mut logger_builder);
    println!("{}", logger_builder);
    println!("{:#?}", foo);
    let logger = logger_builder.build();
    println!("{}", logger);
    let mut vcd = vec![];
    logger.vcd(&mut vcd).unwrap();
    //    println!("{}", String::from_utf8(vcd).unwrap());
    std::fs::write("empty.vcd", vcd).unwrap();
}

#[test]
fn test_using_address() {
    struct Foo {
        id: usize,
    }

    struct Junk {
        id: usize,
        bar1: Foo,
        bar2: Foo,
    }

    let jnk = Junk {
        id: 0,
        bar1: Foo { id: 1 },
        bar2: Foo { id: 2 },
    };

    println!("{:?}", &jnk as *const Junk);
    println!("{:?}", &jnk.bar1 as *const Foo);
    println!("{:?}", &jnk.bar2 as *const Foo);
}
