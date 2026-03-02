use std::collections::BTreeMap;

use anyhow::Result;
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::rest::request::request_object;

pub async fn get<T: DeserializeOwned>(url: impl ToString) -> Result<T> {
    request_object(super::Method::Get, url, BTreeMap::default(), None).await
}

pub async fn download(url: impl ToString) -> Result<Vec<u8>> {
    let url = url.to_string();
    let client = Client::new();
    let bytes = client.get(&url).send().await?.bytes().await?;
    Ok(bytes.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not_wasm)]
    mod not_wasm_tests {
        use super::*;

        #[tokio::test]
        async fn test_download() -> Result<()> {
            let bytes = download("https://www.lrt.lt/img/2026/02/26/2327389-490277-615x345.jpg").await?;
            assert_eq!(bytes.len(), 40246);
            Ok(())
        }
    }
}
