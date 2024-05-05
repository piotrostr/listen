// vim notes
// - re-enable github copilot some way, defo some possible workaround
//   - might redelete cache?
use crate::types::{QuoteRequest, QuoteResponse, SwapRequest, SwapResponse};
use reqwest::Client;
use solana_sdk::signer::Signer;

pub struct Jupiter {
    client: Client,
}

impl Jupiter {
    pub fn new() -> Jupiter {
        Jupiter {
            client: Client::new(),
        }
    }

    async fn get_quote(
        &self,
        request: &QuoteRequest,
    ) -> Result<QuoteResponse, Box<dyn std::error::Error>> {
        let res = self
            .client
            .get("https://quote-api.jup.ag/v6/quote")
            .query(request)
            .send()
            .await?;

        if res.status() != 200 {
            return Err(res.text().await.unwrap().into());
        }

        Ok(res.json::<QuoteResponse>().await?)
    }

    async fn get_swap_tx(
        &self,
        request: &SwapRequest,
    ) -> Result<SwapResponse, Box<dyn std::error::Error>> {
        let res = self
            .client
            .post("https://quote-api.jup.ag/v6/swap")
            .json(request)
            .send()
            .await?;

        if res.status() != 200 {
            println!("{}", serde_json::to_string(&request)?);
            return Err(res.text().await.unwrap().into());
        }

        Ok(res.json::<SwapResponse>().await?)
    }

    pub async fn swap(
        &self,
        input_mint: String,
        output_mint: String,
        amount: u64,
        signer: &dyn Signer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match self
            .get_quote(&QuoteRequest {
                input_mint,
                output_mint,
                amount,
                slippage_bps: Some(50),
                swap_mode: Some("ExactIn".to_string()),
                ..QuoteRequest::default()
            })
            .await
        {
            Ok(response) => {
                println!("{}", serde_json::to_string_pretty(&response)?);

                let swap_response = self
                    .get_swap_tx(&SwapRequest {
                        user_public_key: signer.pubkey().to_string(),
                        quote_response: response,
                        ..SwapRequest::default()
                    })
                    .await?;

                // TODO sign the transaction
            }
            Err(e) => println!("Error fetching quote: {:?}", e),
        }
        Ok(())
    }
}
