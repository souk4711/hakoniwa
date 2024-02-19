use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/embed"]
pub(crate) struct Embed;
