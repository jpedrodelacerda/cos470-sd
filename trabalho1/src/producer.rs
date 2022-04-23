use rand::Rng;
use std::fs::File;
use std::io::{Error, Write};
use std::net::TcpStream;
use std::os::unix::io::{FromRawFd, RawFd};

pub struct Producer {
    writer: Box<dyn Write>, //std::io::Write>,
}

impl Producer {
    pub fn write(&mut self, int: i32) -> Result<usize, Error> {
        // write!(self.writer, "{}", int)
        self.writer.write(&i32::to_be_bytes(int))
    }

    pub fn from_socket(addr: String) -> Result<Self, Error> {
        Ok(Self {
            writer: Box::new(TcpStream::connect(addr).expect("failed to connect to the address")),
        })
    }

    pub fn from_fd(fd: RawFd) -> Result<Self, Error> {
        Ok(Self {
            writer: unsafe { Box::new(File::from_raw_fd(fd)) },
        })
    }

    pub fn produce_random_ints(&mut self, int_numbers: i32) -> Result<(), Error> {
        let mut int = 1;
        let mut rng = rand::thread_rng();
        self.write(int).expect("failed to send first integer");
        for i in 0..int_numbers {
            int += rng.gen_range(0..101);
            self.write(int)?;
        }
        self.write(0).expect("failed to send 0");
        Ok(())
    }
}
