use core::fmt;
use log::{debug, warn};
use std::{error::Error, io::ErrorKind};
use std::{io, u8};

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
    pub message_type: MessageType,
    pub sender: u8,
}

impl Message {
    pub fn new(sender: u8, message_type: MessageType) -> Self {
        Self {
            message_type,
            sender,
        }
    }
    pub fn message_type(&self) -> &MessageType {
        &self.message_type
    }

    pub fn from_string(message: String) -> Result<Self, Box<dyn Error>> {
        debug!("[MESSAGE] Trimming newline");
        let msg_string = message.as_str();
        debug!("[MESSAGE] Trimmed message: {}", &msg_string);
        let mut split = msg_string.split(SEP);

        debug!("[MESSAGE] Parsing message: {}", &msg_string);
        debug!("[MESSAGE] [CODE]: Parsing");
        let msg_code = match split.next() {
            Some(msg_code_str) => msg_code_str.parse::<u8>()?,
            None => {
                return Err(Box::new(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "failed to parse message code",
                )));
            }
        };
        debug!("[MESSAGE] [CODE] Result: {}", msg_code);
        debug!("[MESSAGE] [SENDER]: Parsing");
        let sender = match split.next() {
            Some(sender_str) => sender_str.parse::<u8>()?,
            None => {
                return Err(Box::new(std::io::Error::new(
                    ErrorKind::InvalidData,
                    "failed to parse sender code",
                )))
            }
        };
        debug!("[MESSAGE] [SENDER] Result: {}", sender);
        debug!("[MESSAGE] Parsing finished.");

        Ok(Message::new(sender, msg_code.into()))
    }

    pub fn from_bytes(raw_bytes: &[u8]) -> Result<Self, Box<dyn Error>> {
        debug!("[MESSAGE] raw bytes: {:?}", raw_bytes);
        let mut bytes = raw_bytes.to_vec();
        loop {
            match bytes.last() {
                // Trim padding bytes
                Some(0) => bytes.pop(),
                Some(10) => bytes.pop(),
                _ => break,
            };
        }
        let mut iter = bytes.split(|num| *num == SEPB);

        debug!("[MESSAGE] [BYTES] [CODE] Parsing code");
        let msg_code_bytes = match iter.next() {
            Some(code) => code,
            None => {
                warn!("[MESSAGE] [CODE] failed parsing");
                return Err(Box::new(io::Error::new(
                    ErrorKind::InvalidData,
                    "Failed to parse message code",
                )));
            }
        };
        debug!(
            "[MESSAGE] [BYTES] [CODE] Parsing result: {:?}",
            msg_code_bytes
        );
        debug!("[MESSAGE] [BYTES] [SENDER] Parsing code");
        let msg_sender_bytes = match iter.next() {
            Some(sender) => sender,
            None => {
                warn!("[MESSAGE] [SENDER] failed parsing");
                return Err(Box::new(io::Error::new(
                    ErrorKind::InvalidData,
                    "Failed to parse message sender",
                )));
            }
        };
        debug!(
            "[MESSAGE] [BYTES] [SENDER] Parsing result: {:?}",
            msg_sender_bytes
        );

        let code_str = std::str::from_utf8(msg_code_bytes)?;
        let sender_str = std::str::from_utf8(msg_sender_bytes)?;

        let code = code_str.parse::<u8>()?;
        let sender = sender_str.parse::<u8>()?;

        Ok(Self::new(sender, code.into()))
    }

    pub fn to_bytes(&self) -> Result<[u8; 10], Box<dyn Error>> {
        let string = format!("{}|{}", self.message_type.to_code(), self.sender);
        debug!(
            "[MESSAGE] [BYTES] Message string to be sent = {}",
            string.to_owned()
        );
        let mut bytes = [0; 10];
        let string_bytes = string.as_bytes();
        if string_bytes.len() > 10 {
            return Err(Box::new(io::Error::new(
                ErrorKind::InvalidData,
                "[MESSAGE] Too large",
            )));
        }
        for i in 0..string_bytes.len() {
            bytes[i] = string_bytes[i]
        }
        Ok(bytes)
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
            Self::Unknown => 42,
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

    static EXPECTED: [Message; 4] = [
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
        let cases: Vec<(u8, MessageType)> = vec![
            (42, MessageType::Request),
            (42, MessageType::Grant),
            (42, MessageType::Release),
            (42, MessageType::Unknown),
        ];

        for (case_number, (msg_code, sender)) in cases.into_iter().enumerate() {
            let got = Message::new(msg_code, sender);
            assert_eq!(EXPECTED[case_number], got);
        }
    }

    #[test]
    fn create_message_from_bytes() {
        let cases: Vec<&[u8; 10]> = vec![
            &[49, 124, 52, 50, 10, 0, 0, 0, 0, 0],
            &[50, 124, 52, 50, 10, 0, 0, 0, 0, 0],
            &[51, 124, 52, 50, 10, 0, 0, 0, 0, 0],
            &[52, 50, 124, 52, 50, 10, 0, 0, 0, 0],
        ];

        for (case_number, case) in cases.into_iter().enumerate() {
            let got = Message::from_bytes(case).unwrap();
            assert_eq!(EXPECTED[case_number], got);
        }
    }

    #[test]
    fn create_message_from_string() {
        init();
        let cases: Vec<&str> = vec!["1|42", "2|42", "3|42", "42|42"];
        for (case_number, case) in cases.into_iter().enumerate() {
            let got = Message::from_string(case.to_string()).unwrap();
            assert_eq!(EXPECTED[case_number], got);
        }
    }
}
