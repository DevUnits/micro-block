fn main() {
    println!("cargo:rerun-if-changed=app.manifest.xml");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    embed_manifest::embed_manifest_file("app.manifest.xml")
        .expect("failed to embed application manifest");
    winresource::WindowsResource::new()
        .set_icon("assets/icon.ico")
        .compile()
        .expect("failed to embed icon");
}
