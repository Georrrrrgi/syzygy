use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("user not found")]
    UserNotFound,

    #[error("post not found")]
    PostNotFound,

    #[error("username already exists")]
    UsernameAlreadyExists,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("already following this user")]
    AlreadyFollowing,

    #[error("not following this user")]
    NotFollowing,

    #[error("cannot follow yourself")]
    CannotFollowSelf,

    #[error("post content exceeds maximum length")]
    ContentTooLong,

    #[error("post content is empty")]
    EmptyContent,

    #[error("unauthorized")]
    Unauthorized,
}
