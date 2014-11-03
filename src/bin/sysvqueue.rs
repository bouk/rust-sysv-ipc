extern crate sysv_ipc;
use std::os;
use std::num::from_str_radix;
use sysv_ipc::queue;
use std::default::Default;

fn main() {
    let args: Vec<String> = os::args();

    if args.len() < 3 {
        println!("{}: need at least 2 arguments", args[0])
        return
    }


    let queue_id_string = {
        let slice = args[2].as_slice();
        if slice.len() >= 3 && (slice.starts_with("0x") || slice.starts_with("0X")) {
            slice.slice_from(2)
        } else {
            slice
        }
    };
    let queue_id: i32 = from_str_radix(queue_id_string, 16).expect("Bad hex value!");

    if queue_id < 0 {
        panic!("Queue ID needs to be more than 0");
    }

    let queue = match queue::MessageQueue::new(queue_id, Default::default()) {
        Ok(queue) => queue,
        Err(msg) => panic!(msg)
    };

    match args[1].as_slice() {
        "send" => {
            if args.len() != 4 {
                panic!("Need data to send command")
            }
            match queue.send(1, args[3].as_bytes(), Default::default()) {
                Ok(_) => (),
                Err(msg) => panic!(msg)
            }
        },
        "receive" => {
            if args.len() != 3 {
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
