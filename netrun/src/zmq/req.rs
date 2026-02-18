use std::{cell::RefCell, marker::PhantomData, sync::Arc};

use anyhow::{Result, anyhow};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::Mutex;
use zeromq::{ReqSocket, Socket, SocketRecv, SocketSend};

use crate::serde::{deserialize, serialize};

pub struct Req<In: Serialize + 'static, Out: DeserializeOwned + 'static> {
    socket: RefCell<ReqSocket>,
    _p: PhantomData<Arc<Mutex<(In, Out)>>>,
}

impl<In: Serialize + 'static, Out: DeserializeOwned + 'static> Req<In, Out> {
    pub async fn new(endpoint: &str) -> Result<Self> {
        let mut socket = ReqSocket::new();
        socket.connect(endpoint).await?;

        Ok(Self {
            socket: RefCell::new(socket),
            _p: PhantomData,
        })
    }

    pub async fn send(&self, input: In) -> Result<Out> {
        let data = serialize(input)?;
        let mut socket = self.socket.borrow_mut();

        socket.send(data.into()).await?;

        let data: Vec<u8> = socket.recv().await?.try_into().map_err(|err| anyhow!("{err}"))?;

        Ok(deserialize(&data)?)
    }
}
