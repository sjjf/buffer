use std::io;
use std::io::prelude::*;
use std::io::{stdin, stdout, stderr};
use std::io::{Error, ErrorKind};
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

// To implement the buffer in a relatively painless way I'll build it using a
// VecDeque, with a single mutex to protect push/pop operations. Push
// operations will grab the mutex, check the length against the maximum size,
// if there's space push the new data and then release the mutex; pop
// operations will grab the mutex, pop_front(), and then release the mutex. If
// the pop returns None then it yields to (hopefully) allow the reader thread
// to get more data

fn get_data() -> io::Result<Vec<u8>> {
    let mut data = vec![0; 1024];
    // may block
    match stdin().read(&mut data[..]) {
        Ok(l) =>  match l {
            0 => Err(Error::new(ErrorKind::Other, "EoF")),
            len => {
                data.truncate(len);
                Ok(data)
            },
        },
        Err(error) => {
            write!(stderr(), "Error! {}\n", error);
            return Err(error);
        }
    }
}

// write the whole buffer, repeating if necessary
fn put_data(data: Vec<u8>) -> io::Result<usize> {
    // may block, but probably shouldn't
    match stdout().write(&data[..]) {
        Ok(l) => Ok(l),
        Err(error) => {
            write!(stderr(), "Error! {}\n", error);
            Err(error)
        }
    }
}

#[derive(Clone)]
enum QueueState {
    Incomplete,
    Complete,
}

// In something like C/C++/Python/whatever I'd embed the lock in the structure,
// but I can't figure out how to do this in rust. That would allow me to keep
// all the lock handling in the Queue impl block. Without it I have to do the
// lock handling in the main code.
struct Queue {
    state: QueueState,
    data: VecDeque<Vec<u8>>,
}

impl Queue {

    fn new() -> Queue {
        let q = Queue {
            state: QueueState::Incomplete,
            data: VecDeque::new(),
        };
        q
    }
}

fn main() {
    // The idea here is that we read bytes on one side into a buffer, and on
    // the other we write from the buffer. When the buffer has no data we don't
    // write anything.
    //
    // Since non-blocking I/O doesn't seem to be supported in rust, I guess I'll
    // have to use separate read and write threads, connected by a protected
    // buffer . . .
    let queue = Arc::new(Mutex::new(Queue::new()));
    {
        let queue = queue.clone();
        thread::spawn(move || {
            loop {
                match get_data() {
                    Ok(data) => {
                        let mut queue = queue.lock().unwrap();
                        queue.data.push_back(data);
                    },
                    Err(error) => match error.kind() {
                        ErrorKind::Other => {
                            let mut queue = queue.lock().unwrap();
                            queue.state = QueueState::Complete;
                            break;
                        },
                        _ => {
                            write!(stderr(), "Unexpected Error {}", error)
                                .expect("stderr failure");
                        },
                    }
                }
            }
        });
    }
    let queue = queue.clone();
    loop {
        // do this in a block because the MutexGuard type is RAII and unlocks
        // when it goes out of scope
        let result = {
            let mut queue = queue.lock().unwrap();
            queue.data.pop_front()
        };
        match result {
            Some(data) =>  match put_data(data) {
                Err(err) => {
                    write!(stderr(), "Write error! {}\n", err)
                        .expect("stderr failure");
                },
                Ok(_) => {},
            },
            None => {
                // this is inelegant, but it works okay
                let result = {
                    let queue = queue.lock().unwrap();
                    queue.state.clone()
                };
                match result {
                    QueueState::Incomplete => thread::yield_now(),
                    QueueState::Complete => {
                        break;
                    },
                }
            },
        }
    }
}
