mod cli;
mod contrib;
mod embed;
mod error;

use embed::Embed;
use error::{Error, Result};

pub use cli::execute;
