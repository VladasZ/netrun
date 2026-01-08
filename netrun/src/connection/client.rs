use std::ops::Deref;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use tokio::net::{TcpStream, ToSocketAddrs};

use crate::connection::Connection;

pub struct Client<In, Out> {
    connection: Connection<In, Out>,
}

impl<In: DeserializeOwned + Send, Out: Serialize + Send> Client<In, Out> {
    pub async fn new(address: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self {
            connection: Connection::new(TcpStream::connect(address).await?),
        })
    }
}

impl<In, Out> Deref for Client<In, Out> {
    type Target = Connection<In, Out>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}
