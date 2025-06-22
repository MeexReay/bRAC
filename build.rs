use std::io;

fn main() -> io::Result<()> {
    #[cfg(feature = "winapi")]
    {
        use {std::env, winresource::WindowsResource};
        if env::var_os("CARGO_CFG_WINDOWS").is_some() {
            WindowsResource::new().set_icon("misc/icon.ico").compile()?;
        }
    }
    Ok(())
}
