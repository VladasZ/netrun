use core::net::SocketAddr;
use std::marker::PhantomData;

use anyhow::Result;
use log::{debug, error};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs, tcp::OwnedWriteHalf},
    select, spawn,
    sync::{
        Mutex, RwLock,
        mpsc::{Receiver, Sender, channel},
    },
};
use tokio_util::sync::CancellationToken;

use crate::{
    System,
    connection::{
        BUFFER_SIZE,
        serde::{deserialize, serialize},
    },
};

pub struct Client<In, Out> {
    write:    RwLock<OwnedWriteHalf>,
    receiver: Mutex<Receiver<Option<In>>>,
    cancel:   CancellationToken,
    _p:       PhantomData<Mutex<Out>>,
}

impl<In: DeserializeOwned + Send + 'static, Out: Serialize> Client<In, Out> {
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self::from_stream(TcpStream::connect(addr).await?))
    }

    pub fn from_stream(stream: TcpStream) -> Self {
        let addr = stream.local_addr().expect("Failed to get stream local_addr");
        let id = System::generate_app_instance_id();
        let cancel = CancellationToken::new();

        let (s, r) = channel::<Option<In>>(1);
        let (mut read, write) = stream.into_split();
        let cn = cancel.clone();

        let idd = id.clone();
        spawn(async move {
            let mut buf = vec![0u8; BUFFER_SIZE];

            loop {
                select! {
                    () = cn.cancelled() => {
                        debug!("Client dropped. Stop listening: {addr} - {idd}");
                        break
                    },
                    bytes = read.read(&mut buf) => handle_read(bytes, &buf, &s).await,
                }
            }
        });

        debug!("Connection: {id} created");

        Self {
            write: RwLock::new(write),
            receiver: Mutex::new(r),
            cancel,
            _p: PhantomData,
        }
    }

    pub async fn send(&self, val: impl Into<Out>) -> Result<()> {
        let val = val.into();
        let data = serialize(&val)?;

        self.write.write().await.write_all(&data).await?;

        Ok(())
    }

    pub async fn receive(&self) -> Option<In> {
        self.receiver.lock().await.recv().await.unwrap()
    }

    pub async fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.write.read().await.local_addr()?)
    }

    pub async fn peer_addr(&self) -> Result<SocketAddr> {
        Ok(self.write.read().await.peer_addr()?)
    }
}

impl<In, Out> Drop for Client<In, Out> {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

async fn handle_read<In: DeserializeOwned>(
    bytes: std::io::Result<usize>,
    buf: &[u8],
    sender: &Sender<Option<In>>,
) {
    let bytes = match bytes {
        Ok(b) => b,
        Err(err) => {
            error!("Failed to receive from client: {err}");
            _ = sender
                .send(None)
                .await
                .inspect_err(|e| error!("Failed to send None from client: {e}"));
            return;
        }
    };

    if bytes == 0 {
        return;
    }
    let Ok(msg) =
        deserialize::<In>(&buf[..bytes]).inspect_err(|e| error!("Failed to deserialize from client: {e}"))
    else {
        return;
    };

    _ = sender
        .send(Some(msg))
        .await
        .inspect_err(|e| error!("Failed to send msg from client: {e}"));
}
