use crate::jito::send_jito_tx;
use crate::pump::{
    _make_buy_ixs, get_bonding_curve, get_pump_token_amount,
    make_pump_sell_ix, mint_to_pump_accounts,
};
use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

fn apply_slippage(amount: u64, slippage_bps: u16) -> u64 {
    let slippage = amount * slippage_bps as u64 / 10_000;
    amount - slippage
}

pub async fn buy_pump_fun(
    mint: String,
    sol_amount: u64,
    slippage_bps: u16,
    rpc_client: &RpcClient,
    keypair: &Keypair,
) -> Result<String> {
    let mint = Pubkey::from_str(&mint)?;
    let pump_accounts = mint_to_pump_accounts(&mint);

    let bonding_curve =
        get_bonding_curve(rpc_client, pump_accounts.bonding_curve).await?;
    let token_amount = get_pump_token_amount(
        bonding_curve.virtual_sol_reserves,
        bonding_curve.virtual_token_reserves,
        Some(bonding_curve.real_token_reserves),
        sol_amount,
    )?;

    let owner = keypair.pubkey();

    let buy_ixs = _make_buy_ixs(
        owner,
        pump_accounts.mint,
        pump_accounts.bonding_curve,
        pump_accounts.associated_bonding_curve,
        apply_slippage(token_amount, slippage_bps),
        sol_amount,
    )?;

    let latest_blockhash = rpc_client.get_latest_blockhash().await?;

    let tx = Transaction::new_signed_with_payer(
        buy_ixs.as_slice(),
        Some(&owner),
        &[&keypair],
        latest_blockhash,
    );

    let result = send_jito_tx(tx).await?;
    Ok(result)
}

pub async fn sell_pump_fun(
    mint: String,
    token_amount: u64,
    rpc_client: &RpcClient,
    keypair: &Keypair,
) -> Result<String> {
    let mint = Pubkey::from_str(&mint)?;
    let pump_accounts = mint_to_pump_accounts(&mint);

    let owner = keypair.pubkey();

    let ata = spl_associated_token_account::get_associated_token_address(
        &owner,
        &pump_accounts.mint,
    );

    let ix = make_pump_sell_ix(owner, pump_accounts, token_amount, ata)?;

    let latest_blockhash = rpc_client.get_latest_blockhash().await?;

    let tx = Transaction::new_signed_with_payer(
        [ix].as_slice(),
        Some(&owner),
        &[&keypair],
        latest_blockhash,
    );

    let result = send_jito_tx(tx).await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use solana_sdk::native_token::sol_to_lamports;

    use super::*;
    use crate::util::{load_keypair_for_tests, make_rpc_client};

    #[tokio::test]
    async fn test_buy_pump_fun() {
        let keypair = load_keypair_for_tests();
        let rpc_client = make_rpc_client();
        buy_pump_fun(
            "76VCegXJdjqHXBdQyeVV3Swt3JgXrBoQpXcvRQsYpump".to_string(),
            sol_to_lamports(0.0001),
            200,
            &rpc_client,
            &keypair,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_sell_pump_fun() {
        let keypair = load_keypair_for_tests();
        let rpc_client = make_rpc_client();
        sell_pump_fun(
            "76VCegXJdjqHXBdQyeVV3Swt3JgXrBoQpXcvRQsYpump".to_string(),
            (1. * 1e6) as u64,
            &rpc_client,
            &keypair,
        )
        .await
        .unwrap();
    }
}
