pub enum Error {
    IoError(std::io::Error),
    Config(config::ConfigError),
    BaseUrl(base_url::BaseUrlError),
    Serde(serde_json::Error),
    Api(String),
    NotFound(String),
    FileTypeNotSupported(String),
    Unspecified(String),
    Mismatch { expected: String, actual: String },
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error::Unspecified(message.to_string())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IO Error: {}", e),
            Error::Config(e) => write!(f, "Config Error: {}", e),
            Error::BaseUrl(e) => write!(f, "Base URL Error"),
            Error::Serde(e) => write!(f, "Serde Error: {}", e),
            Error::Api(e) => write!(f, "API Error: {}", e),
            Error::NotFound(e) => write!(f, "Release Not Found Error: {}", e),
            Error::FileTypeNotSupported(e) => write!(f, "File Type Not Supported Error: {}", e),
            Error::Unspecified(e) => write!(f, "Unspecified Error: {}", e),
            Error::Mismatch { expected, actual } => write!(
                f,
                "Hash Mismatch Error: expected {}, got {}",
                expected, actual
            ),
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
        Error::BaseUrl(error)
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
