use std::fs::File;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{FromRawFd, RawFd};

pub struct Consumer {
    reader: Box<dyn Read>, //std::io::Write>,
    writer: Option<Box<dyn Write>>,
}

impl Consumer {
    pub fn from_fd(fd: RawFd) -> Result<Self, Error> {
        Ok(Self {
            reader: unsafe { Box::new(File::from_raw_fd(fd)) },
            writer: None,
        })
    }

    pub fn read(&mut self) -> Result<usize, Error> {
        let mut buffer = [0; 4];
        let n = self.reader.read(&mut buffer)?;
        let int = i32::from_be_bytes(buffer);
        if int == 0 {
            println!("Received 0. Ending consumer");
            std::process::exit(0);
        }
        is_prime(int);
        Ok(n)
    }
}

fn is_prime(int: i32) -> bool {
    for den in 2..(int / 2) {
        if int % den == 0 {
            println!("{} is not prime", int);
            return false;
        }
    }
    println!("{} is prime", int);
    true
}
