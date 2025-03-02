use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow as e, Result};
use binrw::BinRead;
use tokio::sync::Mutex;

use crate::server_address::{RawServerAddress, ServerAddress};
use crate::tinyudp;

/// Get server addresses from a single master server
///
/// # Example
///
/// ```
/// use std::time::Duration;
///
/// async fn test() {
///     let master = "master.quakeworld.nu:27000";
///     let timeout = Duration::from_secs(2);
///     match masterstat::server_addresses(&master, timeout).await {
///         Ok(result) => { println!("found {} server addresses", result.len()) },
///         Err(e) => { eprintln!("error: {}", e); }
///     }
/// }
/// ```
pub async fn server_addresses(
    master_address: &str,
    timeout: Duration,
) -> Result<Vec<ServerAddress>> {
    const STATUS_MSG: [u8; 3] = [99, 10, 0];
    let response = tinyudp::send_and_receive(
        master_address,
        &STATUS_MSG,
        tinyudp::Options {
            timeout,
            buffer_size: 64 * 1024, // 64 kb
        },
    )
    .await?;
    parse_servers_response(&response)
}

/// Get server addresses from many master servers (concurrently)
///
/// # Example
///
/// ```
/// use std::time::Duration;
///
/// async fn test() {
///     let masters = ["master.quakeworld.nu:27000", "master.quakeservers.net:27000"];
///     let timeout = Duration::from_secs(2);
///     let result = masterstat::server_addresses_from_many(&masters, timeout).await;
///     println!("found {} server addresses", result.len());
/// }
/// ```
pub async fn server_addresses_from_many(
    master_addresses: &[impl AsRef<str>],
    timeout: Duration,
) -> Vec<ServerAddress> {
    let mut task_handles = vec![];
    let result_mux = Arc::<Mutex<Vec<ServerAddress>>>::default();

    for master_address in master_addresses.iter().map(|a| a.as_ref().to_string()) {
        let result_mux = result_mux.clone();
        let task = tokio::spawn(async move {
            if let Ok(servers) = server_addresses(&master_address, timeout).await {
                let mut result = result_mux.lock().await;
                result.extend(servers);
            }
        });
        task_handles.push(task);
    }

    futures::future::join_all(task_handles).await;

    let server_addresses = result_mux.lock().await.clone();
    sorted_and_unique(&server_addresses)
}

fn parse_servers_response(response: &[u8]) -> Result<Vec<ServerAddress>> {
    const RESPONSE_HEADER: [u8; 6] = [255, 255, 255, 255, 100, 10];

    if !response.starts_with(&RESPONSE_HEADER) {
        return Err(e!("Invalid response"));
    }

    let body = &mut Cursor::new(&response[RESPONSE_HEADER.len()..]);
    let mut server_addresses = vec![];

    while let Ok(raw_address) = RawServerAddress::read(body) {
        server_addresses.push(ServerAddress::from(raw_address));
    }

    Ok(server_addresses)
}

fn sorted_and_unique(server_addresses: &[ServerAddress]) -> Vec<ServerAddress> {
    let mut servers = server_addresses.to_vec();
    servers.sort();
    servers.dedup();
    servers
}

#[cfg(test)]
mod tests {
    use super::*;
    // use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_server_addresses() -> Result<()> {
        let master = "master.quakeservers.net:27000";
        let timeout = Duration::from_secs(10);
        let result = server_addresses(master, timeout).await?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_server_addresses_from_many() -> Result<()> {
        let masters = [
            "master.quakeservers.net:27000",
            "master.quakeworld.nu:27000",
        ];
        let timeout = Duration::from_secs(10);
        let result = server_addresses_from_many(&masters, timeout).await;
        assert!(result.len() > 500);
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_servers_response() -> Result<()> {
        // invalid response header
        {
            let response = [0xff, 0xff];
            let result = parse_servers_response(&response);
            assert_eq!(result.unwrap_err().to_string(), "Invalid response");
        }

        // valid response
        {
            let response = [
                0xff, 0xff, 0xff, 0xff, 0x64, 0x0a, 192, 168, 1, 1, 0x75, 0x30, 192, 168, 1, 2,
                0x75, 0x30,
            ];
            let result = parse_servers_response(&response)?;
            assert_eq!(result.len(), 2);
            assert_eq!(result[0].ip, "192.168.1.1");
            assert_eq!(result[0].port, 30000);
            assert_eq!(result[1].ip, "192.168.1.2");
            assert_eq!(result[1].port, 30000);
        }

        Ok(())
    }

    #[test]
    fn test_sorted_and_unique() {
        let server1_1 = ServerAddress {
            ip: "192.168.1.1".to_string(),
            port: 1,
        };
        let server1_2 = ServerAddress {
            ip: "192.168.1.1".to_string(),
            port: 2,
        };
        let server3 = ServerAddress {
            ip: "192.168.1.3".to_string(),
            port: 1,
        };
        let server4 = ServerAddress {
            ip: "192.168.1.4".to_string(),
            port: 1,
        };
        let servers = vec![
            server4.clone(),
            server4.clone(),
            server4.clone(),
            server1_1.clone(),
            server1_2.clone(),
            server3.clone(),
        ];
        assert_eq!(
            sorted_and_unique(&servers),
            vec![server1_1, server1_2, server3, server4]
        );
    }
}
