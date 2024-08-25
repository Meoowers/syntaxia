use std::error::Error as Err;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Enum representing command errors, either user-triggered or system-related.
pub enum CommandError {
    User(String),
    System(Error),
}

impl CommandError {
    pub fn from<T: Err + Send + Sync + 'static>(data: T) -> Self {
        CommandError::System(Box::new(data))
    }
}

impl From<&str> for CommandError {
    fn from(data: &str) -> CommandError {
        CommandError::User(data.to_owned())
    }
}

impl From<String> for CommandError {
    fn from(data: String) -> CommandError {
        CommandError::User(data)
    }
}

impl From<Error> for CommandError {
    fn from(data: Error) -> CommandError {
        CommandError::System(data)
    }
}

impl From<serenity::Error> for CommandError {
    fn from(value: serenity::Error) -> Self {
        CommandError::System(value.into())
    }
}

impl From<serde_yaml::Error> for CommandError {
    fn from(value: serde_yaml::Error) -> Self {
        CommandError::System(value.into())
    }
}
