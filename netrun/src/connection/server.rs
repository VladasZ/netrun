use std::{
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use anyhow::Result;
use log::{error, info, warn};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    net::{TcpListener, TcpStream},
    select, spawn,
    sync::{
        Mutex, RwLock,
        mpsc::{Receiver, Sender, channel},
    },
};
use tokio_util::sync::CancellationToken;

use crate::connection::Client;

type Connections<In, Out> = Arc<RwLock<Vec<Client<In, Out>>>>;

pub struct Server<In, Out> {
    connections: Connections<In, Out>,
    cancel:      CancellationToken,
    connected:   Mutex<Receiver<()>>,
    _p:          PhantomData<Mutex<(In, Out)>>,
}

impl<In: DeserializeOwned + Send + 'static, Out: Serialize + Clone + Send + 'static> Server<In, Out> {
    pub async fn new(port: u16) -> Result<Self> {
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)).await?;

        let connections = Arc::new(RwLock::new(vec![]));
        let cancel = CancellationToken::new();

        let conn = connections.clone();
        let cn = cancel.clone();

        let (s, r) = channel(1);

        spawn(async move {
            loop {
                select! {
                    () = cn.cancelled() => {
                        break;
                    }
                    connection = listener.accept() => {
                        match connection {
                            Ok((stream, _)) => Self::add_connection(&conn, &s, stream).await,
                            Err(err) => error!("Failed to accept connection: {err}"),
                        }
                    }
                }
            }
        });

        Ok(Self {
            connections,
            cancel,
            connected: Mutex::new(r),
            _p: PhantomData,
        })
    }

    pub async fn send(&self, msg: impl Into<Out>) -> Result<()> {
        let msg = msg.into();
        let connections = self.connections.read().await;

        if connections.is_empty() {
            warn!("Sending message to server without connections");
            return Ok(());
        }

        for conn in connections.iter() {
            let msg = msg.clone();
            let result = conn.send(msg).await;

            if let Err(err) = result
                && let Some(io_err) = err.downcast_ref::<std::io::Error>()
                && io_err.kind() == std::io::ErrorKind::BrokenPipe
            {
                let a = conn.peer_addr().await?;

                info!("Broken pipe! Removing connection: {a}");
                // trigger cleanup logic here
            }
        }

        Ok(())
    }

    pub async fn wait_for_new_connection(&self) {
        self.connected.lock().await.recv().await;
    }

    pub async fn receive(&self) -> In {
        loop {
            if self.connections.read().await.is_empty() {
                self.wait_for_new_connection().await;
            }

            let conns = self.connections.read().await;
            let conn = conns.first().unwrap();

            if let Some(val) = conn.receive().await {
                return val;
            }

            info!("Removing failed connection from server.");

            drop(conns);

            self.connections.write().await.clear();
        }
    }

    pub async fn connections(&self) -> Result<Vec<SocketAddr>> {
        let conn = self.connections.read().await;

        let mut res = vec![];

        for conn in conn.iter() {
            res.push(conn.peer_addr().await?);
        }

        Ok(res)
    }

    // TODO: handle more that 1 receiver
    async fn add_connection(connections: &Connections<In, Out>, sender: &Sender<()>, stream: TcpStream) {
        let mut connections = connections.write().await;

        info!("Connected: {}", stream.local_addr().unwrap());

        connections.clear();

        connections.push(Client::from_stream(stream));
        if let Err(err) = sender.send(()).await {
            error!("Failed to send connection signal: {err}");
        }
    }
}

impl<In, Out> Drop for Server<In, Out> {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
