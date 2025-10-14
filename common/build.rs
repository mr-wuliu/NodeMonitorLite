fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .compile_protos(&["proto/machine.proto"], &["proto"]) ?;
    println!("cargo:rer11un-if-changed=proto/machine.proto");
    Ok(())
}