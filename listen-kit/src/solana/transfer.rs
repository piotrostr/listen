use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;

pub async fn create_transfer_sol_tx(
    to: &Pubkey,
    amount: u64,
    from: &Pubkey,
) -> Result<Transaction> {
    let tx = Transaction::new_with_payer(
        &[solana_sdk::system_instruction::transfer(from, to, amount)],
        Some(from),
    );
    Ok(tx)
}

pub async fn create_transfer_spl_tx(
    to: &Pubkey,
    amount: u64,
    mint: &Pubkey,
    from: &Pubkey,
    rpc_client: &RpcClient,
) -> Result<Transaction> {
    let from_ata = spl_associated_token_account::get_associated_token_address(
        from, mint,
    );
    let to_ata =
        spl_associated_token_account::get_associated_token_address(to, mint);

    let mut instructions = vec![];

    // Check if recipient's ATA exists, if not create it
    if rpc_client.get_account(&to_ata).await.is_err() {
        instructions.push(
            spl_associated_token_account::instruction::create_associated_token_account(
                from,
                to,
                mint,
                &spl_token::id(),
            ),
        );
    }

    instructions.push(spl_token::instruction::transfer(
        &spl_token::id(),
        &from_ata,
        &to_ata,
        from,
        &[],
        amount,
    )?);

    let tx = Transaction::new_with_payer(&instructions, Some(from));

    Ok(tx)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use solana_sdk::native_token::sol_to_lamports;
    use solana_sdk::pubkey;

    use super::*;
    use crate::solana::util::{make_rpc_client, make_test_signer};

    #[tokio::test]
    async fn test_transfer_sol() {
        let signer = make_test_signer();
        let owner = Pubkey::from_str(&signer.pubkey()).unwrap();
        let amount = sol_to_lamports(0.0001);
        let mut tx = create_transfer_sol_tx(&owner, amount, &owner)
            .await
            .unwrap();
        let result = signer.sign_and_send_solana_transaction(&mut tx).await;
        assert!(result.is_ok(), "{:?}", result);
    }

    #[tokio::test]
    async fn test_transfer_spl() {
        let signer = make_test_signer();
        let rpc_client = make_rpc_client();
        let owner = Pubkey::from_str(&signer.pubkey()).unwrap();
        let mint = pubkey!("Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump");
        let amount = (10. * 1e6) as u64;
        let mut tx = create_transfer_spl_tx(
            &owner,
            amount,
            &mint,
            &owner,
            &rpc_client,
        )
        .await
        .unwrap();
        let result = signer.sign_and_send_solana_transaction(&mut tx).await;
        assert!(result.is_ok(), "{:?}", result);
    }
}
