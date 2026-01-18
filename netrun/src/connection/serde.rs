use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

use crate::{compress, decompress};

pub(crate) fn serialize(val: &impl Serialize) -> Result<Vec<u8>> {
    Ok(compress(&serde_json::to_string(&val)?.into_bytes()))
}

pub(crate) fn deserialize<T: DeserializeOwned>(buff: &[u8]) -> Result<T> {
    let data = decompress(buff);
    let json_str = std::str::from_utf8(&data)?;
    Ok(serde_json::from_str(json_str)?)
}
