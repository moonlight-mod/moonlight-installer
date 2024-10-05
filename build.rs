fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("icon.ico")
            .compile()?;
    }

    Ok(())
}
