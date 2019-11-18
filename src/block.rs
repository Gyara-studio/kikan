use crate::Status;

pub struct Block {
    code: String,
    hook: String,
    title: String,
    pub enable: bool,
}

impl Block {
    pub fn run(&self, stat: &mut Status) {}
}
