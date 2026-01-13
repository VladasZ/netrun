use core::net::SocketAddr;
use std::marker::PhantomData;

use anyhow::Result;
use log::error;
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs, tcp::OwnedWriteHalf},
    select, spawn,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};
use tokio_util::sync::CancellationToken;

use crate::connection::{
    BUFFER_SIZE,
    serde::{deserialize, serialize},
};

pub struct Client<In, Out> {
    write:    Mutex<OwnedWriteHalf>,
    receiver: Mutex<Receiver<In>>,
    cancel:   CancellationToken,
    _p:       PhantomData<Mutex<Out>>,
}

impl<In: DeserializeOwned + Send + 'static, Out: Serialize> Client<In, Out> {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self::from_stream(TcpStream::connect(addr).await?))
    }

    pub fn from_stream(stream: TcpStream) -> Self {
        let cancel = CancellationToken::new();

        let (s, r) = channel::<In>(1);
        let (mut read, write) = stream.into_split();
        let cn = cancel.clone();

        spawn(async move {
            let mut buf = vec![0u8; BUFFER_SIZE];

            loop {
                select! {
                    () = cn.cancelled() => break,
                    bytes = read.read(&mut buf) => handle_read(bytes, &buf, &s).await,
                }
            }
        });

        Self {
            write: Mutex::new(write),
            receiver: Mutex::new(r),
            cancel,
            _p: PhantomData,
        }
    }

    pub async fn send(&self, val: impl Into<Out>) -> Result<()> {
        let val = val.into();
        let data = serialize(&val)?;

        self.write.lock().await.write_all(&data).await?;

        Ok(())
    }

    pub async fn receive(&self) -> Option<In> {
        self.receiver.lock().await.recv().await
    }

    pub async fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.write.lock().await.local_addr()?)
    }

    pub async fn peer_addr(&self) -> Result<SocketAddr> {
        Ok(self.write.lock().await.peer_addr()?)
    }
}

impl<In, Out> Drop for Client<In, Out> {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

async fn handle_read<In: DeserializeOwned>(bytes: std::io::Result<usize>, buf: &[u8], sender: &Sender<In>) {
    let Ok(bytes) = bytes.inspect_err(|e| error!("Failed to receive from client: {e}")) else {
        return;
    };

    if bytes == 0 {
        return;
    }

    let Ok(msg) =
        deserialize(&buf[..bytes]).inspect_err(|e| error!("Failed to deserialize from client: {e}"))
    else {
        return;
    };

    _ = sender
        .send(msg)
        .await
        .inspect_err(|e| error!("Failed to send from client: {e}"));
}
