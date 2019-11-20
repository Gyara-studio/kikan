use crate::{library::Library, Status};

pub struct Block {
    _title: String,
    pub enable: bool,
    ast_stack: Vec<String>,
}

impl Block {
    pub fn new(code: String, _title: String, enable: bool) -> Block {
        let mut block = Block {
            _title,
            enable,
            ast_stack: Vec::new(),
        };
        block.set_code(code);
        block
    }

    fn analyze_code(&mut self, _code: String) {
        self.ast_stack.clear();
    }

    pub fn set_code(&mut self, code: String) {
        self.analyze_code(code);
    }

    pub fn run<T: Library>(&self, _stat: &mut Status, _lib: &mut T) {}
}
