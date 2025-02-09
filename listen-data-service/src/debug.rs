use crate::constants::{USDC_MINT_KEY_STR, WSOL_MINT_KEY_STR};
use crate::raydium_intruction_processor::Diff;
use crate::raydium_intruction_processor::RaydiumAmmV4InstructionProcessor;

#[cfg(test)]
impl RaydiumAmmV4InstructionProcessor {
    pub async fn _debug(&self, signature: &str, diffs: Vec<Diff>) {
        println!("https://solscan.io/tx/{}", signature);

        let swapped_tokens = diffs
            .iter()
            .map(|diff| diff.mint.as_str())
            .collect::<Vec<&str>>();

        if swapped_tokens.contains(&WSOL_MINT_KEY_STR)
            && swapped_tokens.contains(&USDC_MINT_KEY_STR)
        {
            for diff in diffs {
                match self.kv_store.get_metadata(&diff.mint).await {
                    Ok(Some(metadata)) => {
                        println!(
                            "{}: {} ({} -> {})",
                            metadata.mpl.name, diff.diff, diff.pre_amount, diff.post_amount
                        );
                    }
                    _ => {
                        println!(
                            "{}: {} ({} -> {})",
                            diff.mint, diff.diff, diff.pre_amount, diff.post_amount
                        );
                    }
                }
            }
        }
        println!("--------------------------------");
    }
}
