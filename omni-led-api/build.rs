fn main() -> Result<(), std::io::Error> {
    tonic_prost_build::compile_protos("proto/plugin.proto")
}
