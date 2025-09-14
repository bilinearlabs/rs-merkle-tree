use rand::RngCore;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node([u8; 32]);

impl Node {
    pub const LEN: usize = 32;
    pub const ZERO: Node = Node([0; Node::LEN]);
}

impl From<[u8; Node::LEN]> for Node {
    fn from(bytes: [u8; Node::LEN]) -> Self {
        Node(bytes)
    }
}

impl TryFrom<&str> for Node {
    type Error = String;

    fn try_from(hex: &str) -> Result<Self, Self::Error> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);

        if hex.len() != 64 {
            return Err("Hex string must be 64 characters long".to_string());
        }

        let mut bytes = [0u8; Node::LEN];
        for (i, chunk) in hex.chars().collect::<Vec<_>>().chunks(2).enumerate() {
            if i < Node::LEN {
                let byte_str: String = chunk.iter().collect();
                bytes[i] =
                    u8::from_str_radix(&byte_str, 16).map_err(|_| "Invalid hex character")?;
            }
        }

        Ok(Node(bytes))
    }
}

impl AsRef<[u8]> for Node {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "0x{}",
            self.0
                .iter()
                .map(|byte| format!("{:02x}", byte))
                .collect::<String>()
        )
    }
}

impl Node {
    pub fn random() -> Self {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes.into()
    }
}

#[macro_export]
macro_rules! to_node {
    ($hex:expr) => {{
        $crate::node::Node::try_from($hex).expect("Invalid node hex literal")
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node() {
        assert_eq!(
            to_node!("0x1230000000000000000000000000000000000000000000000000000000000000"),
            Node([
                0x12, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00
            ])
        );

        assert_eq!(
            Node::ZERO,
            Node([
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00
            ])
        );

        assert_eq!(
            format!(
                "{}",
                to_node!("0x760bde345debf3075c7fc0bcd2134e16ce5fc1a13adaa66ec6452a391f70595c")
            ),
            "0x760bde345debf3075c7fc0bcd2134e16ce5fc1a13adaa66ec6452a391f70595c"
        );

        assert_eq!(
            Node::from([
                0x76, 0x0b, 0xde, 0x34, 0x5d, 0xeb, 0xf3, 0x07, 0x5c, 0x7f, 0xc0, 0xbc, 0xd2, 0x13,
                0x4e, 0x16, 0xce, 0x5f, 0xc1, 0xa1, 0x3a, 0xda, 0xa6, 0x6e, 0xc6, 0x45, 0x2a, 0x39,
                0x1f, 0x70, 0x59, 0x5c
            ]),
            to_node!("0x760bde345debf3075c7fc0bcd2134e16ce5fc1a13adaa66ec6452a391f70595c")
        );
    }
}
