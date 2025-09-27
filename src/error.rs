use thiserror::Error;
use tokio_graceful_shutdown::errors::{SubsystemError, SubsystemJoinError};

pub type CCProxyResult<T> = Result<T, CCProxyError>;

#[derive(Debug, Error)]
pub enum CCProxyError {
    #[error("The IO error is occurred: {err}")]
    IO {
        #[from]
        err: std::io::Error,
    },

    #[error("The graceful shutdown error is occurred: {err}")]
    GracefulShutdown {
        #[from]
        err: tokio_graceful_shutdown::errors::GracefulShutdownError<Self>,
    },

    #[error("The config error is occurred: {err}")]
    Config {
        #[from]
        err: Box<figment::Error>,
    },

    #[error("The tracing appender rolling init error is occurred: {err}")]
    TracingAppenderRollingInit {
        #[from]
        err: tracing_appender::rolling::InitError,
    },

    #[error("The tracing subscriber filter parse error is occurred: {err}")]
    TracingSubscriberParse {
        #[from]
        err: tracing_subscriber::filter::ParseError,
    },

    #[error("The RakNet error is occurred: {err}")]
    RakNet {
        err: rust_raknet::error::RaknetError,
    },

    #[error("The upstream server responded a invalid MOTD.")]
    UpstreamMotdInvalid,

    #[error("The MOTD is invalid.")]
    MotdInvalid,

    #[error("The Query Protocol packet is invalid.")]
    QueryInvalid,

    #[error("Cannot receive the Query Protocol packet due to timeout.")]
    QueryTimeout,
}

impl From<rust_raknet::error::RaknetError> for CCProxyError {
    fn from(err: rust_raknet::error::RaknetError) -> Self {
        Self::RakNet { err }
    }
}

pub fn sub_sys_err_to_ccproxy_err(err: &SubsystemJoinError<CCProxyError>) -> Option<&CCProxyError> {
    let SubsystemJoinError::SubsystemsFailed(err) = err;
    if let SubsystemError::Failed(_name, err) = &err[0] {
        return Some(err.get_error());
    }

    None
}
