use std::{
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use anyhow::Result;
use log::{debug, error, trace};
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

pub struct Server<In, Out> {
    cancel:    CancellationToken,
    connected: Mutex<Receiver<Client<In, Out>>>,
    _p:        PhantomData<Mutex<(In, Out)>>,
}

impl<
    In: Serialize + DeserializeOwned + Send + 'static,
    Out: Serialize + DeserializeOwned + Clone + Send + 'static,
> Server<In, Out>
{
    pub async fn new(port: u16) -> Result<Self> {
        let listener = TcpListener::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)).await?;

        let cancel = CancellationToken::new();

        let cn = cancel.clone();

        let (s, r) = channel(1);

        spawn(async move {
            loop {
                select! {
                    () = cn.cancelled() => {
                        debug!("Stopping server listening on: {port}");
                        break;
                    }
                    connection = listener.accept() => {
                        match connection {
                            Ok((stream, _)) => Self::add_connection(&s, stream).await,
                            Err(err) => error!("Failed to accept connection: {err}"),
                        }
                    }
                }
            }
        });

        Ok(Self {
            cancel,
            connected: Mutex::new(r),
            _p: PhantomData,
        })
    }

    pub async fn wait_for_new_connection(&self) -> Client<In, Out> {
        self.connected.lock().await.recv().await.expect("Dropped server")
    }

    async fn add_connection(new_connection: &Sender<Client<In, Out>>, stream: TcpStream) {
        trace!("New connection");

        let connection = Client::from_stream(stream);

        if let Err(err) = new_connection.send(connection).await {
            error!("Failed to send connection signal: {err}");
        }
    }
}

impl<In, Out> Drop for Server<In, Out> {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
