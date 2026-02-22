use std::collections::HashMap;

use anyhow::Result;
use serde::de::DeserializeOwned;

use crate::rest::request::request_object;

pub async fn get<T: DeserializeOwned>(url: impl ToString) -> Result<T> {
    request_object(super::Method::Get, url, HashMap::default(), None).await
}
