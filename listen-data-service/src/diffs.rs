use std::collections::HashMap;

use anyhow::Result;
use solana_transaction_status::{
    TransactionTokenBalance, UiTransactionTokenBalance,
};

use crate::constants::{RAYDIUM_AUTHORITY_MINT_KEY_STR, WSOL_MINT_KEY_STR};

pub trait TokenBalanceInfo {
    fn get_mint(&self) -> &str;
    fn get_ui_amount(&self) -> Option<f64>;
    fn get_owner(&self) -> &str;
}

impl TokenBalanceInfo for TransactionTokenBalance {
    fn get_mint(&self) -> &str {
        &self.mint
    }

    fn get_ui_amount(&self) -> Option<f64> {
        self.ui_token_amount.ui_amount
    }

    fn get_owner(&self) -> &str {
        &self.owner
    }
}

impl TokenBalanceInfo for UiTransactionTokenBalance {
    fn get_mint(&self) -> &str {
        &self.mint
    }

    fn get_ui_amount(&self) -> Option<f64> {
        self.ui_token_amount.ui_amount
    }

    fn get_owner(&self) -> &str {
        self.owner.as_ref().map(|s| s.as_str()).unwrap_or_default()
    }
}

pub fn process_diffs(
    diffs: &Vec<Diff>,
    sol_price: f64,
) -> Result<(f64, f64, String)> {
    let (token0, token1) = (&diffs[0], &diffs[1]);

    let amount0 = token0.diff.abs();
    let amount1 = token1.diff.abs();

    let (sol_amount, token_amount, coin_mint) =
        match (token0.mint.as_str(), token1.mint.as_str()) {
            (WSOL_MINT_KEY_STR, other_mint) => (amount0, amount1, other_mint),
            (other_mint, WSOL_MINT_KEY_STR) => (amount1, amount0, other_mint),
            _ => return Err(anyhow::anyhow!("Non-WSOL swap")),
        };

    let price = (sol_amount.abs() / token_amount.abs()) * sol_price;
    let swap_amount = sol_amount * sol_price;

    Ok((price, swap_amount, coin_mint.to_string()))
}
#[derive(Debug)]
pub struct Diff {
    pub mint: String,
    pub pre_amount: f64,
    pub post_amount: f64,
    pub diff: f64,
    pub owner: String,
}

pub fn get_token_balance_diff<T: TokenBalanceInfo + std::fmt::Debug>(
    pre_balances: &[T],
    post_balances: &[T],
) -> Vec<Diff> {
    let mut diffs = Vec::new();
    let mut pre_balances_map = HashMap::new();
    let mut post_balances_map = HashMap::new();

    for balance in pre_balances {
        if let Some(amount) = balance.get_ui_amount() {
            let key = (
                balance.get_mint().to_string(),
                balance.get_owner().to_string(),
            );
            pre_balances_map.insert(key, amount);
        }
    }

    for balance in post_balances {
        if let Some(amount) = balance.get_ui_amount() {
            let key = (
                balance.get_mint().to_string(),
                balance.get_owner().to_string(),
            );
            post_balances_map.insert(key, amount);
        }
    }

    // dont take the diffs from the raydium authority mint nor zero diffs
    let should_collect = |diff: &Diff| diff.diff != 0.0;

    for ((mint, owner), pre_amount) in pre_balances_map.iter() {
        if let Some(post_amount) =
            post_balances_map.get(&(mint.clone(), owner.clone()))
        {
            let diff = post_amount - pre_amount;
            let res = Diff {
                mint: mint.clone(),
                pre_amount: *pre_amount,
                post_amount: *post_amount,
                diff,
                owner: owner.clone(),
            };
            if should_collect(&res) {
                diffs.push(res);
            }
        }
    }

    for ((mint, owner), post_amount) in post_balances_map {
        if !pre_balances_map.contains_key(&(mint.clone(), owner.clone())) {
            let res = Diff {
                mint,
                pre_amount: 0.0,
                post_amount,
                diff: post_amount,
                owner,
            };
            if should_collect(&res) {
                diffs.push(res);
            }
        }
    }

    diffs
}
