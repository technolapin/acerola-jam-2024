#[derive(Debug, Clone)]
pub struct Error(String);


use std::fmt::Debug;

impl Error
{
    fn new<T: Debug>(e: T) -> Error
    {
	Self(format!("{:?}", e))
    }
    pub fn err<T: Debug, O>(e: T) -> Result<O, Error>
    {
	Result::Err(Self::new(e))
    }
}

macro_rules! impl_error
{
    ($type: ty) =>
    {
	impl From<$type> for Error
	{
	    fn from(e: $type) -> Error
	    {
		Error::new(e)
	    }
	}
    }
}


impl_error!(glutin::error::Error);
