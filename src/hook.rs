use crate::{block::Block, Status};

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

    pub fn trigger(&self, stat: &mut Status) {
        for block in self.blocks.iter() {
            if block.enable == true {
                block.run(stat);
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
