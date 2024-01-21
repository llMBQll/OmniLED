fn main() {
    prost_build::compile_protos(&["src/types.proto"], &["src/"]).unwrap();
}
