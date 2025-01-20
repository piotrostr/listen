use std::str::FromStr;

use anyhow::{anyhow, Result};
use serde::Serialize;
use solana_account_decoder::parse_account_data::ParsedAccount;
use solana_account_decoder::UiAccountData;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_client::rpc_response::RpcKeyedAccount;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Holding {
    pub mint: String,
    pub ata: String,
    pub amount: u64,
}

pub fn parse_holding(ata: RpcKeyedAccount) -> Result<Holding> {
    if let UiAccountData::Json(ParsedAccount {
        program: _,
        parsed,
        space: _,
    }) = ata.account.data
    {
        let amount = parsed["info"]["tokenAmount"]["amount"]
            .as_str()
            .expect("amount")
            .parse::<u64>()?;
        let mint =
            Pubkey::from_str(parsed["info"]["mint"].as_str().expect("mint"))?;
        let ata = Pubkey::from_str(&ata.pubkey)?;
        Ok(Holding {
            mint: mint.to_string(),
            ata: ata.to_string(),
            amount,
        })
    } else {
        Err(anyhow!("failed to parse holding"))
    }
}

pub async fn get_holdings(
    rpc_client: &RpcClient,
    owner: &Pubkey,
) -> Result<Vec<Holding>> {
    let atas = rpc_client
        .get_token_accounts_by_owner(
            owner,
            TokenAccountsFilter::ProgramId(spl_token::id()),
        )
        .await?;
    let holdings = atas
        .iter()
        .map(|ata| parse_holding(ata.clone()).expect("parse holding"))
        .filter(|holding| holding.amount > 0)
        .collect::<Vec<Holding>>();

    Ok(holdings)
}
