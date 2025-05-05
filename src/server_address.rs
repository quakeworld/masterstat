use binrw::BinRead;
use std::fmt::Display;

#[cfg(feature = "json")]
use serde::{Serialize, Serializer};

#[derive(BinRead)]
#[br(big)]
pub(crate) struct RawServerAddress {
    ip: [u8; 4],
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl From<&ServerAddress> for String {
    fn from(addr: &ServerAddress) -> Self {
        addr.to_string()
    }
}

#[cfg(feature = "json")]
impl Serialize for ServerAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
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
    fn test_from_server_address_ref_for_string() {
        let address = ServerAddress {
            ip: "10.10.10.10".to_string(),
            port: 30000,
        };
        let address_str: String = String::from(&address);
        assert_eq!(address_str, "10.10.10.10:30000");
    }

    #[test]
    fn test_server_address_display() {
        let address = ServerAddress {
            ip: "192.168.1.1".to_string(),
            port: 30000,
        };
        assert_eq!(address.to_string(), "192.168.1.1:30000");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_serialize() -> Result<()> {
        assert_eq!(
            serde_json::to_string(&ServerAddress {
                ip: "10.10.10.10".to_string(),
                port: 30000,
            })?,
            r#""10.10.10.10:30000""#
        );

        Ok(())
    }
}
