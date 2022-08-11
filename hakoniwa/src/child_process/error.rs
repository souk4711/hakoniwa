pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

macro_rules! map_err {
    ($mod:ident :: $fn:ident ($($arg:expr),* )) => {
        $mod::$fn($($arg),*).map_err(|err| {
            Error(format!("{}::{}(...) -> {}", stringify!($mod), stringify!($fn), err))
        })
    };

    ($obj:ident . $fn:ident ($($arg:expr),* )) => {
        $obj.$fn($($arg),*).map_err(|err| {
            Error(format!("{}.{}(...) -> {}", stringify!($obj), stringify!($fn), err))
        })
    };
}
pub(crate) use map_err;
