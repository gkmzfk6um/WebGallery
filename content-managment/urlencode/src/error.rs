
#[derive(Debug)]
pub struct Error 
{
    path: Option<String>,
    message: String,
    urlencode_error: Option<serde_urlencoded::ser::Error>
}
pub type Result<T> = std::result::Result<T,Error>;

impl Error {
    pub fn new(path: Option<String>, message: String) -> Error
    {
        Error {
            path,message,
            urlencode_error:None
        }
    }
}

impl From<serde_urlencoded::ser::Error> for Error
{
    fn from(error: serde_urlencoded::ser::Error) -> Error
    {
        Error {
            path: None,
            message: "urlencoding of key value array failed".into(),
            urlencode_error: Some(error)
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.path
        {
            Some(path) => {write!(f, "UrlEncodeError [{}]({})",path,self.message)},
            None => {write!(f, "UrlEncodeError ({})",self.message)},
        }
    }
}

impl serde::ser::StdError for Error 
{
    fn source(&self) -> Option<&(dyn serde::ser::StdError + 'static)> {
        match &self.urlencode_error
        {
            Some(err) => {Some(err)},
            None => None
        } 
    }

}

impl serde::ser::Error for Error 
{
    fn custom<T>(msg: T) -> Self 
    where 
        T: std::fmt::Display 
    {
        Error::new(None, format!("{}",msg))
    }
}

