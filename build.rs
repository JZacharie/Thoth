fn main() {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", now);

    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("resources/thoth.ico");
        res.compile().expect("failed to embed icon resource");
    }
}
