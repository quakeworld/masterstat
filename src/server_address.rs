use binrw::BinRead;
use std::fmt::Display;

#[derive(BinRead)]
#[br(big)]
pub struct RawServerAddress {
    pub ip: [u8; 4],
    pub port: u16,
}

#[derive(Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub struct ServerAddress {
    pub ip: String,
    pub port: u16,
}

impl Display for ServerAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

impl From<RawServerAddress> for ServerAddress {
    fn from(raw: RawServerAddress) -> Self {
        ServerAddress {
            ip: raw.ip.map(|b| b.to_string()).join("."),
            port: raw.port,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server_address::{RawServerAddress, ServerAddress};
    use anyhow::Result;
    use binrw::BinRead;
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    #[test]
    fn test_raw_server_address() -> Result<()> {
        let raw_address = RawServerAddress::read(&mut Cursor::new(&[192, 168, 1, 1, 117, 48]))?;
        assert_eq!(raw_address.ip, [192, 168, 1, 1]);
        assert_eq!(raw_address.port, 30000);
        Ok(())
    }

    #[test]
    fn test_server_address_from_raw_server_address() {
        let raw_address = RawServerAddress {
            ip: [192, 168, 1, 1],
            port: 30000,
        };
        let address = ServerAddress::from(raw_address);
        assert_eq!(address.ip, "192.168.1.1");
        assert_eq!(address.port, 30000);
    }

    #[test]
    fn test_server_address_display() {
        let address = ServerAddress {
            ip: "192.168.1.1".to_string(),
            port: 30000,
        };
        assert_eq!(address.to_string(), "192.168.1.1:30000");
    }
}
