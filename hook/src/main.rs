use prost::Message;
use std::io;
use std::io::prelude::*;
use std::io::Cursor;

pub mod hook {
    include!(concat!(env!("OUT_DIR"), "/hook.rs"));
}

fn main() {
    let mut stdin = io::stdin().lock();
    let input = stdin.fill_buf().unwrap();

    let req = hook::Req::decode(&mut Cursor::new(input)).unwrap();
    for (key, value) in std::env::vars() {
        eprintln!("{key}: {value}");
    }

    let mut resp = hook::Resp::default();
    resp.bar = req.foo;

    let mut output = Vec::new();
    output.reserve(resp.encoded_len());
    resp.encode(&mut output).unwrap();

    io::stdout().write_all(&output).unwrap();
}
