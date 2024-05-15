use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{
    constants,
    jito::{self, SearcherClient},
    provider::Provider,
    raydium,
    tx_parser::NewPool,
};
use dotenv_codegen::dotenv;
use log::{info, warn};
use raydium_library::amm;
use solana_client::nonblocking;
use solana_sdk::{signature::Keypair, signer::Signer};

// Trader is a wrapper to listen on liquidity burn of a new listing
// plus verify that the supply is not centralized, and perform same sanity
// checks as listener
// this is to separate out listening and parsing too and enable off-line
// processing
// ideally it would also track position and sell at the right time
pub struct Trader {}

pub async fn handle_new_pair(
    new_pool_info: NewPool,
    amount: u64,
    slippage: u64,
    wallet: &Keypair,
    provider: &Provider,
    searcher_client: &mut SearcherClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let mint = if new_pool_info.input_mint.to_string()
        == constants::SOLANA_PROGRAM_ID
    {
        new_pool_info.output_mint
    } else {
        new_pool_info.input_mint
    };

    let (is_safe, msg) =
        provider.sanity_check(&mint).await.expect("sanity check");
    if !is_safe
        && !dialoguer::Confirm::new()
            .with_prompt(format!("Unsafe pool {}: {}", mint, msg))
            .interact()
            .unwrap()
    {
        warn!("Unsafe pool, skipping");
        return Ok(());
    }

    let swap_context = raydium::make_swap_context(
        provider,
        new_pool_info.amm_pool_id,
        new_pool_info.input_mint,
        new_pool_info.output_mint,
        wallet,
        slippage,
        amount,
    )
    .await
    .expect("makes swap context");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up the Ctrl+C signal handler
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    info!("Waiting for burn pct to be over 90%, ctrl+c to continue with other pairs");

    // this is blocking but the messages wait in the websocket, so if a new pair
    // comes around and liquidity is locked, it is ok to wait here
    // prod might wanna use tokio and spawn this in a task
    while running.load(Ordering::SeqCst) {
        let result = raydium_library::amm::calculate_pool_vault_amounts(
            &provider.rpc_client,
            &swap_context.amm_program,
            &swap_context.amm_pool,
            &swap_context.amm_keys,
            &swap_context.market_keys,
            amm::utils::CalculateMethod::Simulate(wallet.pubkey()),
        )?;
        let burn_pct =
            raydium::get_burn_pct(&provider.rpc_client, &mint, result)
                .expect("get burn pct");

        if burn_pct > 90. {
            break;
        }

        info!("Burn pct is {}, waiting", burn_pct);
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    let start = std::time::Instant::now();
    let quick = true;
    let mut ixs =
        raydium::make_swap_ixs(provider, wallet, &swap_context, quick)
            .expect("make swap ixs");

    info!("took {:?} to pack", start.elapsed());

    let tip = 50000;

    let rpc_client = &nonblocking::rpc_client::RpcClient::new(
        dotenv!("RPC_URL").to_string(),
    );
    let swap_result =
        jito::send_swap_tx(&mut ixs, tip, wallet, searcher_client, &rpc_client)
            .await;

    info!("swap result: {:?}", swap_result);

    Ok(())
}
