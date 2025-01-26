use log::info;
use solana_account_decoder::{
    parse_account_data::ParsedAccount, UiAccountData,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use spl_token::instruction::close_account;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;

use crate::pump::TOKEN_PROGRAM;
use crate::util::env;

pub async fn close_all_atas(
    rpc_client: Arc<RpcClient>,
    keypair: &Keypair,
) -> Result<(), Box<dyn Error>> {
    let atas = rpc_client
        .get_token_accounts_by_owner(
            &keypair.pubkey(),
            TokenAccountsFilter::ProgramId(TOKEN_PROGRAM),
        )
        .await?;
    info!("Total ATAs: {}", atas.len());
    let owner = keypair.pubkey();
    for ata in atas {
        if let UiAccountData::Json(ParsedAccount {
            program: _,
            parsed,
            space: _,
        }) = ata.account.data
        {
            if parsed["info"]["tokenAmount"]["amount"]
                .as_str()
                .expect("amount")
                == "0"
            {
                info!("Closing ATA: {}", ata.pubkey);
                let rpc_client = Arc::new(RpcClient::new(env("RPC_URL")));
                let tx = Transaction::new_signed_with_payer(
                    &[close_account(
                        &TOKEN_PROGRAM,
                        &Pubkey::from_str(&ata.pubkey)?,
                        &owner,
                        &owner,
                        &[&owner],
                    )?],
                    Some(&owner),
                    &[keypair],
                    rpc_client.get_latest_blockhash().await?,
                );
                let rpc_client = rpc_client.clone();
                tokio::spawn(async move {
                    rpc_client.send_transaction(&tx).await.unwrap();
                });
            }
        }
    }

    Ok(())
}
