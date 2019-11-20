pub trait Library {
    fn run(&mut self, stack: &mut Vec<String>);
}
