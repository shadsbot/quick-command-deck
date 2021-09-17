use protobuf_codegen_pure::Customize;

fn main() {
    match protobuf_codegen_pure::Codegen::new()
        .customize(Customize {
            gen_mod_rs: Some(true),
            ..Default::default()
        })
        .out_dir("src/protos")
        .include("../")
        .input("../communique.proto")
        .run()
    {
        Err(e) => {
            println!("cargo:error={:?}", e)
        }
        Ok(_) => {}
    }
}
