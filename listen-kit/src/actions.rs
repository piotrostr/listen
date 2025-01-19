use anyhow::Result;
use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::str::FromStr;

use crate::deploy_token::{deploy_token, DeployTokenParams};
use crate::price::fetch_token_price;
use crate::trade::trade;
use crate::transfer::{transfer_sol, transfer_spl};

pub struct Actions {
    pub keypair: Keypair,
    pub rpc_client: RpcClient,
    pub client: Client,
}

// TODO
// * add a potential flag to simulate before sending txs, return tx simulation, useful for larger
// txs
impl Actions {
    pub fn new(private_key: String, rpc_url: String) -> Self {
        let keypair = Keypair::from_base58_string(&private_key);
        let rpc_client = RpcClient::new(rpc_url);
        let client = Client::new();

        Self {
            keypair,
            rpc_client,
            client,
        }
    }
    pub async fn trade(
        &self,
        input_mint: String,
        input_amount: u64,
        output_mint: String,
        slippage_bps: u16,
    ) -> Result<String> {
        trade(
            input_mint,
            input_amount,
            output_mint,
            slippage_bps,
            &self.keypair,
        )
        .await
    }

    pub async fn transfer_sol(
        &self,
        to: String,
        amount: u64,
    ) -> Result<String> {
        transfer_sol(
            Pubkey::from_str(&to)?,
            amount,
            &self.keypair,
            &self.rpc_client,
        )
        .await
    }

    /// param amount is token amount, accounting for decimals
    /// e.g. 1 Fartcoin = 1 * 10^6 (6 decimals)
    pub async fn transfer_token(
        &self,
        to: String,
        amount: u64,
        mint: String,
    ) -> Result<String> {
        transfer_spl(
            Pubkey::from_str(&to)?,
            amount,
            Pubkey::from_str(&mint)?,
            &self.keypair,
            &self.rpc_client,
        )
        .await
    }

    pub async fn wallet_address(&self) -> String {
        self.keypair.pubkey().to_string()
    }

    pub async fn get_balance(&self) -> Result<u64> {
        let balance =
            self.rpc_client.get_balance(&self.keypair.pubkey()).await?;
        Ok(balance)
    }

    /// get_token_balance returns the amount as String and the decimals as u8
    /// in order to convert to UI amount: amount / 10^decimals
    pub async fn get_token_balance(
        &self,
        mint: String,
    ) -> Result<(String, u8)> {
        let mint = Pubkey::from_str(&mint)?;
        let ata = spl_associated_token_account::get_associated_token_address(
            &self.keypair.pubkey(),
            &mint,
        );
        let balance = self.rpc_client.get_token_account_balance(&ata).await?;
        Ok((balance.amount, balance.decimals))
    }

    pub async fn deploy_token(
        &self,
        deploy_token_params: DeployTokenParams,
    ) -> Result<String> {
        deploy_token(deploy_token_params, &self.keypair, &self.rpc_client)
            .await
    }

    pub async fn fetch_token_price(&self, mint: String) -> Result<f64> {
        fetch_token_price(mint, &self.client).await
    }

    pub async fn buy_pump_token() -> Result<String> {
        unimplemented!()
    }

    pub async fn sell_pump_token() -> Result<String> {
        unimplemented!()
    }

    pub async fn fetch_metadata(&self) -> Result<String> {
        unimplemented!()
    }

    /// research_token returns aggregated data from any link from metadata
    pub async fn research_token(&self) -> Result<String> {
        unimplemented!()
    }

    pub async fn get_token_data_by_ticker(&self) -> Result<String> {
        unimplemented!()
    }

    pub async fn get_token_data_by_pubkey(&self) -> Result<String> {
        unimplemented!()
    }
}
