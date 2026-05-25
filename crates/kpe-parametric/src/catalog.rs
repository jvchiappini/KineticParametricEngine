use std::collections::HashMap;
use kpe_schema::block::BlockDefinition;

pub struct Catalog {
    custom_blocks: HashMap<String, BlockDefinition>,
    industry_blocks: HashMap<String, BlockDefinition>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            custom_blocks: HashMap::new(),
            industry_blocks: HashMap::new(),
        }
    }

    pub fn register_custom(&mut self, block: BlockDefinition) {
        self.custom_blocks.insert(block.id.clone(), block);
    }

    pub fn register_industry(&mut self, block: BlockDefinition) {
        self.industry_blocks.insert(block.id.clone(), block);
    }

    pub fn get(&self, id: &str) -> Option<&BlockDefinition> {
        self.custom_blocks.get(id)
            .or_else(|| self.industry_blocks.get(id))
    }

    pub fn list_custom(&self) -> Vec<&BlockDefinition> {
        self.custom_blocks.values().collect()
    }

    pub fn list_industry(&self) -> Vec<&BlockDefinition> {
        self.industry_blocks.values().collect()
    }

    pub fn len(&self) -> usize {
        self.custom_blocks.len() + self.industry_blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.custom_blocks.is_empty() && self.industry_blocks.is_empty()
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
