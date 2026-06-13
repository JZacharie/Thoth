fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("resources/thoth.ico");
        res.compile().expect("failed to embed icon resource");
    }
}
