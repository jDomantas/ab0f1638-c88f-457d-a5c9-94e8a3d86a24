use std::fmt::Display;

pub trait ResultExt {
    fn log_if_err(self);
}

impl<T, E: Display> ResultExt for Result<T, E> {
    fn log_if_err(self) {
        if let Err(e) = self {
            error!("{}", e);
        }
    }
}
