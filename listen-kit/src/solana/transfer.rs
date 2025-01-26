use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

use crate::solana::{blockhash::BLOCKHASH_CACHE, transaction::send_tx};

pub async fn transfer_sol(
    to: Pubkey,
    amount: u64,
    keypair: &Keypair,
) -> Result<String> {
    let from = keypair.pubkey();
    let recent_blockhash = BLOCKHASH_CACHE.get_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[solana_sdk::system_instruction::transfer(&from, &to, amount)],
        None,
        &[&keypair],
        recent_blockhash,
    );
    let res = send_tx(tx).await?;

    Ok(res)
}

pub async fn transfer_spl(
    to: Pubkey,
    amount: u64,
    mint: Pubkey,
    keypair: &Keypair,
    rpc_client: &RpcClient,
) -> Result<String> {
    let from = keypair.pubkey();
    let from_ata = spl_associated_token_account::get_associated_token_address(
        &from, &mint,
    );
    let to_ata = spl_associated_token_account::get_associated_token_address(
        &to, &mint,
    );

    let mut instructions = vec![];

    // Check if recipient's ATA exists, if not create it
    if rpc_client.get_account(&to_ata).await.is_err() {
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                &from,
                &to,
                &mint,
                &spl_token::id(),
            ),
        );
    }

    instructions.push(spl_token::instruction::transfer(
        &spl_token::id(),
        &from_ata,
        &to_ata,
        &from,
        &[],
        amount,
    )?);

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&from),
        &[keypair],
        BLOCKHASH_CACHE.get_blockhash().await?,
    );

    let res = send_tx(tx).await?;

    Ok(res)
}

#[cfg(test)]
mod tests {
    use solana_sdk::native_token::sol_to_lamports;
    use solana_sdk::pubkey;
    use solana_sdk::signer::Signer;

    use super::*;
    use crate::solana::util::{load_keypair_for_tests, make_rpc_client};

    #[tokio::test]
    async fn test_transfer_sol() {
        let keypair = load_keypair_for_tests();
        let to = keypair.pubkey();
        let amount = sol_to_lamports(0.0001);
        let result = transfer_sol(to, amount, &keypair).await;
        assert!(result.is_ok(), "{:?}", result);
    }

    #[tokio::test]
    async fn test_transfer_spl() {
        let keypair = load_keypair_for_tests();
        let rpc_client = make_rpc_client();
        let to = keypair.pubkey();
        let mint = pubkey!("Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump");
        let amount = (10. * 1e6) as u64;
        let result =
            transfer_spl(to, amount, mint, &keypair, &rpc_client).await;
        assert!(result.is_ok(), "{:?}", result);
    }
}
