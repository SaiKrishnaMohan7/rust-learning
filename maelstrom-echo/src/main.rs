use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

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
    Echo {
        msg_id: u64,
        echo: String,
    },
    EchoOk {
        msg_id: u64,
        in_reply_to: u64,
        echo: String,
    },
}

struct Node {
    id: String,
    next_msg_id: u64,
}

impl Node {
    fn new() -> Self {
        Node {
            id: String::new(),
            next_msg_id: 0,
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
        let message: Message = serde_json::from_str(&line).unwrap();
        match message.body {
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
                    dest: message.src,
                    body: reply_body,
                };
                node.send(&mut stdout, &reply);
            }
            Body::Echo { msg_id, echo } => {
                let outgoing_id = node.next_id();
                let reply_body = Body::EchoOk {
                    msg_id: outgoing_id,
                    in_reply_to: msg_id,
                    echo,
                };
                let reply: Message = Message {
                    src: node.id.clone(),
                    dest: message.src,
                    body: reply_body,
                };
                node.send(&mut stdout, &reply);
            }
            Body::InitOk { .. } | Body::EchoOk { .. } => (),
        };
    }
}
