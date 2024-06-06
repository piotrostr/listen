use std::str::FromStr;

use crate::http_client::HttpClient;
use crate::util::healthz;
use crate::{
    buyer,
    provider::Provider,
    util::{env, pubkey_to_string, string_to_pubkey},
};
use crate::{constants, seller};
use actix_web::post;
use actix_web::web::Json;
use actix_web::{App, Error, HttpResponse, HttpServer};
use base64::Engine;
use futures_util::StreamExt;
use log::{info, warn};
use raydium_library::amm;
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::{EncodableKey, Signer};

#[derive(Deserialize, Serialize)]
pub struct SellRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub amm_pool: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub input_mint: Pubkey,
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub output_mint: Pubkey,
    pub lamports_spent: u64,
    pub insta: Option<bool>,
}

#[post("/sell")]
async fn handle_sell(sell_request: Json<SellRequest>) -> Result<HttpResponse, Error> {
    info!(
        "handling sell_request {}",
        serde_json::to_string_pretty(&sell_request)?
    );
    tokio::spawn(async move {
        let wallet = Keypair::read_from_file(env("FUND_KEYPAIR_PATH")).expect("read wallet");
        let provider = Provider::new(env("RPC_URL"));
        let token_account = spl_associated_token_account::get_associated_token_address(
            &wallet.pubkey(),
            &sell_request.input_mint,
        );
        let pubsub_client = PubsubClient::new(&env("WS_URL"))
            .await
            .expect("make pubsub client");
        let balance = tokio::select! {
                balance = get_spl_balance_stream(&pubsub_client, &token_account) => balance.expect("balance stream"),
                balance = get_spl_balance(&provider, &token_account) => balance.expect("balance rpc"),
        };
        info!("balance: {}", balance);
        if balance == 0 {
            warn!("could not fetch balance, exiting");
            return;
        }
        // TODO generally, those params should be different for pump.fun coins and
        // the standard coins
        // --
        // number one thing now would be to analyze after looking at some charts
        // rn I think the crucial thing is to get rid of the rugs where someone
        // even though all checks pass, some holder dumps $XXK and -99.9%s the token
        if !sell_request.insta.unwrap_or(false) {
            let ok = seller::listen_price(
                &sell_request.amm_pool,
                &provider.rpc_client,
                &pubsub_client,
                Some((balance as f64 * 0.9) as u64),
                Some(sell_request.lamports_spent as f64 * 1.6),
                Some(sell_request.lamports_spent as f64 * 0.69),
                Some(sell_request.lamports_spent),
            )
            .await
            .expect("listen price");
            if !ok {
                return;
            }
        }

        buyer::swap(
            &sell_request.amm_pool,
            &sell_request.input_mint,
            &sell_request.output_mint,
            balance, // sell initial and leave the remainder
            &wallet,
            &provider,
        )
        .await
        .expect("swap");
    });

    Ok(HttpResponse::Ok().json(json!({"status": "OK, triggered sell"})))
}

#[derive(Deserialize, Serialize)]
pub struct SimpleSellRequest {
    #[serde(
        serialize_with = "pubkey_to_string",
        deserialize_with = "string_to_pubkey"
    )]
    pub amm_pool: Pubkey,
}

#[post("/sell-simple")]
async fn handle_sell_simple(sell_request: Json<SimpleSellRequest>) -> Result<HttpResponse, Error> {
    info!(
        "handling simple_sell_request {}",
        serde_json::to_string_pretty(&sell_request)?
    );
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let amm_program = Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY).unwrap();
    let amm_keys = amm::utils::load_amm_keys(&rpc_client, &amm_program, &sell_request.amm_pool)
        .await
        .expect("amm_keys");

    let (input_mint, output_mint) =
        if amm_keys.amm_pc_mint.to_string() == constants::SOLANA_PROGRAM_ID {
            (amm_keys.amm_coin_mint, amm_keys.amm_pc_mint)
        } else {
            (amm_keys.amm_pc_mint, amm_keys.amm_coin_mint)
        };

    HttpClient::new()
        .sell(&SellRequest {
            amm_pool: sell_request.amm_pool,
            input_mint,
            output_mint,
            lamports_spent: 0u64,
            insta: Some(true),
        })
        .await
        .expect("sell");

    Ok(HttpResponse::Ok().json(json!({"status": "OK"})))
}

pub async fn get_spl_balance_stream(
    pubsub_client: &PubsubClient,
    token_account: &Pubkey,
) -> Result<u64, Box<dyn std::error::Error>> {
    let (mut stream, unsub) = pubsub_client
        .account_subscribe(
            token_account,
            Some(RpcAccountInfoConfig {
                commitment: Some(CommitmentConfig::processed()),
                encoding: Some(UiAccountEncoding::Base64),
                ..Default::default()
            }),
        )
        .await
        .expect("account_subscribe");

    tokio::select! {
        log = stream.next() => {
            if let UiAccountData::Binary(data, UiAccountEncoding::Base64) = log.expect("log").value.data {
                let log_data = base64::prelude::BASE64_STANDARD.decode(&data).expect("decode spl b64");
                let spl_account = spl_token::state::Account::unpack(&log_data).expect("unpack spl");
                unsub().await;
                Ok(spl_account.amount)
            } else {
                warn!("get_spl_balance_stream {}: unexpected data", token_account.to_string());
                Err("unexpected data".into())
            }
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(20)) => {
            warn!("get_spl_balance_stream {}: timeout", token_account.to_string());
            Err("timeout".into())
        },
    }
}

pub async fn run_seller_service() -> std::io::Result<()> {
    info!("Running seller service on 8081");
    HttpServer::new(move || {
        App::new()
            .service(handle_sell)
            .service(handle_sell_simple)
            .service(healthz)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}

pub async fn get_spl_balance(
    provider: &Provider,
    token_account: &Pubkey,
) -> Result<u64, Box<dyn std::error::Error>> {
    let mut backoff = 1000;
    for _ in 0..5 {
        match provider
            .rpc_client
            .get_token_account_balance(token_account)
            .await
        {
            Ok(balance) => {
                if balance.amount == "0" {
                    continue;
                }
                return Ok(balance
                    .amount
                    .parse::<u64>()
                    .expect("balance string to u64"));
            }
            Err(e) => {
                warn!("{} error getting balance: {}", token_account.to_string(), e);
                tokio::time::sleep(tokio::time::Duration::from_millis(backoff)).await;
                backoff *= 2;
                continue;
            }
        };
    }
    Err(format!("could not fetch balance for {}", token_account).into())
}
