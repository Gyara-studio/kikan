mod block;
mod hook;
mod library;

use block::Block;
use hook::Hook;
use library::Library;

pub type Status = std::collections::HashMap<String, String>;

#[derive(Default)]
pub struct Runtime<T: Library> {
    stat: Status,
    hooks: std::collections::HashMap<String, Hook>,
    library: Option<T>,
}

impl<T: Library> Runtime<T> {
    pub fn new() -> Runtime<T> {
        Runtime {
            stat: Status::new(),
            hooks: std::collections::HashMap::new(),
            library: None,
        }
    }

    pub fn regist_hook(&mut self, hook: Hook) {
        let name = hook.get_name();
        self.hooks.insert(name, hook);
    }

    pub fn trigger_hook(&mut self, hook_name: &str) {
        if self.library.is_none() {
            // TODO: not set up error
        }
        if let Some(hook) = self.hooks.get(hook_name) {
            hook.trigger(&mut self.stat, self.library.as_mut().unwrap());
        }
    }

    pub fn new_block(&mut self, hook_name: String, title: String, code: String, enable: bool) {
        let block = Block::new(code, title, enable);
        if let Some(hook) = self.hooks.get_mut(&hook_name) {
            hook.insert_block(block);
        } else {
            let mut hook = Hook::new(hook_name);
            hook.insert_block(block);
            self.regist_hook(hook);
        }
    }

    pub fn link_library(&mut self, lib: T) {
        self.library = Some(lib);
    }
}
