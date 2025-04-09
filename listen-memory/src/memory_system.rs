use crate::memory_note::MemoryNote;
use anyhow::Result;

pub const K: usize = 5;

pub struct MemorySystem {}

impl MemorySystem {
    pub fn from_env() -> Self {
        Self {}
    }

    pub fn add_note(&self, content: String) -> Result<()> {
        todo!()
    }

    pub fn find_related_memories(&self, query: String) -> Result<Vec<MemoryNote>> {
        todo!()
    }

    pub fn process_memory(&self, memory: MemoryNote) -> Result<()> {
        todo!()
    }
}
