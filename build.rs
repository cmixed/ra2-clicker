fn main() {
    slint_build::compile("ui/app.slint").expect("Slint UI compilation failed");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        winresource::WindowsResource::new()
            .set_icon("ui/RA2.ico")
            .compile()
            .expect("Failed to embed icon");
    }
}
