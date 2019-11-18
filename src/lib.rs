mod block;
mod hook;
mod library;

use block::Block;
use hook::Hook;

pub type Status = std::collections::HashMap<String, String>;

pub struct Runtime {
    stat: Status,
    hooks: std::collections::HashMap<String, Hook>,
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            stat: Status::new(),
            hooks: std::collections::HashMap::new(),
        }
    }

    pub fn regist_hook(&mut self, hook: Hook) {
        let name = hook.get_name();
        self.hooks.insert(name, hook);
    }

    pub fn trigger_hook(&mut self, hook_name: &String) {
        if let Some(hook) = self.hooks.get(hook_name) {
            hook.trigger(&mut self.stat);
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
}
