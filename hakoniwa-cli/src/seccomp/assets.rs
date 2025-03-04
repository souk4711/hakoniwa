use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/seccomp/assets"]
pub(crate) struct Assets;
