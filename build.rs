fn main() {
    slint_build::compile("ui/app.slint").expect("failed to compile slint UI");

    // Embed the app icon as a Windows resource so the .exe shows it in Explorer
    // and the taskbar. No-op (and not compiled) on other hosts.
    #[cfg(windows)]
    {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/icons/icon.ico");
        resource
            .compile()
            .expect("failed to embed Windows icon resource");
    }
}
