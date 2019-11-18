use crate::Status;

pub struct Block {
    pub code: String,
    pub hook: String,
    pub title: String,
    pub enable: bool,
}

impl Block {
    pub fn run(&self, _stat: &mut Status) {}
}
