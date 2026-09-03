use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::{self, BufRead, Write},
};

/// The wire envelope. Always these three fields → a STRUCT (product type).
#[derive(Debug, Deserialize, Serialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

/// The payload. Exactly ONE of these shapes → an ENUM (sum type).
///
/// `tag = "type"`      -> serde reads the JSON "type" field to pick the variant
/// `rename_all`        -> InitOk <-> "init_ok
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Body {
    Init {
        msg_id: u64,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        msg_id: u64,
        in_reply_to: u64,
    },
    Broadcast {
        msg_id: u64,
        message: u64,
    },
    BroadcastOk {
        msg_id: u64,
        in_reply_to: u64,
    },
    Read {
        msg_id: u64,
    },
    ReadOk {
        msg_id: u64,
        in_reply_to: u64,
        messages: Vec<u64>,
    },
    Topology {
        msg_id: u64,
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk {
        msg_id: u64,
        in_reply_to: u64,
    },
}

struct Node {
    id: String,
    next_msg_id: u64,
    messages: HashSet<u64>,
}

impl Node {
    fn new() -> Self {
        Node {
            id: String::new(),
            next_msg_id: 0,
            messages: HashSet::new(),
        }
    }
}
impl Node {
    fn next_id(&mut self) -> u64 {
        let next_id = self.next_msg_id + 1;
        self.next_msg_id = next_id;

        next_id
    }
    fn send(&self, stdout: &mut impl Write, reply: &Message) {
        let reply_json_string = serde_json::to_string(reply).unwrap();
        writeln!(stdout, "{reply_json_string}").unwrap();
        stdout.flush().unwrap();
    }
    fn insert_message(&mut self, message: u64) {
        self.messages.insert(message);
    }
}
fn main() {
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();
    let mut node = Node::new();

    for line in stdin.lines() {
        let line = match line {
            Ok(line) => line,
            Err(err) => {
                eprintln!("Error reading line: {err}");
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let message_envelope: Message = serde_json::from_str(&line).unwrap();
        match message_envelope.body {
            Body::Init {
                msg_id,
                node_id,
                node_ids,
            } => {
                node.id = node_id;
                let outgoing_id = node.next_id();

                let reply_body = Body::InitOk {
                    msg_id: outgoing_id,
                    in_reply_to: msg_id,
                };
                let reply = Message {
                    src: node.id.clone(),
                    dest: message_envelope.src,
                    body: reply_body,
                };
                node.send(&mut stdout, &reply);
            }
            Body::Broadcast { msg_id, message } => {
                let outgoing_id = node.next_id();
                let reply_body = Body::BroadcastOk {
                    msg_id: outgoing_id,
                    in_reply_to: msg_id,
                };
                let reply = Message {
                    src: node.id.clone(),
                    dest: message_envelope.src,
                    body: reply_body,
                };
                node.insert_message(message);
                node.send(&mut stdout, &reply);
            }
            Body::Read { msg_id } => {
                let outgoing_id = node.next_id();
                let read_ok_body = Body::ReadOk {
                    msg_id: outgoing_id,
                    in_reply_to: msg_id,
                    messages: node.messages.iter().map(|m| *m).collect(),
                };
                let reply = Message {
                    src: node.id.clone(),
                    dest: message_envelope.src,
                    body: read_ok_body,
                };
                node.send(&mut stdout, &reply);
            }
            Body::Topology { msg_id, topology } => {
                eprintln!("{:?}", topology);
                let outgoing_id = node.next_id();
                let reply_topo_ok = Body::TopologyOk {
                    msg_id: outgoing_id,
                    in_reply_to: msg_id,
                };
                let reply = Message {
                    src: node.id.clone(),
                    dest: message_envelope.src,
                    body: reply_topo_ok,
                };
                node.send(&mut stdout, &reply);
            }
            Body::InitOk { .. }
            | Body::ReadOk { .. }
            | Body::TopologyOk { .. }
            | Body::BroadcastOk { .. } => (),
        };
    }
}
