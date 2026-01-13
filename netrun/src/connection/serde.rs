use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

pub(crate) fn serialize(val: &impl Serialize) -> Result<Vec<u8>> {
    Ok(serde_json::to_string(&val)?.into_bytes())
}

pub(crate) fn deserialize<T: DeserializeOwned>(buff: &[u8]) -> Result<T> {
    let json_str = std::str::from_utf8(buff)?;
    Ok(serde_json::from_str(json_str)?)
}
