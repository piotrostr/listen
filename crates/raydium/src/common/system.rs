use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

pub fn create_rent_exempt(
    from: &Pubkey,
    to: &Pubkey,
    owner: &Pubkey,
    lamports: u64,
    space: u64,
) -> Vec<Instruction> {
    vec![solana_sdk::system_instruction::create_account(
        from, to, lamports, space, owner,
    )]
}
