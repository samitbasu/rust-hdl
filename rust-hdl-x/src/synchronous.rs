pub trait Synchronous {
    type Input;
    type Output;
    type State: Default + Copy;

    fn update(&self, state: Self::State, inputs: Self::Input) -> (Self::Output, Self::State);
    fn default_output(&self) -> Self::Output;
}
