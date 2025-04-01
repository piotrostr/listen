use anyhow::Result;
use privy::util::base64decode;

use crate::reasoning_loop::StreamResponse;

pub fn ensure_base64_rendered(content: &str) -> Result<String> {
    // Check if this is a JSON result wrapper
    if content.contains("<content>") {
        decode_content_tags(content)
    } else {
        Ok(content.to_string())
    }
}

fn decode_content_tags(content: &str) -> Result<String> {
    let mut result = content.to_string();
    let mut position = 0;

    while let Some(start) = result[position..].find("<content>") {
        let start = position + start;
        if let Some(end) = result[start..].find("</content>") {
            let end = start + end;
            let base64_content = &result[start + "<content>".len()..end];

            let decoded_bytes = base64decode(base64_content)?;
            let decoded_message =
                serde_json::from_slice::<StreamResponse>(&decoded_bytes)?;

            // Replace the entire tag including content
            result.replace_range(
                start..end + "</content>".len(),
                &serde_json::to_string(&decoded_message)?,
            );

            position = start + 1;
        } else {
            break;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ensure_base64_rendered() {
        let resultstr = include_str!("sample.txt");
        let result = ensure_base64_rendered(&resultstr).unwrap();
        println!("{}", result);
    }
}
