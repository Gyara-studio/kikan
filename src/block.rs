use crate::Status;

pub struct Block {
    title: String,
    pub enable: bool,
    ast_stack: Vec<String>,
}

impl Block {
    pub fn new(code: String, title: String, enable: bool) -> Block {
        let mut block = Block {
            title,
            enable,
            ast_stack: Vec::new(),
        };
        block.set_code(code);
        block
    }

    fn analyze_code(&mut self, code: String) {
        self.ast_stack.clear();
    }

    pub fn set_code(&mut self, code: String) {
        self.analyze_code(code);
    }

    pub fn run(&self, _stat: &mut Status) {}
}
