fn main() {
    // Generate protobuf structs.
    protobuf_codegen::Codegen::new()
        .includes(["include", "proto"])
        .input("proto/debugpb.proto")
        .cargo_out_dir("protos")
        .run()
        .unwrap();

    // Generate tonic service stubs.
    let out_dir = format!(
        "{}/protos",
        std::env::var("OUT_DIR").expect("No OUT_DIR defined")
    );
    tonic_build_protobuf::Builder::new()
        .out_dir(&out_dir)
        .proto_path("crate")
        .file_name(|pkg, svc| format!("{pkg}_{svc}_tonic"))
        .compile(&["proto/debugpb.proto"], &["proto", "include"]);

    // Generate mod file.
    let content = r"
pub mod debugpb;
pub mod debugpb_debug_tonic;
";
    let mod_path = std::path::Path::new(&out_dir).join("mod.rs");
    let previous_content = std::fs::read(&mod_path);
    if previous_content
        .map(|previous_content| previous_content != content.as_bytes())
        .unwrap_or(true)
    {
        std::fs::write(mod_path, content).unwrap();
    }
}
