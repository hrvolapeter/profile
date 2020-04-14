fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../scheduler_proto/scheduler.proto")?;
    println!("cargo:rerun-if-changed=../scheduler_proto/scheduler.proto");
    Ok(())
}
