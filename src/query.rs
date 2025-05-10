use std::{io::Cursor, time::Duration};

use anyhow::{Result, anyhow as e};
use binrw::BinRead;

use crate::server_address::RawServerAddress;

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
///     match masterstat::query(&master, timeout).await {
///         Ok(addresses) => { println!("found {} server addresses", addresses.len()) },
///         Err(e) => { eprintln!("error: {}", e); }
///     }
/// }
/// ```
pub async fn query(master_address: &str, timeout: Duration) -> Result<Vec<String>> {
    const STATUS_MSG: [u8; 3] = [99, 10, 0];
    let response = tinyudp::send_and_receive(
        master_address,
        &STATUS_MSG,
        tinyudp::ReadOptions {
            timeout,
            buffer_size: 64 * 1024, // 64 kb
        },
    )
    .await?;
    parse_response(&response)
}

fn parse_response(response: &[u8]) -> Result<Vec<String>> {
    const RESPONSE_HEADER: [u8; 6] = [255, 255, 255, 255, 100, 10];

    if !response.starts_with(&RESPONSE_HEADER) {
        return Err(e!("Invalid response"));
    }

    let body = &mut Cursor::new(&response[RESPONSE_HEADER.len()..]);
    let mut addresses = vec![];

    while let Ok(raw_address) = RawServerAddress::read(body) {
        addresses.push(raw_address.to_string());
    }

    Ok(addresses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    #[cfg_attr(feature = "ci", ignore)]
    async fn test_query() -> Result<()> {
        let master = "master.quakeservers.net:27000";
        let timeout = Duration::from_secs(2);
        let result = query(master, timeout).await?;
        assert!(!result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_parse_response() -> Result<()> {
        // invalid response header
        {
            let response = [0xff, 0xff];
            let result = parse_response(&response);
            assert_eq!(result.unwrap_err().to_string(), "Invalid response");
        }

        // valid response
        {
            let response = [
                0xff, 0xff, 0xff, 0xff, 0x64, 0x0a, 192, 168, 1, 1, 0x75, 0x30, 192, 168, 1, 2,
                0x75, 0x30,
            ];
            let result = parse_response(&response)?;
            assert_eq!(result.len(), 2);
            assert_eq!(result[0], "192.168.1.1:30000".to_string());
            assert_eq!(result[1], "192.168.1.2:30000".to_string());
        }

        Ok(())
    }
}
