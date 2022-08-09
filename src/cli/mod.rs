mod error;
mod root;
mod run;
mod server;

use root::RootCommand;
use run::RunCommand;
use server::ServerCommand;

pub use root::execute;
