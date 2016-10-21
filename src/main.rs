#![feature(proc_macro)]

extern crate zmq;
extern crate termion;
extern crate num_cpus;
extern crate chrono;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::thread;
use chrono::*;
use termion::color;

static SOCKET_ADDR: &'static str = "inproc://taskqueue";
static FORMAT_STRING: &'static str = "%H:%M:%S%.6f";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum ZmqMessage {
    LocalTimeMessage(String),
    EndMessage
}

fn worker(mut pull_socket: zmq::Socket) {
    // receive some messages
    let mut msg = zmq::Message::new().unwrap();

    loop {
        pull_socket.recv(&mut msg, 0).unwrap();

        // print out the message
        let m: ZmqMessage = serde_json::from_str(&msg.as_str().unwrap()).unwrap();
        println!("{blue}[INFO]{reset} message received: {red}{message:?}{reset}",
                 red = color::Fg(color::Red),
                 blue = color::Fg(color::Blue),
                 message = m,
                 reset = color::Fg(color::Reset));
    }
}

fn main() {
    // zmq context and sockets
    let mut ctx = zmq::Context::new();
    let mut push_socket = ctx.socket(zmq::PUSH).unwrap();
    push_socket.bind(SOCKET_ADDR).unwrap();

   // simple thread numbers
    let num_threads = num_cpus::get() * 2;
    let mut children = vec![];

    // create the child threads
    for _ in 0..num_threads {
        // create the socket and connect it
        let mut pull_socket = ctx.socket(zmq::PULL).unwrap();
        pull_socket.connect(SOCKET_ADDR).unwrap();

        // push the socket
        children.push(thread::spawn(move || {
            worker(pull_socket);
        }));
    }

    // send a whole bunch of random messages
    for _ in 0..num_threads*5 {
        let time_string = Local::now().format(FORMAT_STRING).to_string();
        let zmq_message = ZmqMessage::LocalTimeMessage(time_string);
        let now = serde_json::to_string(&zmq_message).unwrap();
        push_socket.send_str(&now, 0).unwrap();
    }

    // join the children pools
    for child in children {
        let _ = child.join();
    }
}
