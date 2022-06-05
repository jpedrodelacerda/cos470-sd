use bytes::Bytes;
use core::fmt;
use log::debug;
use std::io::Read;
use std::u8;
use std::{error::Error, io::ErrorKind};
use tokio::io::BufReader;
use tokio::io::{self, AsyncReadExt};

static SEP: &str = "|";
static SEPB: u8 = 124;

#[derive(PartialEq, Debug)]
pub enum MessageType {
    // Message sent by a process to request
    // access to Scheduler's critial region
    // Code: 1
    Request,
    // Message sent by the Scheduler
    // to a process to give access to
    // Scheduler's critial region
    // Code: 2
    Grant,
    // Message sent by a process
    // when it leaves Schedule's critical region
    // Code: 3
    Release,
    // Unknown message.
    // This should happen when the code is not found.
    // Code: >3
    Unknown,
}

#[derive(PartialEq, Debug)]
pub struct Message {
    message_type: MessageType,
    sender: u8,
}

impl Message {
    pub fn new(sender: u8, message_type: u8) -> Self {
        Self {
            message_type: MessageType::from(message_type),
            sender,
        }
    }

    pub fn from_string(string: String) -> Result<Self, Box<dyn Error>> {
        let mut split = string.split(SEP);

        let msg_code = match split.next() {
            Some(msg_code_str) => msg_code_str.parse::<u8>()?,
            None => todo!(),
        };
        let sender = match split.next() {
            Some(sender_str) => sender_str.parse::<u8>()?,
            None => todo!(),
        };

        Ok(Message::new(msg_code, sender))
    }

    pub fn from_bytes(raw_bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        println!("{:?}", raw_bytes);
        let message_utf8 = String::from_utf8(raw_bytes.to_vec())?;
        Self::from_string(message_utf8)
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{SEP}{}", self.message_type, self.sender)
    }
}

impl From<u8> for MessageType {
    fn from(code: u8) -> Self {
        return match code {
            1 => Self::Request,
            2 => Self::Grant,
            3 => Self::Release,
            _ => Self::Unknown,
        };
    }
}

impl MessageType {
    pub fn to_code(&self) -> u8 {
        return match self {
            Self::Request => 1,
            Self::Grant => 2,
            Self::Release => 3,
            Self::Unknown => 0,
        };
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Request => write!(f, "REQUEST"),
            Self::Grant => write!(f, "GRANT"),
            Self::Release => write!(f, "RELEASE"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    static expected: [Message; 4] = [
        Message {
            message_type: MessageType::Request,
            sender: 42,
        },
        Message {
            message_type: MessageType::Grant,
            sender: 42,
        },
        Message {
            message_type: MessageType::Release,
            sender: 42,
        },
        Message {
            message_type: MessageType::Unknown,
            sender: 42,
        },
    ];

    #[test]
    fn create_message() {
        let cases: Vec<(u8, u8)> = vec![(42, 1), (42, 2), (42, 3), (42, 42)];

        for (case_number, (msg_code, sender)) in cases.into_iter().enumerate() {
            let got = Message::new(msg_code, sender);
            assert_eq!(expected[case_number], got);
        }
    }

    #[test]
    fn create_message_from_bytes() {
        let cases: Vec<&[u8]> = vec![
            &[42, 124, 52, 50],
            &[42, 124, 52, 50],
            &[42, 124, 52, 50],
            &[42, 124, 52, 50],
        ];

        for (case_number, case) in cases.into_iter().enumerate() {
            let got = Message::from_bytes(case).unwrap();
            assert_eq!(expected[case_number], got);
        }
    }

    #[test]
    fn create_message_from_string() {
        init();
        let cases: Vec<String> = vec!["42|1".to_string()];
        for (case_number, case) in cases.into_iter().enumerate() {
            let got = Message::from_string(case).unwrap();
            assert_eq!(expected[case_number], got);
        }
    }
}
