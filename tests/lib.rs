extern crate sysv_ipc;
use sysv_ipc::queue::{MessageQueue};
use std::default::Default;

fn setup() -> MessageQueue {
    match MessageQueue::new(0, Default::default()) {
        Ok(result) => result,
        Err(msg) => panic!(msg)
    };
}

#[test]
fn send() {
    let queue = setup();
    queue.send(1, "Wow".as_bytes(), Default::default()).ok().unwrap();
    let (msg_type, bytes) = queue.receive(0, Default::default()).ok().unwrap();
    assert_eq!(msg_type, 1);
    assert_eq!(bytes.as_slice(), "Wow".as_bytes());
}

#[test]
fn remove() {
    let queue = setup();
    assert_eq!(queue.remove().ok().unwrap(), ());
}
