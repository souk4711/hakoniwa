use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/embed"]
pub struct Embed;
