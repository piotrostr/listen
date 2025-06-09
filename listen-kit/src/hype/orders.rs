use crate::evm::transfer::create_transfer_erc20_tx;
use crate::evm::util::{execute_evm_transaction, make_provider};
use crate::signer::SignerContext;
use crate::{common::spawn_with_signer, signer::AsHypeSigner};
use anyhow::Result;
use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::signer::Signer;
use hyperliquid_rust_sdk::{
    BaseUrl, ClientOrder, ClientOrderRequest, ClientTrigger, ExchangeClient,
    MarketOrderParams,
};
use rig_tool_macro::tool;
use std::sync::Arc;

#[tool(description = "
Open a market order on the exchange.

Parameters:
- coin: the coin to trade
- side: the side of the order
- size: the size of the order
- leverage: the leverage to use, e.g. 0.1 eth with price of eth $1000 is 100 usdc notional, with 4x leverage requires minimum 25 usdc margin

Example
{
  \"coin\": \"ETH\",
  \"side\": \"buy\",
  \"size\": \"0.01\",
}

{
  \"coin\": \"ETH\",
  \"side\": \"sell\",
  \"size\": \"0.01\",
}

Minimum buy size is 10 USDC worth, less will result in error creating the order.
")]
pub async fn market_open(
    coin: String,
    side: String,
    size: String,
    leverage: u32,
) -> Result<serde_json::Value> {
    let signer = SignerContext::current().await;
    spawn_with_signer(signer.clone(), move || async move {
        let client = ExchangeClient::new(
            None,
            signer.as_hype_signer(),
            Some(BaseUrl::Mainnet),
            None,
            None,
        )
        .await?;
        client.update_leverage(leverage, &coin, true, None).await?;
        let info = client
            .market_open(MarketOrderParams {
                asset: &coin,
                is_buy: side == "buy",
                sz: size.parse::<f64>()?,
                px: None,
                cloid: None,
                slippage: Some(0.001), // .1% slippage
                wallet: None,
            })
            .await?;
        Ok(serde_json::to_value(info)?)
    })
    .await
    .await?
}

#[tool(description = "
Deposit USDC into the exchange.
{
  \"amount\": \"10000000\", // 10 usdc, 6 decimals
}
Minimum amount is 5 USDC, if less is sent it will not be accepted and be lost forever.
")]
pub async fn deposit_usdc(amount: String) -> Result<String> {
    if amount.parse::<u64>().unwrap() < 5_000_000 {
        return Err(anyhow::anyhow!("Minimum amount is 5 USDC"));
    }
    execute_evm_transaction(move |owner| async move {
        create_transfer_erc20_tx(
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string(), // usdc
            "0x2df1c51e09aecf9cacb7bc98cb1742757f163df7".to_string(), // hyperliquid deposit bridge 2
            amount,
            &make_provider(42161)?,
            owner,
        )
        .await
    })
    .await
}

// this is a transfer to self, doesnt really do anything but the part with spawning an agent to be used elsewhere
pub async fn _action_with_agent() -> Result<serde_json::Value> {
    let signer = SignerContext::current().await;
    spawn_with_signer(signer.clone(), move || async move {
        let exchange_client = ExchangeClient::new(
            None,
            signer.as_hype_signer(),
            Some(BaseUrl::Mainnet),
            None,
            None,
        )
        .await?;
        let (private_key, response) = exchange_client
            .approve_agent(Some(&*signer.as_hype_signer()))
            .await?;
        let wallet: LocalWallet = private_key.parse()?;
        let wallet_signer: Arc<dyn Signer> = Arc::new(wallet);
        let exchange_client = ExchangeClient::new(
            None,
            wallet_signer,
            Some(BaseUrl::Mainnet),
            None,
            None,
        )
        .await?;
        tracing::info!("approve_agent: {:?}", response);
        exchange_client
            .usdc_transfer(
                "123",
                &signer
                    .address()
                    .ok_or(anyhow::anyhow!("No address found"))?,
                Some(&signer.as_hype_signer()),
            )
            .await?;
        Ok(serde_json::json!({}))
    })
    .await
    .await?
}
