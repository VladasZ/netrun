use std::{fmt::Debug, ops::Deref};

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::connection::Connection;

pub struct Client<T> {
    connection: Connection<T>,
}

impl<T: Serialize + DeserializeOwned + Send + Debug> Client<T> {
    pub async fn new(address: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self {
            connection: Connection::new(TcpStream::connect(address).await?),
        })
    }
}

impl<T> Deref for Client<T> {
    type Target = Connection<T>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
