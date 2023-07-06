pub trait Synchronous {
    type Input;
    type Output;
    type State: Default;

    fn update(&self, state: Self::State, inputs: Self::Input) -> (Self::Output, Self::State);
}
