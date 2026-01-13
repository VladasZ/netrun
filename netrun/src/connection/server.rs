use std::{
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use anyhow::Result;
use log::{debug, error, warn};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{
    net::{TcpListener, TcpStream},
    select, spawn,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};
use tokio_util::sync::CancellationToken;

use crate::connection::Client;

type Connections<In, Out> = Arc<Mutex<Vec<Client<In, Out>>>>;

pub struct Server<In, Out> {
    connections: Connections<In, Out>,
    cancel:      CancellationToken,
    connected:   Mutex<Receiver<()>>,
    _p:          PhantomData<Mutex<(In, Out)>>,
}

impl<In: DeserializeOwned + Send + 'static, Out: Serialize + Clone + Send + 'static> Server<In, Out> {
    pub async fn new(port: u16) -> Result<Self> {
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)).await?;

        let connections = Arc::new(Mutex::new(vec![]));
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
        let connections = self.connections.lock().await;

        if connections.is_empty() {
            warn!("Sending message to server without connections");
            return Ok(());
        }

        for conn in connections.iter() {
            let msg = msg.clone();
            conn.send(msg).await?;
        }

        Ok(())
    }

    pub async fn wait_for_connection(&self) {
        self.connected.lock().await.recv().await;
    }

    pub async fn receive(&self) -> Option<In> {
        if self.connections.lock().await.is_empty() {
            self.wait_for_connection().await;
        }

        let conn = self.connections.lock().await;

        conn.first().unwrap().receive().await
    }

    pub async fn dump_connections(&self) -> Result<()> {
        for conn in self.connections.lock().await.iter() {
            dbg!(conn.peer_addr().await?);
            dbg!(conn.local_addr().await?);
        }

        Ok(())
    }

    // TODO: handle more that 1 receiver
    async fn add_connection(connections: &Connections<In, Out>, sender: &Sender<()>, stream: TcpStream) {
        let mut connections = connections.lock().await;

        debug!("Connected: {}", stream.local_addr().unwrap());

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
