pub(crate) trait Layer {
    type Output;
    type Input;
    type Err;

    fn update(&mut self, input: Self::Input) -> Result<Self::Output, Self::Err>;
}
