use crate::{block::Block, library::Library, Status};

pub struct Hook {
    name: String,
    blocks: Vec<Block>,
}

impl Hook {
    pub fn new(name: String) -> Hook {
        Hook {
            name,
            blocks: Vec::new(),
        }
    }

    pub fn trigger<T: Library>(&self, stat: &mut Status, lib: &mut T) {
        for block in self.blocks.iter() {
            if block.enable == true {
                block.run(stat, lib);
            }
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn insert_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}
