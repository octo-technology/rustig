extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["proto/hook.proto"], &["proto"]).unwrap();
}
