use binrw::BinRead;
use std::fmt::Display;

#[derive(Debug, BinRead, PartialEq)]
#[br(big)]
pub(crate) struct RawServerAddress {
    ip: [u8; 4],
    port: u16,
}

impl Display for RawServerAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ip_str = self.ip.map(|b| b.to_string()).join(".");
        write!(f, "{}:{}", ip_str, self.port)
    }
}

#[cfg(test)]
mod tests {
    use crate::server_address::RawServerAddress;
    use binrw::BinRead;
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    #[test]
    fn test_read() {
        assert_eq!(
            RawServerAddress::read(&mut Cursor::new(&[192, 168, 1, 1, 117, 48])).unwrap(),
            RawServerAddress {
                ip: [192, 168, 1, 1],
                port: 30000
            }
        );
    }

    #[test]
    fn test_display() {
        let address = RawServerAddress {
            ip: [192, 168, 1, 1],
            port: 30000,
        };
        assert_eq!(address.to_string(), "192.168.1.1:30000".to_string());
    }
}
