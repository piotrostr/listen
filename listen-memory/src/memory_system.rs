use crate::memory_note::MemoryNote;
use anyhow::Result;

pub struct MemorySystem {}

impl MemorySystem {
    pub fn from_env() -> Self {
        Self {}
    }

    pub fn add_note(&self, content: String) -> Result<()> {
        todo!()
    }
}
