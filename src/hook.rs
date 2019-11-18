use crate::{block::Block, Status};

pub struct Hook {
    name: String,
    blooks: Vec<Block>,
}

impl Hook {
    pub fn trigger(&self, stat: &mut Status) {
        for block in self.blooks.iter() {
            if block.enable == true {
                block.run(stat);
            }
        }
    }
}
