use std::collections::HashMap;
use std::str::FromStr;

use log::{debug, info, warn};
use raydium_library::amm;
use raydium_library::amm::AmmKeys;
use raydium_library::amm::MarketPubkeys;
use serde::Serialize;
use serde_json::Value;
use solana_account_decoder::parse_account_data::ParsedAccount;
use solana_account_decoder::UiAccountData;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_client::rpc_response::RpcKeyedAccount;
use solana_sdk::signer::EncodableKey;
use spl_token::instruction::burn;
use spl_token::state::Mint;
use std::error::Error;
use timed::timed;
use utoipa::ToSchema;

use crate::jito::send_jito_tx;
use crate::seller_service::load_amm_keys;
use crate::{constants, Provider};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use raydium_library::common;
use reqwest::Client;
use serde_json::json;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_client::rpc_filter::Memcmp;
use solana_client::rpc_filter::MemcmpEncodedBytes;
use solana_client::rpc_filter::RpcFilterType;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Default, Clone, Serialize, ToSchema)]
pub struct Holding {
    pub mint: String,
    pub ata: String,
    pub amount: u64,
}

pub fn parse_holding(ata: RpcKeyedAccount) -> Result<Holding, Box<dyn Error>> {
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
        Err("failed to parse holding".into())
    }
}

/// sweep_raydium is a bit iffy in terms of creating the objects on every swap,
/// but it works and does not require refactoring the existing raydium code
/// it works just fine for a few hundred swaps to perform as part of sweep,
/// but it makes sense to refactor a bit for large scale operations
pub async fn sweep_raydium(
    rpc_client: &RpcClient,
    wallet_path: String,
) -> Result<(), Box<dyn Error>> {
    let owner = Keypair::read_from_file(&wallet_path)?.pubkey();
    let atas = rpc_client
        .get_token_accounts_by_owner(
            &owner,
            TokenAccountsFilter::ProgramId(spl_token::id()),
        )
        .await?;
    info!("found {} token accounts", atas.len());
    let holdings = atas
        .iter()
        .map(|ata| parse_holding(ata.clone()).expect("parse holding"))
        .filter(|holding| holding.amount > 0)
        .collect::<Vec<Holding>>();
    let holdings_set = holdings
        .iter()
        .map(|h| h.mint.to_string())
        .collect::<std::collections::HashSet<String>>();

    download_raydium_json(false).await?;

    info!("opening raydium.json and parsing the pools (might take a while)");
    let raw_pools: Value =
        serde_json::from_str(&std::fs::read_to_string("raydium.json")?)?;
    let pools = raw_pools["unOfficial"].as_array().unwrap();
    info!("parsed raydium.json, found {} pools", pools.len());
    let mut pools_map = HashMap::new();
    let mut quote_or_base_map = HashMap::new();
    for pool in pools {
        let base_mint = pool["baseMint"].as_str().unwrap();
        if holdings_set.contains(base_mint) {
            pools_map.insert(base_mint.to_string(), pool);
            quote_or_base_map.insert(base_mint.to_string(), "base");
        }
        let quote_mint = pool["quoteMint"].as_str().unwrap();
        if holdings_set.contains(quote_mint) {
            pools_map.insert(quote_mint.to_string(), pool);
            quote_or_base_map.insert(quote_mint.to_string(), "quote");
        }
    }

    info!(
        "got {} relevant pools, total of {} holdings (ideally shoul equal)",
        pools_map.len(),
        holdings.len()
    );

    for holding in holdings {
        let _frozenlist = ["BEhY5iV6NNGcYnM3miZWAc5jau7Z37foQyQ3BdnyAAGn"];
        info!("{} burning token instead", holding.mint);
        let tx = Transaction::new_signed_with_payer(
            &[burn(
                &spl_token::id(),
                &Pubkey::from_str(&holding.ata)?,
                &Pubkey::from_str(&holding.mint)?,
                &owner,
                &[&owner],
                holding.amount,
            )?],
            Some(&owner),
            &[&Keypair::read_from_file(&wallet_path)?],
            rpc_client.get_latest_blockhash().await?,
        );
        match rpc_client
            .send_transaction_with_config(
                &tx,
                RpcSendTransactionConfig {
                    encoding: None,
                    skip_preflight: true,
                    preflight_commitment: None,
                    max_retries: None,
                    min_context_slot: None,
                },
            )
            .await
        {
            Ok(signature) => {
                info!("burn transaction: {}", signature);
            }
            Err(e) => {
                warn!("burn transaction failed: {}", e)
            }
        };
    }

    Ok(())
}

pub async fn download_raydium_json(
    update: bool,
) -> Result<(), Box<dyn Error>> {
    if Path::new("raydium.json").exists() && !update {
        warn!("raydium.json already exists. Skipping download.");
        return Ok(());
    }

    let url = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";
    let client = Client::new();
    let res = client.get(url).send().await?;
    let total_size = res.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut file = File::create("raydium.json")?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;
        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message("Download completed");
    Ok(())
}

pub struct Raydium {}

pub struct SwapArgs {
    pub amm_pool: Pubkey,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub amount: u64,
    pub slippage: u64,
    pub wallet: Keypair,
    pub rpc_client: RpcClient,
    pub confirmed: bool,
    /// no_sanity: skip sanity checks
    pub no_sanity: bool,
}

pub struct Swap {
    pre_swap_instructions: Vec<Instruction>,
    post_swap_instructions: Vec<Instruction>,
}

pub struct SwapContext {
    pub amm_program: Pubkey,
    pub amm_pool: Pubkey,
    pub amm_keys: amm::AmmKeys,
    pub market_keys: amm::openbook::MarketPubkeys,
    pub swap: Swap,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub amount: u64,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub slippage: u64,
    pub swap_base_in: bool,
}

pub async fn get_calc_result(
    rpc_client: &RpcClient,
    amm_pool: &Pubkey,
) -> Result<(amm::CalculateResult, MarketPubkeys, AmmKeys), Box<dyn Error>> {
    let amm_program = constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY;

    // load amm keys
    let amm_keys =
        amm::utils::load_amm_keys(rpc_client, &amm_program, amm_pool).await?;
    debug!("amm keys: {:?}", amm_keys);
    // load market keys
    let market_keys = amm::openbook::get_keys_for_market(
        rpc_client,
        &amm_keys.market_program,
        &amm_keys.market,
    )
    .await?;
    debug!("market keys: {:?}", market_keys);

    let result = amm::calculate_pool_vault_amounts(
        rpc_client,
        &amm_program,
        amm_pool,
        &amm_keys,
        &market_keys,
        amm::utils::CalculateMethod::CalculateWithLoadAccount,
    )
    .await?;
    debug!("result: {:?}", result);

    Ok((result, market_keys, amm_keys))
}

pub fn get_burn_pct(
    mint_data: Mint,
    result: amm::CalculateResult,
) -> Result<f64, Box<dyn Error>> {
    // Calculate divisor for token decimals
    let base = 10u64;
    let divisor = base.pow(mint_data.decimals as u32);

    // Convert lp_reserve and supply to proper scale
    let lp_reserve = result.pool_lp_amount as f64 / divisor as f64;
    let supply = mint_data.supply as f64 / divisor as f64;

    // Calculate max_lp_supply and burn_amount
    let max_lp_supply = lp_reserve.max(supply);
    let burn_amount = max_lp_supply - supply;

    // Avoid division by zero and ensure correct burn percentage calculation
    let burn_pct = if max_lp_supply > 0.0 {
        (burn_amount / max_lp_supply) * 100.0
    } else {
        0.0
    };

    debug!(
        "LP total: {}, LP pooled: {}, LP burnt: {}",
        max_lp_supply, lp_reserve, burn_amount
    );

    Ok(burn_pct)
}

pub fn calc_result_to_financials(
    coin_mint_is_sol: bool,
    result: amm::CalculateResult,
    owner_balance: u64,
) -> f64 {
    let sol_price = 145.;
    if coin_mint_is_sol {
        let sol_amount = result.pool_coin_vault_amount as f64 / 1e9;
        let usd_amount = sol_amount * sol_price;
        let price = result.pool_coin_vault_amount as f64
            / result.pool_pc_vault_amount as f64;
        let owner_balance_sol = owner_balance as f64 * price / 1e9;
        debug!(
            "{}",
            serde_json::to_string_pretty(&json!(
                {
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "sol_amount": sol_amount,
                    "usd_amount": usd_amount,
                    "price": price,
                    "owner_balance": owner_balance,
                    "owner_balance_sol": owner_balance_sol,
                }
            ))
            .expect("to string pretty")
        );
        sol_amount
    } else {
        let sol_amount = result.pool_pc_vault_amount as f64 / 1e9;
        let usd_amount = sol_amount * sol_price;
        let price = result.pool_pc_vault_amount as f64
            / result.pool_coin_vault_amount as f64;
        let owner_balance_sol = owner_balance as f64 * price / 1e9;
        debug!(
            "{}",
            serde_json::to_string_pretty(&json!(
                {
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "sol_amount": sol_amount,
                    "usd_amount": usd_amount,
                    "price": price,
                    "owner_balance": owner_balance,
                    "owner_balance_sol": owner_balance_sol,
                }
            ))
            .expect("to string pretty")
        );
        sol_amount
    }
}

pub async fn make_swap_context(
    rpc_client: &RpcClient,
    amm_pool: Pubkey,
    input_token_mint: Pubkey,
    output_token_mint: Pubkey,
    wallet: &Keypair,
    slippage: u64,
    amount: u64,
) -> Result<SwapContext, Box<dyn Error>> {
    let amm_program = constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY;
    // load amm keys
    let amm_keys = load_amm_keys(rpc_client, &amm_program, &amm_pool).await?;
    // load market keys
    let market_keys = amm::openbook::get_keys_for_market(
        rpc_client,
        &amm_keys.market_program,
        &amm_keys.market,
    )
    .await?;
    let mut swap = Swap {
        pre_swap_instructions: vec![],
        post_swap_instructions: vec![],
    };
    let user_source = handle_token_account(
        &mut swap,
        rpc_client,
        &input_token_mint,
        amount,
        &wallet.pubkey(),
        &wallet.pubkey(),
    )
    .await?;
    let user_destination = handle_token_account(
        &mut swap,
        rpc_client,
        &output_token_mint,
        0,
        &wallet.pubkey(),
        &wallet.pubkey(),
    )
    .await?;
    Ok(SwapContext {
        amm_program,
        amm_keys,
        amm_pool,
        market_keys,
        swap,
        user_source,
        user_destination,
        amount,
        input_token_mint,
        output_token_mint,
        slippage,
        swap_base_in: true,
    })
}

#[timed(duration(printer = "info!"))]
pub async fn make_swap_ixs(
    rpc_client: &RpcClient,
    wallet: &Keypair,
    swap_context: &SwapContext,
    quick: bool,
) -> Result<Vec<Instruction>, Box<dyn Error>> {
    // calculate amm pool vault with load data at the same time or use simulate to calculate
    // this step adds some latency, could be pre-calculated while waiting for the JITO leader
    let other_amount_threshold = if !quick {
        let result = raydium_library::amm::calculate_pool_vault_amounts(
            rpc_client,
            &swap_context.amm_program,
            &swap_context.amm_pool,
            &swap_context.amm_keys,
            &swap_context.market_keys,
            amm::utils::CalculateMethod::Simulate(wallet.pubkey()),
        )
        .await?;
        self::calc_result_to_financials(
            swap_context.market_keys.coin_mint.to_string()
                == constants::SOLANA_PROGRAM_ID.to_string(),
            result,
            0,
        );
        // let sol_amount = result.pool_coin_vault_amount as f64 / 1e9;
        // if sol_amount < 50. {
        //     // TODO make this configurable, 7k (50 sol) is a bare threshold
        //     return Err("Pool is small, aborting swap".into());
        // }
        let direction = if swap_context.input_token_mint
            == swap_context.amm_keys.amm_coin_mint
            && swap_context.output_token_mint
                == swap_context.amm_keys.amm_pc_mint
        {
            amm::utils::SwapDirection::Coin2PC
        } else {
            amm::utils::SwapDirection::PC2Coin
        };
        let other_amount_threshold = amm::swap_with_slippage(
            result.pool_pc_vault_amount,
            result.pool_coin_vault_amount,
            result.swap_fee_numerator,
            result.swap_fee_denominator,
            direction,
            swap_context.amount,
            swap_context.swap_base_in,
            swap_context.slippage,
        )
        .unwrap_or(0);

        let mint_account = rpc_client
            .get_account(&swap_context.output_token_mint)
            .await?;
        let mint_data = Mint::unpack(&mint_account.data)?;
        let burn_pct =
            self::get_burn_pct(mint_data, result).expect("get burn pct");

        info!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "pool_pc_vault_amount": result.pool_pc_vault_amount,
                "pool_coin_vault_amount": result.pool_coin_vault_amount,
                "lp_mint": swap_context.amm_keys.amm_lp_mint.to_string(),
                "pool_lp_amount": result.pool_lp_amount,
                "swap_fee_numerator": result.swap_fee_numerator,
                "swap_fee_denominator": result.swap_fee_denominator,
                "other_amount_threshold": other_amount_threshold,
                "liquidity_burn_pct": burn_pct,
            }))?
        );
        if burn_pct < 90. {
            return Err(format!("LP is only {} burnt", burn_pct).into());
        }

        other_amount_threshold
    } else {
        info!("Quick swap, skipping pool vault calculation");
        0
    };
    // let market_cap = util::lamports_to_sol(result.pool_coin_vault_amount);
    // info!("market cap: {}", market_cap);
    // if market_cap < 50. {
    //     return Err("Market cap too low, aborting swap".into());
    // }
    let swap_ix = amm::instructions::swap(
        &swap_context.amm_program,
        &swap_context.amm_keys,
        &swap_context.market_keys,
        &wallet.pubkey(),
        &swap_context.user_source,
        &swap_context.user_destination,
        swap_context.amount,
        other_amount_threshold,
        swap_context.swap_base_in,
    )?;
    debug!(
        "swap_ix program_id: {:?}, accounts: {} ",
        swap_ix.program_id,
        serde_json::to_string_pretty(
            &swap_ix
                .accounts
                .iter()
                .map(|x| x.pubkey.to_string())
                .collect::<Vec<String>>()
        )?,
    );
    let ixs = [
        make_compute_budget_ixs(0, 300_000),
        swap_context.swap.pre_swap_instructions.clone(),
        vec![swap_ix],
        swap_context.swap.post_swap_instructions.clone(),
    ];
    Ok(ixs.concat())
}

impl Default for Raydium {
    fn default() -> Self {
        Self::new()
    }
}

impl Raydium {
    pub const fn new() -> Self {
        Raydium {}
    }

    #[deprecated = "slow and not production required"]
    pub async fn get_amm_pool_id(
        &self,
        rpc_client: &RpcClient,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Pubkey {
        // this is obtained from LIQUIDITY_LAYOUT_V4 from TypeScript Raydium SDK
        const INPUT_MINT_OFFSET: usize = 53;
        const OUTPUT_MINT_OFFSET: usize = 85;

        let _accounts =
            rpc_client
            .get_program_accounts_with_config(
                &constants::OPENBOOK_PROGRAM_ID,
                RpcProgramAccountsConfig {
                    filters: Some(vec![
                        RpcFilterType::Memcmp(Memcmp::new(
                            INPUT_MINT_OFFSET,
                            MemcmpEncodedBytes::Base64(input_mint.to_string()),
                        )),
                        RpcFilterType::Memcmp(Memcmp::new(
                            OUTPUT_MINT_OFFSET,
                            MemcmpEncodedBytes::Base64(output_mint.to_string()),
                        )),
                    ]),
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        commitment: Some(
                            solana_sdk::commitment_config::CommitmentConfig::processed(),
                        ),
                        data_slice: None,
                        min_context_slot: None,
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        Pubkey::default()
    }

    // swap_simple is a wrapper around swap that requires only the token mint
    pub fn swap_simple(&self, _output_token_mint: Pubkey, _sol_amount: u64) {
        // need to fetch amm pool by input/output first, not critical but useful
    }

    pub async fn swap(
        &self,
        swap_args: SwapArgs,
    ) -> Result<(), Box<dyn Error>> {
        let SwapArgs {
            amm_pool,
            input_token_mint,
            output_token_mint,
            amount,
            slippage,
            wallet,
            rpc_client,
            confirmed,
            no_sanity,
        } = swap_args;
        let swap_context = self::make_swap_context(
            &rpc_client,
            amm_pool,
            input_token_mint,
            output_token_mint,
            &wallet,
            slippage,
            amount,
        )
        .await?;
        let ixs = self::make_swap_ixs(
            &rpc_client,
            &wallet,
            &swap_context,
            no_sanity,
        )
        .await?;
        info!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "amount": amount,
                "input": input_token_mint.to_string(),
                "output": output_token_mint.to_string(),
                "funder": wallet.pubkey().to_string(),
                "slippage": slippage,
            }))?
        );
        if !confirmed
            && !dialoguer::Confirm::new()
                .with_prompt("Go for it?")
                .interact()?
        {
            return Ok(());
        }
        let tx = Transaction::new_signed_with_payer(
            ixs.as_slice(),
            Some(&wallet.pubkey()),
            &[&wallet],
            rpc_client.get_latest_blockhash().await?,
        );
        let sim_res = rpc_client.simulate_transaction(&tx).await?;
        info!("Simulation: {}", serde_json::to_string_pretty(&sim_res)?);
        send_jito_tx(tx).await?;
        Ok(())
    }
}

pub async fn handle_token_account(
    swap: &mut Swap,
    rpc_client: &RpcClient,
    mint: &Pubkey,
    amount: u64,
    owner: &Pubkey,
    funding: &Pubkey,
) -> Result<Pubkey, Box<dyn Error>> {
    // two cases - an account is a token account or a native account (WSOL)
    if mint.eq(&constants::SOLANA_PROGRAM_ID) {
        let rent = rpc_client
            .get_minimum_balance_for_rent_exemption(
                spl_token::state::Account::LEN,
            )
            .await?;
        let lamports = rent + amount;
        let seed = &Keypair::new().pubkey().to_string()[0..32];
        let token = generate_pub_key(owner, seed);
        let mut init_ixs =
            create_init_token(&token, seed, mint, owner, funding, lamports);
        let mut close_ixs = common::close_account(&token, owner, owner);
        // swap.signers.push(token);
        swap.pre_swap_instructions.append(&mut init_ixs);
        swap.post_swap_instructions.append(&mut close_ixs);
        Ok(token)
    } else {
        let token =
            &spl_associated_token_account::get_associated_token_address(
                owner, mint,
            );
        let mut ata_ixs =
            common::create_ata_token_or_not(funding, mint, owner);
        swap.pre_swap_instructions.append(&mut ata_ixs);
        Ok(*token)
    }
}

pub fn create_init_token(
    token: &Pubkey,
    seed: &str,
    mint: &Pubkey,
    owner: &Pubkey,
    funding: &Pubkey,
    lamports: u64,
) -> Vec<Instruction> {
    vec![
        solana_sdk::system_instruction::create_account_with_seed(
            funding,
            token,
            owner,
            seed,
            lamports,
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_account(
            &spl_token::id(),
            token,
            mint,
            owner,
        )
        .unwrap(),
    ]
}

pub fn generate_pub_key(from: &Pubkey, seed: &str) -> Pubkey {
    Pubkey::create_with_seed(from, seed, &spl_token::id()).unwrap()
}

pub fn make_compute_budget_ixs(
    price: u64,
    max_units: u32,
) -> Vec<Instruction> {
    vec![
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(price),
        solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(max_units),
    ]
}

pub fn make_priority_compute_budget_ixs(
    _provider: &Provider,
    _addressess: &[Pubkey],
) -> Vec<Instruction> {
    // let res = provider.rpc_client.get_recent_prioritization_fees(addresses).unwrap();
    vec![]
}
