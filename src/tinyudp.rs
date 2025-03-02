use anyhow::Result;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;
use thiserror::Error;
use tokio::net::{ToSocketAddrs, UdpSocket};

#[derive(Debug, Error)]
pub enum TinyudpError {
    #[error("failed to bind socket: {0}")]
    BindFailed(#[source] std::io::Error),

    #[error("failed to send message: {0}")]
    SendFailed(#[source] std::io::Error),

    #[error("failed to receive message: {0}")]
    ReceiveFailed(#[source] std::io::Error),

    #[error("timeout reached while waiting for response")]
    TimeoutReached,
}

pub async fn send_and_receive(
    target: impl ToSocketAddrs,
    message: &[u8],
    options: Options,
) -> Result<Vec<u8>, TinyudpError> {
    let socket = bind().await?;
    socket
        .send_to(message, target)
        .await
        .map_err(TinyudpError::SendFailed)?;

    let mut buffer = vec![0; options.buffer_size];
    let (bytes_read, _) = tokio::select! {
        _ = tokio::time::sleep(options.timeout) => Err(TinyudpError::TimeoutReached),
        res = socket.recv_from(&mut buffer) => res.map_err(TinyudpError::ReceiveFailed),
    }?;

    let response = buffer[..bytes_read].to_vec();
    Ok(response)
}

async fn bind() -> Result<UdpSocket, TinyudpError> {
    let address = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    UdpSocket::bind(address)
        .await
        .map_err(TinyudpError::BindFailed)
}

#[derive(Debug)]
pub struct Options {
    pub timeout: Duration,
    pub buffer_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn test_send_and_receive() -> Result<()> {
        let response = send_and_receive(
            "quake.se:28501",
            b"\xff\xff\xff\xffstatus",
            Options {
                timeout: Duration::from_secs_f32(0.2),
                buffer_size: 32 * 1024,
            },
        )
        .await?;
        assert!(String::from_utf8_lossy(&response).contains("QUAKE.SE KTX"));
        Ok(())
    }
}
