#[allow(clippy::unnecessary_wraps, reason = "system-dependent buildscript")]
fn main() -> std::io::Result<()> {
	#[cfg(windows)]
	{
		winresource::WindowsResource::new()
			.set_icon("../../assets/icon.ico")
			.compile()?;
	}

	Ok(())
}
