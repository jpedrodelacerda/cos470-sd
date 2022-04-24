use std::fs::File;
use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::io::{FromRawFd, RawFd};
use std::str::FromStr;

pub struct Consumer {
    reader: Option<Box<dyn Read>>, //std::io::Write>,
    listener: Option<TcpListener>,
}

impl Consumer {
    pub fn from_socket(addr: String) -> Result<Self, Error> {
        println!("starting consumer on {}", addr);
        let listener = TcpListener::bind(addr)?;
        Ok(Self {
            reader: None,
            listener: Some(listener),
        })
    }

    pub fn from_fd(fd: RawFd) -> Result<Self, Error> {
        Ok(Self {
            reader: Some(unsafe { Box::new(File::from_raw_fd(fd)) }),
            listener: None,
        })
    }

    pub fn read(&mut self) -> Result<(), Error> {
        if let Some(reader) = &mut self.reader {
            let mut buffer = [0; 4];
            let _ = reader.read(&mut buffer)?;
            let int = i32::from_be_bytes(buffer);
            if int == 0 {
                finish_consumer()
            }
            if is_prime(int) {
                println!("{} is prime", int);
            } else {
                println!("{} is not prime", int);
            };
        }
        if let Some(listener) = &mut self.listener {
            for stream in listener.incoming() {
                process_stream(&mut stream?);
            }
        }
        Ok(())
    }
}

fn process_stream(stream: &mut TcpStream) {
    let reader = BufReader::new(stream.try_clone().unwrap());
    for line in reader.lines() {
        let message = line.unwrap();
        let int = i32::from_str(&message).unwrap();
        if int == 0 {
            let _ = write!(stream, "finishing consumer when 0 is consumed.\n");
            finish_consumer();
        }
        if is_prime(int) {
            let answer = format!("{} is prime\n", int);
            send_answer(stream, answer)
        } else {
            let answer = format!("{} is not prime\n", int);
            send_answer(stream, answer)
        }
    }
}

fn send_answer(stream: &mut TcpStream, answer: String) {
    print!("{}", answer);
    let _ = stream
        .write_all(answer.as_bytes())
        .expect("failed to send answer");
    let _ = stream.flush();
}

fn finish_consumer() {
    println!("Received 0. Ending consumer.");
    std::process::exit(0);
}

fn is_prime(int: i32) -> bool {
    for den in 2..(int / 2) {
        if int % den == 0 {
            return false;
        }
    }
    true
}
