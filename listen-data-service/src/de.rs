use serde::{de::Error, Deserialize, Deserializer};
use serde_json::Value;

pub fn deserialize_string_or_bool<'de, D>(
    deserializer: D,
) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrBool {
        String(String),
        Bool(bool),
    }

    match Option::<StringOrBool>::deserialize(deserializer)? {
        Some(StringOrBool::String(s)) => match s.to_lowercase().as_str() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => Err(Error::custom(format!("Invalid boolean string: {}", s))),
        },
        Some(StringOrBool::Bool(b)) => Ok(Some(b)),
        None => Ok(None),
    }
}

pub fn deserialize_string_or_object<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Object(_) | Value::Null => Ok("".to_string()),
        _ => Err(Error::custom(format!(
            "Expected string or object, got: {}",
            value
        ))),
    }
}

pub fn deserialize_optional_string_or_object<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(Some(s)),
        Value::Object(_) | Value::Null => Ok(None),
        _ => Err(Error::custom(format!(
            "Expected string or object, got: {}",
            value
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::metadata::IpfsMetadata;

    #[test]
    fn test_ipfs_metadata_bool_deserialization() {
        // Test string "true"
        let string_true = serde_json::json!({
            "name": "Test",
            "symbol": "TST",
            "showName": "true"
        });

        // Test boolean true
        let bool_true = serde_json::json!({
            "name": "Test",
            "symbol": "TST",
            "showName": true
        });

        let metadata1: IpfsMetadata =
            serde_json::from_value(string_true).unwrap();
        let metadata2: IpfsMetadata =
            serde_json::from_value(bool_true).unwrap();

        assert_eq!(metadata1.show_name, Some(true));
        assert_eq!(metadata2.show_name, Some(true));
    }

    #[test]
    fn test_ipfs_metadata_object_fields() {
        let object_fields = serde_json::json!({
            "name": "test",
            "symbol": "TST",
            "description": {},
            "twitter": null,
            "website": {}
        });

        let metadata: IpfsMetadata =
            serde_json::from_value(object_fields).unwrap();

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.twitter, None);
        assert_eq!(metadata.website, None);
    }
}
