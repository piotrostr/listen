use std::str::FromStr;

use log::{debug, error, info};
use raydium_library::amm;

use raydium_library::common;
use serde_json::json;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_client::rpc_config::RpcProgramAccountsConfig;
use solana_client::rpc_filter::Memcmp;
use solana_client::rpc_filter::MemcmpEncodedBytes;
use solana_client::rpc_filter::RpcFilterType;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction};
use solana_transaction_status::Encodable;

use crate::{constants, util, Provider};

pub struct Raydium {}

pub struct Swap {
    pre_swap_instructions: Vec<Instruction>,
    post_swap_instructions: Vec<Instruction>,
}

impl Default for Raydium {
    fn default() -> Self {
        Self::new()
    }
}

impl Raydium {
    pub fn new() -> Self {
        Raydium {}
    }

    #[deprecated = "slow and not production required"]
    pub fn get_amm_pool_id(
        &self,
        provider: &Provider,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Pubkey {
        // this is obtained from LIQUIDITY_LAYOUT_V4 from TypeScript Raydium SDK
        const INPUT_MINT_OFFSET: usize = 53;
        const OUTPUT_MINT_OFFSET: usize = 85;

        let accounts = provider
            .rpc_client
            .get_program_accounts_with_config(
                &Pubkey::from_str(constants::OPENBOOK_PROGRAM_ID).unwrap(),
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
                            solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                        ),
                        data_slice: None,
                        min_context_slot: None,
                    },
                    ..Default::default()
                },
            )
            .unwrap();
        println!("{:?}", &accounts);
        Pubkey::default()
    }

    // swap_simple is a wrapper around swap that requires only the token mint
    pub fn swap_simple(&self, _output_token_mint: Pubkey, _sol_amount: u64) {
        // need to fetch amm pool by input/output first
    }

    pub fn swap(
        &self,
        amm_pool_id: Pubkey,
        input_token_mint: Pubkey,
        output_token_mint: Pubkey,
        slippage_bps: u64,
        amount_specified: u64,
        swap_base_in: bool, // keep false
        wallet: &Keypair,
        provider: &Provider,
        confirmed: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let amm_program = Pubkey::from_str(constants::RAYDIUM_LIQUIDITY_POOL_V4_PUBKEY)?;
        // load amm keys
        let amm_keys = amm::utils::load_amm_keys(&provider.rpc_client, &amm_program, &amm_pool_id)?;
        // load market keys
        let market_keys = amm::openbook::get_keys_for_market(
            &provider.rpc_client,
            &amm_keys.market_program,
            &amm_keys.market,
        )?;
        // calculate amm pool vault with load data at the same time or use simulate to calculate
        let result = raydium_library::amm::calculate_pool_vault_amounts(
            &provider.rpc_client,
            &amm_program,
            &amm_pool_id,
            &amm_keys,
            &market_keys,
            amm::utils::CalculateMethod::Simulate(wallet.pubkey()),
        )?;
        let direction = if input_token_mint == amm_keys.amm_coin_mint
            && output_token_mint == amm_keys.amm_pc_mint
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
            amount_specified,
            swap_base_in,
            slippage_bps,
        )?;
        let mut swap = Swap {
            pre_swap_instructions: vec![],
            post_swap_instructions: vec![],
        };
        let user_source = handle_token_account(
            &mut swap,
            provider,
            &input_token_mint,
            amount_specified,
            &wallet.pubkey(),
            &wallet.pubkey(),
        )?;
        let user_destination = handle_token_account(
            &mut swap,
            provider,
            &output_token_mint,
            0,
            &wallet.pubkey(),
            &wallet.pubkey(),
        )?;
        // build swap instruction
        let max_slippage = true;
        let swap_ix = amm::instructions::swap(
            &amm_program,
            &amm_keys,
            &market_keys,
            &wallet.pubkey(),
            &user_source,
            &user_destination,
            amount_specified,
            if max_slippage {
                0
            } else {
                other_amount_threshold
            },
            swap_base_in,
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
        let ixs = vec![
            // TODO make this configurable, currently static but total is still max
            // 0.0005 SOL which is peanuts
            make_compute_budget_ixs(25_000, 500_000),
            swap.pre_swap_instructions,
            vec![swap_ix],
            swap.post_swap_instructions,
        ];
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "amount": amount_specified,
                "input": input_token_mint,
                "output": output_token_mint,
                "funder": wallet.pubkey(),
                "slippage": slippage_bps as f32 / 100.,
            }))
            .unwrap()
        );
        if !confirmed
            && !dialoguer::Confirm::new()
                .with_prompt("Go for it?")
                .interact()?
        {
            return Ok(());
        }
        let tx = Transaction::new_signed_with_payer(
            &ixs.concat(),
            Some(&wallet.pubkey()),
            &[wallet],
            provider.rpc_client.get_latest_blockhash()?,
        );
        let sim_res = provider.rpc_client.simulate_transaction(&tx).unwrap();
        info!("Simulation: {}", serde_json::to_string_pretty(&sim_res)?);
        match provider.send_tx(&tx, true) {
            Ok(signature) => {
                info!("Transaction {} successful", signature);
                return Ok(());
            }
            Err(e) => {
                error!("Transaction failed: {}", e);
                dbg_print_tx(&tx);
            }
        };
        Ok(())
    }
}

pub fn handle_token_account(
    swap: &mut Swap,
    provider: &Provider,
    mint: &Pubkey,
    amount: u64,
    owner: &Pubkey,
    funding: &Pubkey,
) -> Result<Pubkey, Box<dyn std::error::Error>> {
    // two cases - an account is a token account or a native account (WSOL)
    if (*mint).to_string() == constants::SOLANA_PROGRAM_ID {
        let rent = provider
            .rpc_client
            .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)?;
        let lamports = rent + amount;
        let seed = &Keypair::new().pubkey().to_string()[0..32];
        let token = generate_pub_key(owner, seed);
        let mut init_ixs = create_init_token(&token, seed, mint, owner, funding, lamports);
        let mut close_ixs = common::close_account(&token, owner, owner);
        // swap.signers.push(token);
        swap.pre_swap_instructions.append(&mut init_ixs);
        swap.post_swap_instructions.append(&mut close_ixs);
        Ok(token)
    } else {
        let token = &spl_associated_token_account::get_associated_token_address(owner, mint);
        let mut ata_ixs = common::create_ata_token_or_not(funding, mint, owner);
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
        spl_token::instruction::initialize_account(&spl_token::id(), token, mint, owner).unwrap(),
    ]
}

pub fn generate_pub_key(from: &Pubkey, seed: &str) -> Pubkey {
    Pubkey::create_with_seed(from, seed, &spl_token::id()).unwrap()
}

pub fn make_compute_budget_ixs(price: u64, max_units: u32) -> Vec<Instruction> {
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

pub fn dbg_print_tx(tx: &Transaction) {
    debug!(
        "Transaction: {}",
        serde_json::to_string_pretty(
            &tx.encode(solana_transaction_status::UiTransactionEncoding::Base58)
        )
        .unwrap(),
    );
}
