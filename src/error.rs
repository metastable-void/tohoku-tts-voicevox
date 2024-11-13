
use std::error::Error;
use std::fmt::{self, Display, Debug};

pub trait ErrorDescription {
    fn description(&self) -> impl Display;
    fn code(&self) -> Option<i32> {
        None
    }
    fn error_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl<T> ErrorDescription for T
where 
    T: Display,
{
    fn description(&self) -> impl Display {
        self
    }
}

pub struct GenericError<T>(pub T) where T: ErrorDescription;

impl<T> GenericError<T>
where 
    T: ErrorDescription,
{
    pub const fn new(err: T) -> Self {
        Self(err)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn as_inner(&self) -> &T {
        &self.0
    }

    pub fn as_inner_mut(&mut self) -> &mut T {
        &mut self.0
    }

    pub fn map<U, F>(self, f: F) -> GenericError<U>
    where 
        U: ErrorDescription,
        F: FnOnce(T) -> U,
    {
        GenericError(f(self.0))
    }

    pub fn error_name(&self) -> &'static str {
        self.0.error_name()
    }

    pub fn code(&self) -> Option<i32> {
        self.0.code()
    }
}

impl<T> Debug for GenericError<T>
where 
    T: ErrorDescription,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(code) = self.0.code() {
            write!(f, "GenericError({}): {} ({})", self.error_name(), self.0.description(), code)
        } else {
            write!(f, "GenericError({}): {}", self.error_name(), self.0.description())
        }
    }
}

impl<T> Display for GenericError<T>
where 
    T: ErrorDescription,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.description())
    }
}

impl<T> Error for GenericError<T>
where 
    T: ErrorDescription,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl<T> From<T> for GenericError<T>
where 
    T: ErrorDescription,
{
    fn from(err: T) -> Self {
        Self::new(err)
    }
}
