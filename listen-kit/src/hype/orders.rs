use crate::signer::SignerContext;
use crate::{common::spawn_with_signer, signer::AsHypeSigner};
use anyhow::Result;
use ethers::signers::LocalWallet;
use hyperliquid_rust_sdk::signer::Signer;
use hyperliquid_rust_sdk::{
    BaseUrl, ClientOrder, ClientOrderRequest, ClientTrigger, ExchangeClient,
};
use rig_tool_macro::tool;
use std::sync::Arc;

#[tool(description = "
Send a market order to the exchange.

example
{
  \"coin\": \"ETH/USDC\",
  \"side\": \"buy\",
  \"size\": \"0.01\",
}

{
  \"coin\": \"ETH/USDC\",
  \"side\": \"sell\",
  \"size\": \"0.01\",
}
")]
pub async fn send_market_order(
    coin: String,
    side: String,
    size: String,
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
        let info = client
            .order(
                ClientOrderRequest {
                    asset: coin,
                    is_buy: side == "buy",
                    reduce_only: false,
                    limit_px: 0.,
                    sz: size.parse::<f64>()?,
                    cloid: None,
                    order_type: ClientOrder::Trigger(ClientTrigger {
                        is_market: true,
                        trigger_px: 0.,
                        tpsl: "".to_string(),
                    }),
                },
                Some(&*signer.as_hype_signer()),
            )
            .await?;
        Ok(serde_json::to_value(info)?)
    })
    .await
    .await?
}

#[tool(description = "
Deposit USDC into the exchange.
{
  \"amount\": \"1000\",
}
")]
pub async fn deposit_usdc(amount: String) -> Result<serde_json::Value> {
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
                &amount,
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
