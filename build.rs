use std::io;

fn main() -> io::Result<()> {
    #[cfg(feature = "winapi")]
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        use {std::env, winresource::WindowsResource};
        WindowsResource::new().set_icon("misc/icon.ico").compile()?;
    }
    Ok(())
}
