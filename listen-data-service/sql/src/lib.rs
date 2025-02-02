mod pb;

use core::panic;
use substreams_database_change::pb::database::DatabaseChanges;
use sha2::{Sha256, Digest};
use substreams_database_change::tables::Tables as DatabaseChangeTables;
use crate::pb::sf::substreams::solana::v1::Transactions;
use prost::Message;

#[substreams::handlers::map]
fn db_out(input: Transactions) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut tables = DatabaseChangeTables::new();

    substreams::skip_empty_output();
    // No Id field has been found...
    // The input has been hashed to create a unique ID, replace it with the field you want to use as ID
    let mut hasher = Sha256::new();
    let mut buf = vec![];
    input.encode(&mut buf).unwrap();
    hasher.update(buf);
    let result = hasher.finalize();
    let row = tables.create_row("Transactions", format!("{:x}", result));
    row.set("id", format!("{:x}", result));

    Ok(tables.to_database_changes())
}
