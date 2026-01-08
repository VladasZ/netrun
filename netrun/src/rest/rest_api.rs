use std::{collections::HashMap, fmt::Display, sync::OnceLock};

use parking_lot::Mutex;

static STATIC_API: OnceLock<RestAPI> = OnceLock::new();

#[derive(Debug)]
pub struct RestAPI {
    base_url: String,
    headers:  Mutex<HashMap<String, String>>,
}

impl RestAPI {
    pub fn init(base_url: impl Display) {
        _ = STATIC_API
            .set(Self {
                base_url: format!("{base_url}"),
                headers:  Mutex::default(),
            })
            .inspect_err(|err| log::error!("err: {err:?}"));

        Self::clear_all_headers();
    }

    pub fn is_ok() -> bool {
        STATIC_API.get().is_some()
    }

    fn get() -> &'static Self {
        STATIC_API.get().expect("API is not initialised. Use API::init(\"base_url\")")
    }
}

impl RestAPI {
    pub fn base_url() -> &'static str {
        &Self::get().base_url
    }

    pub fn headers() -> HashMap<String, String> {
        Self::get().headers.lock().clone()
    }

    pub fn remove_header(key: impl ToString) {
        Self::get().headers.lock().remove(&key.to_string());
    }

    pub fn clear_all_headers() {
        Self::get().headers.lock().clear();
    }

    pub fn add_header(key: impl ToString, value: impl ToString) {
        Self::get().headers.lock().insert(key.to_string(), value.to_string());
    }

    pub fn set_access_token(token: impl ToString) {
        Self::add_header("token", token);
    }
}
