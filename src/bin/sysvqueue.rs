extern crate sysv_ipc;
use sysv_ipc::queue;
use std::default::Default;
use std::env;

fn main() {
    let mut args = env::args();
    let (args_length, _) = args.size_hint();

    if args_length < 3 {
        println!("{}: need at least 2 arguments", args.nth(0).unwrap());
        return;
    }

    let command = args.nth(1).unwrap();

    let value = args.next().unwrap();
    let queue_id_string = {
        let slice = value.as_str();
        if slice.len() >= 3 && (slice.starts_with("0x") || slice.starts_with("0X")) {
            &slice[2..]
        } else {
            &slice
        }
    };
    let queue_id: i32 = i32::from_str_radix(queue_id_string, 16).expect("Bad hex value!");

    if queue_id < 0 {
        panic!("Queue ID needs to be more than 0");
    }

    let queue = match queue::MessageQueue::new(queue_id, Default::default()) {
        Ok(queue) => queue,
        Err(msg) => panic!(msg)
    };
    match command.as_str() {
        "send" => {
            if args_length != 4 {
                panic!("Need data to send command")
            }
            match queue.send(1, args.next().unwrap().as_bytes(), Default::default()) {
                Ok(_) => (),
                Err(msg) => panic!(msg)
            }
        },
        "receive" => {
            if args_length != 3 {
                panic!("Too many arguments")
            }
            let message = String::from_utf8(match queue.receive(0, Default::default()) {
                Ok((_, message)) => message,
                Err(msg) => panic!(msg)
            }).ok().unwrap();

            println!("{}", message);
        },
        "remove" => {
            match queue.remove() {
                Ok(_) => (),
                Err(msg) => panic!(msg),
            }
        },
        _ => {
            panic!("Unknown command")
        }
    }
}
