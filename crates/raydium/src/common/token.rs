use solana_sdk::{
    instruction::Instruction, program_pack::Pack, pubkey::Pubkey,
};

pub fn create_ata_token_or_not(
    funding: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Vec<Instruction> {
    vec![
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            funding,
            owner,
            mint,
            &spl_token::id(),
        ),
    ]
}

pub fn create_init_token(
    token: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
    funding: &Pubkey,
    lamports: u64,
) -> Vec<Instruction> {
    vec![
        solana_sdk::system_instruction::create_account(
            funding,
            token,
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

pub fn create_init_mint(
    funding: &Pubkey,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    decimals: u8,
    lamports: u64,
) -> Vec<Instruction> {
    vec![
        solana_sdk::system_instruction::create_account(
            funding,
            mint,
            lamports,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        ),
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint,
            mint_authority,
            None,
            decimals,
        )
        .unwrap(),
    ]
}

pub fn mint_to(
    mint: &Pubkey,
    to_token: &Pubkey,
    mint_authority: &Pubkey,
    amount: u64,
) -> Vec<Instruction> {
    vec![spl_token::instruction::mint_to(
        &spl_token::id(),
        mint,
        &to_token,
        &mint_authority,
        &[],
        amount,
    )
    .unwrap()]
}

pub fn transfer_to(
    from: &Pubkey,
    to: &Pubkey,
    from_authority: &Pubkey,
    amount: u64,
) -> Vec<Instruction> {
    vec![spl_token::instruction::transfer(
        &spl_token::id(),
        from,
        to,
        &from_authority,
        &[],
        amount,
    )
    .unwrap()]
}

pub fn close_account(
    close_account: &Pubkey,
    destination: &Pubkey,
    close_authority: &Pubkey,
) -> Vec<Instruction> {
    vec![spl_token::instruction::close_account(
        &spl_token::id(),
        close_account,
        destination,
        &close_authority,
        &[],
    )
    .unwrap()]
}
