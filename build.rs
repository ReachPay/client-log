fn main() {
    tonic_build::compile_protos("proto/ClientLogService.proto").unwrap();
}
