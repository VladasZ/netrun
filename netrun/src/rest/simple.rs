use std::collections::BTreeMap;

use anyhow::Result;
use serde::de::DeserializeOwned;

use crate::rest::request::request_object;

pub async fn get<T: DeserializeOwned>(url: impl ToString) -> Result<T> {
    request_object(super::Method::Get, url, BTreeMap::default(), None).await
}
