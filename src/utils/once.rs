use std::sync::OnceLock;

// TODO: Remove this once https://github.com/rust-lang/rust/issues/116693 is stable

pub trait OnceLockTry<T> {
    fn __try_init<F, E>(&self, f: F) -> Result<&T, E>
        where F: FnOnce() -> Result<T, E>;

    fn __try_insert(&self, value: T) -> Result<&T, (&T, T)>;
}

impl <T>OnceLockTry<T> for OnceLock<T> {
    #[cold]
    fn __try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let val = f()?;
        self.__try_insert(val).map_or_else(|_| panic!("reentrant init"), |v| Ok(v))
    }

    #[allow(clippy::unwrap_used, clippy::option_if_let_else)]
    fn __try_insert(&self, value: T) -> Result<&T, (&T, T)> {
        let mut value = Some(value);
        let res = self.get_or_init(|| value.take().unwrap());
        match value {
            None => Ok(res),
            Some(value) => Err((res, value)),
        }
    }
}
