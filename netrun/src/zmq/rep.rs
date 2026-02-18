use std::any::type_name;

use anyhow::Result;
use hreads::spawn;
use log::error;
use serde::{Serialize, de::DeserializeOwned};
use zeromq::RepSocket;
use zeromq::Socket;
use zeromq::SocketRecv;
use zeromq::SocketSend;

use crate::{
    Function,
    serde::{deserialize, serialize},
};

pub struct Rep<In: DeserializeOwned + 'static, Out: Serialize + 'static> {
    receive: Function<In, Out>,
}

impl<In: DeserializeOwned + 'static, Out: Serialize + 'static> Rep<In, Out> {
    pub async fn new(endpoint: &str) -> Result<Self> {
        let mut socket = RepSocket::new();
        socket.bind(endpoint).await?;

        let function = Function::default();
        let receive = function.clone();

        spawn(async move {
            loop {
                let mut repl: String = socket.recv().await.unwrap().try_into().unwrap();
                println!("Received: {:?}", repl);
                repl.push_str(" Reply");
                socket.send(repl.into()).await.unwrap();

                // let bytes = match socket.recv_bytes(0) {
                //     Ok(bytes) => bytes,
                //     Err(err) => {
                //         error!("Failed to receive in {}: {err}", type_name::<Self>());
                //         continue;
                //     }
                // };

                // let input: In = match deserialize(&bytes) {
                //     Ok(input) => input,
                //     Err(err) => {
                //         error!(
                //             "Failed to parse input packet for: {}. Error:\n{err}",
                //             type_name::<Self>(),
                //         );

                //         let _ = socket.send("Error", 0).inspect_err(|err| {
                //             error!("{err}");
                //         });

                //         continue;
                //     }
                // };

                // let output = function.call(input);

                // let out_data = match serialize(&output) {
                //     Ok(output) => output,
                //     Err(err) => {
                //         error!(
                //             "Failed to serialize output packet for: {}. Error:\n{err}",
                //             type_name::<Self>(),
                //         );

                //         let _ = socket.send("Error", 0).inspect_err(|err| {
                //             error!("{err}");
                //         });

                //         continue;
                //     }
                // };

                // let _ = socket.send(out_data, 0).inspect_err(|err| {
                //     error!("Failed to send response: {err}");
                // });
            }
        });

        Ok(Self { receive })
    }

    pub fn on_receive(&self, action: impl FnMut(In) -> Out + Send + 'static) {
        self.receive.replace(action);
    }
}

#[cfg(test)]
mod test {


    use anyhow::Result;

    use crate::zmq::Rep;

    #[tokio::test]
    async fn test_rep() -> Result<()> {
        let rep = Rep::<i32, i32>::new("tcp://0.0.0.0:6969").await?;

        rep.on_receive(|val| val * 2);

        loop {}

        // sleep(Duration::from_secs_f32(1.0)).await;

        Ok(())
    }
}
