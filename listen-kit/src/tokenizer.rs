use std::sync::Mutex;

use anyhow::Result;
use tokenizers::tokenizer::{EncodeInput, Tokenizer};
use tokenizers::InputSequence;

lazy_static::lazy_static! {
    static ref TOKENIZER: Mutex<Tokenizer> = Mutex::new(get_tokenizer());
}

pub fn get_tokenizer() -> Tokenizer {
    let tokenizer_data = include_bytes!("../claude-v3-tokenizer.json");

    Tokenizer::from_bytes(tokenizer_data).unwrap()
}

pub fn tokenize(text: &str) -> Result<Vec<(u32, String)>> {
    let tokenizer = TOKENIZER.lock().unwrap();

    let val = EncodeInput::Single(InputSequence::Raw(text.into()));

    let encoded_text = tokenizer.encode(val, false);

    match encoded_text {
        Ok(encoded_text) => Ok(encoded_text
            .get_ids()
            .iter()
            .zip(encoded_text.get_tokens().iter().cloned())
            .map(|(id, token)| (*id, token.to_string()))
            .collect()),
        Err(err) => Err(anyhow::Error::msg(err.to_string())),
    }
}

pub fn count_tokens(text: &str) -> Result<usize> {
    let tokenizer = TOKENIZER.lock().unwrap();

    let val = EncodeInput::Single(InputSequence::Raw(text.into()));

    let encoded_text = tokenizer.encode(val, false);

    match encoded_text {
        Ok(encoded_text) => Ok(encoded_text.len()),
        Err(err) => Err(anyhow::Error::msg(err.to_string())),
    }
}
