use serde::{de::Error, Deserialize, Deserializer};
use serde_json::Value;

pub fn deserialize_string_or_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
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

pub fn deserialize_string_or_object<'de, D>(deserializer: D) -> Result<String, D::Error>
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
