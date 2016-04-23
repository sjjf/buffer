use std::io;
use std::io::prelude::*;
use std::io::{stdin, stdout, stderr};
use std::io::{Error, ErrorKind};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// To implement the buffer in a relatively painless way I'll build it using a
// VecDeque, with a single mutex to protect push/pop operations. Push
// operations will grab the mutex, check the length against the maximum size,
// if there's space push the new data and then release the mutex; pop
// operations will grab the mutex, pop_front(), and then release the mutex. If
// the pop returns None then it spins in a reasonably tight loop (but with a
// small random delay to ensure that it doesn't block the push thread).

// return data, or None if we read EOF
//
// We could return a Result, but I don't think EOF is an error . . .
fn get_data() -> io::Result<Vec<u8>> {
    let mut data = vec![0; 1024];
    match stdin().read(&mut data[..]) {
        Ok(l) =>  match l {
                0 => Err(Error::new(ErrorKind::Other, "EoF")),
                _ => Ok(data),
        },
        Err(error) => {
            write!(stderr(), "Error! {}\n", error);
            return Err(error);
        }
    }
}

// write the whole buffer, repeating if necessary
fn put_data(data: Vec<u8>) -> io::Result<usize> {
    match stdout().write(&data[..]) {
        Ok(l) => Ok(l),
        Err(error) => {
            write!(stderr(), "Error! {}\n", error);
            Err(error)
        }
    }
}

fn main() {
    // The idea here is that we read bytes on one side into a buffer, and on
    // the other we write from the buffer. When the buffer has no data we don't
    // write anything.
    //
    // This requires non-blocking reads that feed as much data as is available
    // into the buffer, and non-blocking writes to stdout, connected by a tight
    // loop.
    //
    // Okay, since non-blocking I/O doesn't seem to be supported, I guess I'll
    // have to use separate read and write threads, connected by a protected
    // buffer . . .
    loop {
        match get_data() {
            Ok(data) =>  match put_data(data) {
                    Err(err) => write!(stderr(), "Write error! {}\n", err).unwrap(),
                    Ok(_) => {},
            },
            Err(error) => match error.kind() {
                ErrorKind::Other => {
                    write!(stderr(), "Nothing left!\n").unwrap();
                    break;
                },
                _ => write!(stderr(), "Unexpected Error {}", error).unwrap(),
            }
        }
    }
}
