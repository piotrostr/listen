use anyhow::Result;

use rig_tool_macro::tool;

use super::trade::create_trade_tx;
use super::util::{execute_evm_transaction, make_provider};

// TODO it is worth to include description of the function, possibly using
// docstring for the model to understand what is going on stuff like lamports
// vs ether decimals vs pump decimals and so on etc models can do but need
// to be explicitly stated
#[tool]
pub async fn check_router_allowance() -> Result<bool> {
    todo!()
}

#[tool]
pub async fn trade(
    input_token_address: String,
    input_amount: String,
    output_token_address: String,
) -> Result<String> {
    execute_evm_transaction(move |owner| async move {
        create_trade_tx(
            input_token_address,
            input_amount,
            output_token_address,
            &make_provider()?,
            owner,
        )
        .await
    })
    .await
}

#[tool]
pub async fn transfer_eth() -> Result<String> {
    todo!()
}

#[tool]
pub async fn transfer_erc20() -> Result<String> {
    todo!()
}

#[tool]
pub async fn wallet_address() -> Result<String> {
    todo!()
}

#[tool]
pub async fn get_balance() -> Result<String> {
    todo!()
}

#[tool]
pub async fn get_token_balance() -> Result<String> {
    todo!()
}
