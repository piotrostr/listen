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

// pub async fn top_holders_check(provider: &Provider, mint: &Pubkey) {
//     provider.rpc_client.get_account(pubkey)
// }

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
    if !is_safe {
        warn!("Unsafe pool: {}, skipping", msg);
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

    info!("Waiting for burn pct to be over 90%, ctrl+c to continue with other pairs");

    // this is blocking but the messages wait in the websocket, so if a new pair
    // comes around and liquidity is locked, it is ok to wait here
    // prod might wanna use tokio and spawn this in a task
    let mut ok = false;
    for _ in 0..10 {
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

        let coin_mint_is_sol = swap_context.market_keys.coin_mint.to_string()
            == constants::SOLANA_PROGRAM_ID;
        let pooled_sol =
            raydium::calc_result_to_financials(coin_mint_is_sol, result, 0);
        if pooled_sol < 100. {
            info!("pooled sol: {} is below the 100. thresh", pooled_sol);
            info!("there is {}% liq burnt", burn_pct);
            ok = false;
            break;
        }

        if burn_pct > 90. {
            ok = true;
            break;
        }

        info!("Burn pct is {}, waiting", burn_pct);
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    if !ok {
        warn!("not ok to swap, skipping");
        return Ok(());
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
