use rand::Rng;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Error, Write};
use std::net::TcpStream;
use std::os::unix::io::{FromRawFd, RawFd};

pub struct Producer {
    writer: Option<File>,
    stream: Option<TcpStream>,
}

impl Producer {
    pub fn write(&mut self, int: i32) -> Result<(), Error> {
        let int_bytes = i32::to_be_bytes(int);
        if let Some(writer) = &mut self.writer {
            return writer.write_all(&int_bytes);
        }
        if let Some(stream) = &mut self.stream {
            let mut writer = BufWriter::new(stream.try_clone().unwrap());
            let s = format! {"{}\n", int};
            print!("sending {}", s);
            let _ = writer.write_all(&s.into_bytes());
            let _ = writer.flush();

            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut message = String::new();
            let _ = reader.read_line(&mut message);
            print!("message received: {}", message);
            return Ok(());
        }
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "failed to write data",
        ))
    }

    pub fn from_socket(addr: String) -> Result<Self, Error> {
        Ok(Self {
            writer: None,
            stream: Some(TcpStream::connect(addr).expect("failed to connect to the address")),
        })
    }

    pub fn from_fd(fd: RawFd) -> Result<Self, Error> {
        Ok(Self {
            writer: Some(unsafe { File::from_raw_fd(fd) }),
            stream: None,
        })
    }

    pub fn produce_random_ints(&mut self, int_numbers: i32) -> Result<(), Error> {
        let mut int = 1;
        let mut rng = rand::thread_rng();
        self.write(int).unwrap(); //.expect("failed to send first integer");
        for _ in 0..int_numbers {
            int += rng.gen_range(0..101);
            self.write(int).unwrap();
        }
        self.write(0).expect("failed to send 0");
        Ok(())
    }
}
