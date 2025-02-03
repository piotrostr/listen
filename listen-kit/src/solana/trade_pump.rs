use crate::solana::pump::{
    _make_buy_ixs, get_bonding_curve, get_pump_token_amount,
    make_pump_sell_ix, mint_to_pump_accounts,
};
use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

fn apply_slippage(amount: u64, slippage_bps: u16) -> u64 {
    let slippage = amount * slippage_bps as u64 / 10_000;
    amount - slippage
}

pub async fn create_buy_pump_fun_tx(
    mint: String,
    sol_amount: u64,
    slippage_bps: u16,
    rpc_client: &RpcClient,
    owner: &Pubkey,
) -> Result<Transaction> {
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

    let buy_ixs = _make_buy_ixs(
        *owner,
        pump_accounts.mint,
        pump_accounts.bonding_curve,
        pump_accounts.associated_bonding_curve,
        apply_slippage(token_amount, slippage_bps),
        sol_amount,
    )?;

    let tx = Transaction::new_with_payer(buy_ixs.as_slice(), Some(owner));

    Ok(tx)
}

pub async fn create_sell_pump_fun_tx(
    mint: String,
    token_amount: u64,
    owner: &Pubkey,
) -> Result<Transaction> {
    let mint = Pubkey::from_str(&mint)?;
    let pump_accounts = mint_to_pump_accounts(&mint);

    let ata = spl_associated_token_account::get_associated_token_address(
        owner,
        &pump_accounts.mint,
    );

    let ix = make_pump_sell_ix(*owner, pump_accounts, token_amount, ata)?;

    let tx = Transaction::new_with_payer([ix].as_slice(), Some(owner));

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use solana_sdk::native_token::sol_to_lamports;

    use super::*;
    use crate::solana::util::{make_rpc_client, make_test_signer};

    #[tokio::test]
    async fn test_buy_pump_fun() {
        let signer = make_test_signer();
        let rpc_client = make_rpc_client();
        let mut tx = create_buy_pump_fun_tx(
            "76VCegXJdjqHXBdQyeVV3Swt3JgXrBoQpXcvRQsYpump".to_string(),
            sol_to_lamports(0.0001),
            500,
            &rpc_client,
            &Pubkey::from_str(&signer.pubkey()).unwrap(),
        )
        .await
        .unwrap();
        let result = signer.sign_and_send_solana_transaction(&mut tx).await;
        assert!(result.is_ok(), "{:?}", result);
    }

    #[tokio::test]
    async fn test_sell_pump_fun() {
        let signer = make_test_signer();
        let mut tx = create_sell_pump_fun_tx(
            "76VCegXJdjqHXBdQyeVV3Swt3JgXrBoQpXcvRQsYpump".to_string(),
            (1. * 1e6) as u64,
            &Pubkey::from_str(&signer.pubkey()).unwrap(),
        )
        .await
        .unwrap();
        let result = signer.sign_and_send_solana_transaction(&mut tx).await;
        assert!(result.is_ok(), "{:?}", result);
    }
}
