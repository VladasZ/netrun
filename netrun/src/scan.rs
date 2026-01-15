use std::{
    iter::once,
    net::{IpAddr, Ipv4Addr},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use anyhow::Result;
use log::error;
use rust_network_scanner::{NetworkScanner, PortScanResult};
use tokio::{
    spawn,
    sync::{Mutex, mpsc::channel},
    time::{Duration, timeout},
};

fn local_network() -> Vec<IpAddr> {
    once(Ipv4Addr::LOCALHOST.into())
        .chain((1..255).map(|i| Ipv4Addr::new(192, 168, 0, i).into()))
        .collect()
}

async fn check_port(ip: IpAddr, port: u16, scanner: &NetworkScanner) -> Result<Vec<PortScanResult>> {
    let timeout_duration = Duration::from_millis(200);

    let Ok(scan) = timeout(timeout_duration, scanner.scan_ports(ip, port, port)).await else {
        return Ok(vec![]);
    };

    Ok(scan?.open_ports)
}

pub async fn scan_for_port(port: u16) -> Result<Vec<(IpAddr, Vec<PortScanResult>)>> {
    let local = local_network();

    let result = Arc::new(Mutex::new(vec![]));
    let finished_counter = Arc::new(AtomicUsize::new(local.len() - 1));
    let (s, mut r) = channel::<()>(1);

    let s = Arc::new(s);

    for ip in local {
        let counter = finished_counter.clone();
        let ss = s.clone();
        let res = result.clone();
        spawn(async move {
            let scanner = NetworkScanner::new();

            let open_ports = check_port(ip, port, &scanner)
                .await
                .inspect_err(|err| error!("Failed to check port: {err}"))
                .unwrap();

            if !open_ports.is_empty() {
                res.lock().await.push((ip, open_ports));
            }

            if counter.fetch_sub(1, Ordering::Relaxed) == 0 {
                ss.send(()).await.unwrap();
            }
        });
    }

    r.recv().await;

    let result = Arc::try_unwrap(result).expect("Failed to extract port results from Arc");

    Ok(result.into_inner())
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use rust_network_scanner::PortStatus;
    use test_log::test;

    use super::*;
    use crate::Server;

    #[test(tokio::test)]
    async fn test_scan() -> Result<()> {
        let _server: Server<(), ()> = Server::new(57779).await?;

        let (ip, open_ports) = scan_for_port(57779).await?.into_iter().next().unwrap();

        let open_port = open_ports.first().unwrap();

        assert_eq!(IpAddr::V4(Ipv4Addr::LOCALHOST), ip);
        assert_eq!(open_port.port, 57779);
        assert_eq!(open_port.status, PortStatus::Open);

        Ok(())
    }

    #[ignore]
    #[test(tokio::test)]
    async fn find() -> Result<()> {
        dbg!(scan_for_port(55400).await?);

        Ok(())
    }
}
