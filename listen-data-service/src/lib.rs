pub mod config;
pub mod package;
pub mod pb;
pub mod substreams;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::{RecordId, Surreal};

use self::pb::sf::substreams::rpc::v2::{BlockScopedData, BlockUndoSignal};

#[derive(Serialize, Deserialize)]
struct Cursor {
    id: Option<RecordId>,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Block {
    number: u64,
    hash: String,
    timestamp: i64,
    data: serde_json::Value,
}

pub async fn persist_cursor(db: &Surreal<Client>, cursor: String) -> surrealdb::Result<()> {
    let _: Option<Cursor> = db
        .update(("cursor", "latest"))
        .content(Cursor {
            id: None,
            value: cursor,
        })
        .await?;
    Ok(())
}

pub async fn load_persisted_cursor(db: &Surreal<Client>) -> Result<Option<String>> {
    let cursor: Option<Cursor> = db.select(("cursor", "latest")).await?;
    Ok(cursor.map(|c| c.value))
}

pub async fn process_block_undo_signal(
    db: &Surreal<Client>,
    undo_signal: &BlockUndoSignal,
) -> Result<()> {
    if let Some(last_valid_block) = &undo_signal.last_valid_block {
        // Delete records from the invalid block range
        let query = "DELETE FROM blocks WHERE number > $block";
        let _ = db
            .query(query)
            .bind(("block", last_valid_block.number))
            .await?;
    };

    Ok(())
}

pub async fn process_block_scoped_data(
    _db: &Surreal<Client>,
    data: &BlockScopedData,
) -> Result<()> {
    let block_data = data.output.as_ref().unwrap();
    println!("Processing block: {:?}", block_data);

    // let block: Block = parse_block(block_data)?;

    // Store in SurrealDB
    // let result: Result<Vec<_>, surrealdb::Error> = db.create("blocks").content(block_data).await;

    // match result {
    //     Ok(_) => Ok(()),
    //     Err(e) => {
    //         eprintln!("Failed to store block: {}", e);
    //         Err(e.into())
    //     }
    // }
    //
    Ok(())
}
