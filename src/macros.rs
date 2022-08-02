macro_rules! defer {
    ($($body:tt)*) => {
        let _guard = {
            pub struct Guard<F: FnOnce()>(Option<F>);

            impl<F: FnOnce()> Drop for Guard<F> {
                fn drop(&mut self) {
                    (self.0).take().map(|f| f());
                }
            }

            Guard(Some(|| {
                let _ = { $($body)* };
            }))
        };
    };
}
pub(crate) use defer;

macro_rules! tryfn {
    ($fn:expr, $($arg:tt)*) => {
        match $fn {
            Ok(val) => Ok(val),
            Err(err) => {
                let err = format!("{} => {}", format!($($arg)*), err.to_string());
                Err($crate::Error::FnError(err))
            }
        }
    };
}
pub(crate) use tryfn;
