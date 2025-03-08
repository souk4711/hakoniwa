use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/assets"]
pub(crate) struct Assets;
