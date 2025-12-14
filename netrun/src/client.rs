use std::net::SocketAddr;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{from_str, to_string};
use tarpc::{client, context, tokio_serde::formats::Json};

use crate::channel_service::ChannelServiceClient;

pub struct Client {
    addr: SocketAddr,
    cl:   ChannelServiceClient,
}

impl Client {
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        let transport = tarpc::serde_transport::tcp::connect(addr, Json::default);

        let cl = ChannelServiceClient::new(client::Config::default(), transport.await?).spawn();

        Ok(Self { addr, cl })
    }

    pub async fn send<In: Serialize, Out: DeserializeOwned>(&self, data: In) -> Result<Out> {
        let response = self.cl.send_data(context::current(), to_string(&data)?).await?;
        Ok(from_str(&response)?)
    }
}
