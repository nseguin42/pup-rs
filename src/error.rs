pub enum Error {
    IoError(std::io::Error),
    Config(config::ConfigError),
    Url(String),
    Serde(serde_json::Error),
    Api(String),
    NotFound(String),
    FileTypeNotSupported(String),
    Unspecified(String),
    Mismatch { expected: String, actual: String },
    CacheFileNotFound(String),
    NoDownloadStrategy,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error::Unspecified(message.to_string())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IO: {}", e),
            Error::Config(e) => write!(f, "Config: {}", e),
            Error::Url(e) => write!(f, "URL Error"),
            Error::Serde(e) => write!(f, "Serde: {}", e),
            Error::Api(e) => write!(f, "API: {}", e),
            Error::NotFound(e) => write!(f, "Not found: {}", e),
            Error::FileTypeNotSupported(e) => write!(f, "File type not supported: {}", e),
            Error::Unspecified(e) => write!(f, "Unspecified: {}", e),
            Error::CacheFileNotFound(e) => write!(f, "Cache file not found: {}", e),
            Error::Mismatch { expected, actual } => write!(
                f,
                "Hash Mismatch Error: expected {}, got {}",
                expected, actual
            ),
            Error::NoDownloadStrategy => write!(f, "No download strategy"),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<config::ConfigError> for Error {
    fn from(error: config::ConfigError) -> Self {
        Error::Config(error)
    }
}

impl From<base_url::BaseUrlError> for Error {
    fn from(error: base_url::BaseUrlError) -> Self {
        Error::Url("Invalid URL".to_string())
    }
}

impl From<base_url::ParseError> for Error {
    fn from(error: base_url::ParseError) -> Self {
        Error::Url(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serde(error)
    }
}

impl From<octocrab::Error> for Error {
    fn from(error: octocrab::Error) -> Self {
        Error::Api(error.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Api(error.to_string())
    }
}
