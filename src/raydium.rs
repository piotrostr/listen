use raydium_library::amm;

use raydium_library::common;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use solana_transaction_status::Encodable;

use crate::{constants, util, Provider};

pub struct Raydium {}

pub struct Swap {
    pre_swap_instructions: Vec<Instruction>,
    post_swap_instructions: Vec<Instruction>,
    signers: Vec<Keypair>,
}

impl Raydium {
    pub fn new() -> Self {
        Self {}
    }

    pub fn swap(
        &self,
        amm_program: Pubkey,
        amm_pool_id: Pubkey,
        input_token_mint: Pubkey,
        output_token_mint: Pubkey,
        slippage_bps: u64,
        amount_specified: u64,
        swap_base_in: bool, // keep false
        wallet: &Keypair,
        provider: &Provider,
    ) -> Result<(), Box<dyn std::error::Error>> {
        dbg!(
            &amm_program,
            &amm_pool_id,
            &input_token_mint,
            &output_token_mint,
            slippage_bps,
            amount_specified,
            swap_base_in,
            &wallet.pubkey(),
        );
        // load amm keys
        let amm_keys = amm::utils::load_amm_keys(
            &provider.rpc_client,
            &amm_program,
            &amm_pool_id,
        )?;
        dbg!(amm_keys);
        // load market keys
        let market_keys = amm::openbook::get_keys_for_market(
            &provider.rpc_client,
            &amm_keys.market_program,
            &amm_keys.market,
        )?;
        dbg!(&market_keys);
        // calculate amm pool vault with load data at the same time or use simulate to calculate
        let result = raydium_library::amm::calculate_pool_vault_amounts(
            &provider.rpc_client,
            &amm_program,
            &amm_pool_id,
            &amm_keys,
            &market_keys,
            amm::utils::CalculateMethod::Simulate(wallet.pubkey()),
        )?;
        dbg!(&result);
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
        dbg!(amount_specified, other_amount_threshold, swap_base_in);
        let mut swap = Swap {
            pre_swap_instructions: vec![],
            post_swap_instructions: vec![],
            signers: vec![],
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
        dbg!(user_source, user_destination);
        let swap_ix = amm::instructions::swap(
            &amm_program,
            &amm_keys,
            &market_keys,
            &wallet.pubkey(),
            &user_source,
            &user_destination,
            amount_specified,
            other_amount_threshold,
            swap_base_in,
        )?;
        println!(
            "swap_ix program_id: {:?}, accounts: {:?} ",
            swap_ix.program_id,
            swap_ix
                .accounts
                .iter()
                .map(|x| x.pubkey)
                .collect::<Vec<Pubkey>>(),
        );
        let ixs = vec![
            swap.pre_swap_instructions,
            vec![swap_ix],
            swap.post_swap_instructions,
        ];
        let mut signing_keypairs: Vec<&Keypair> = vec![];
        signing_keypairs.push(wallet);
        signing_keypairs.append(&mut swap.signers.iter().collect());
        dbg!(signing_keypairs.clone());
        println!(
            "Swapping {} of {} for {} by {}, slippage: {}%, block hash",
            {
                if input_token_mint.to_string() == constants::SOLANA_PROGRAM_ID
                {
                    util::lamports_to_sol(amount_specified)
                } else {
                    amount_specified as f64
                }
            },
            input_token_mint,
            output_token_mint,
            wallet.pubkey(),
            slippage_bps as f32 / 100.,
        );
        let tx = Transaction::new_signed_with_payer(
            &ixs.concat(),
            Some(&wallet.pubkey()),
            &signing_keypairs,
            provider.rpc_client.get_latest_blockhash()?,
        );
        let res = provider.rpc_client.simulate_transaction(&tx).unwrap();
        println!("Simulate: {:?}", res);
        // print the transaction as encoded string
        println!(
            "Transaction: {:?}",
            tx.encode(solana_transaction_status::UiTransactionEncoding::Base58),
        );
        match provider.send_tx(&tx, false) {
            Ok(signature) => {
                println!("Transaction {} successful", signature);
                return Ok(());
            }
            Err(e) => {
                println!("Transaction failed: {}", e);
            }
        };
        // if !dialoguer::Confirm::new()
        //     .with_prompt("Go for it?")
        //     .interact()?
        // {
        //     return Ok(());
        // };
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
        let rent = provider.rpc_client.get_minimum_balance_for_rent_exemption(
            spl_token::state::Account::LEN as usize,
        )?;
        let lamports = rent + amount;
        let token = Keypair::new();
        let mut init_ixs = common::create_init_token(
            &token.pubkey(),
            &mint,
            owner,
            funding,
            lamports,
        );
        let mut close_ixs =
            common::close_account(&token.pubkey(), owner, owner);
        let token_pubkey = token.pubkey();
        swap.signers.push(token);
        swap.pre_swap_instructions.append(&mut init_ixs);
        swap.post_swap_instructions.append(&mut close_ixs);
        Ok(token_pubkey)
    } else {
        let ata_pubkey =
            &spl_associated_token_account::get_associated_token_address(
                &owner, &mint,
            );
        let mut ata_ixs =
            common::create_ata_token_or_not(funding, &mint, owner);
        swap.pre_swap_instructions.append(&mut ata_ixs);
        Ok(*ata_pubkey)
    }
}
