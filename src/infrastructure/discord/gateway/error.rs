use std::io;
use thiserror::Error;

use super::constants::GatewayOpcode;

pub type GatewayResult<T> = Result<T, GatewayError>;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("connection closed with code {code}: {reason}")]
    ConnectionClosed { code: u16, reason: String },

    #[error("websocket error: {message}")]
    WebSocket { message: String },

    #[error("authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("session invalidated, resumable: {resumable}")]
    SessionInvalidated { resumable: bool },

    #[error("heartbeat timeout: no acknowledgment received")]
    HeartbeatTimeout,

    #[error("rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("reconnection limit exceeded after {attempts} attempts")]
    ReconnectionLimitExceeded { attempts: u32 },

    #[error("compression error: {message}")]
    CompressionError { message: String },

    #[error("serialization error: {message}")]
    SerializationError { message: String },

    #[error("protocol error: unexpected opcode {opcode:?}")]
    UnexpectedOpcode { opcode: Option<GatewayOpcode> },

    #[error("protocol error: {message}")]
    ProtocolError { message: String },

    #[error("timeout waiting for {operation}")]
    Timeout { operation: String },

    #[error("channel closed")]
    ChannelClosed,

    #[error("not connected to gateway")]
    NotConnected,

    #[error("already connecting or connected")]
    AlreadyConnected,

    #[error("gateway shutting down")]
    ShuttingDown,

    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

impl GatewayError {
    #[must_use]
    pub fn connection_failed(message: impl Into<String>) -> Self {
        Self::ConnectionFailed {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn websocket(message: impl Into<String>) -> Self {
        Self::WebSocket {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn auth_failed(message: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn compression(message: impl Into<String>) -> Self {
        Self::CompressionError {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::ProtocolError {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionFailed { .. }
                | Self::ConnectionClosed { .. }
                | Self::WebSocket { .. }
                | Self::HeartbeatTimeout
                | Self::RateLimited { .. }
                | Self::SessionInvalidated { resumable: true }
                | Self::Io(_)
        )
    }

    #[must_use]
    #[allow(clippy::match_wildcard_for_single_variants, clippy::match_same_arms)]
    pub const fn should_reconnect(&self) -> bool {
        match self {
            Self::SessionInvalidated { resumable } => *resumable,

            Self::ConnectionFailed { .. }
            | Self::ConnectionClosed { .. }
            | Self::WebSocket { .. }
            | Self::HeartbeatTimeout
            | Self::CompressionError { .. }
            | Self::Io(_)
            | Self::RateLimited { .. } => true,

            Self::AuthenticationFailed { .. }
            | Self::ReconnectionLimitExceeded { .. }
            | Self::ShuttingDown
            | Self::NotConnected
            | Self::AlreadyConnected
            | Self::ProtocolError { .. }
            | Self::SerializationError { .. }
            | Self::UnexpectedOpcode { .. }
            | Self::Timeout { .. } => false,

            _ => false,
        }
    }

    #[must_use]
    pub const fn can_resume(&self) -> bool {
        matches!(
            self,
            Self::ConnectionClosed { .. }
                | Self::WebSocket { .. }
                | Self::HeartbeatTimeout
                | Self::Io(_)
        )
    }

    #[must_use]
    pub const fn close_code(&self) -> Option<u16> {
        if let Self::ConnectionClosed { code, .. } = self {
            Some(*code)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GatewayCloseCode {
    UnknownError = 4000,
    UnknownOpcode = 4001,
    DecodeError = 4002,
    NotAuthenticated = 4003,
    AuthenticationFailed = 4004,
    AlreadyAuthenticated = 4005,
    InvalidSequence = 4007,
    RateLimited = 4008,
    SessionTimedOut = 4009,
    InvalidShard = 4010,
    ShardingRequired = 4011,
    InvalidApiVersion = 4012,
    InvalidIntents = 4013,
    DisallowedIntents = 4014,
}

impl GatewayCloseCode {
    #[must_use]
    pub const fn from_u16(code: u16) -> Option<Self> {
        match code {
            4000 => Some(Self::UnknownError),
            4001 => Some(Self::UnknownOpcode),
            4002 => Some(Self::DecodeError),
            4003 => Some(Self::NotAuthenticated),
            4004 => Some(Self::AuthenticationFailed),
            4005 => Some(Self::AlreadyAuthenticated),
            4007 => Some(Self::InvalidSequence),
            4008 => Some(Self::RateLimited),
            4009 => Some(Self::SessionTimedOut),
            4010 => Some(Self::InvalidShard),
            4011 => Some(Self::ShardingRequired),
            4012 => Some(Self::InvalidApiVersion),
            4013 => Some(Self::InvalidIntents),
            4014 => Some(Self::DisallowedIntents),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_resumable(self) -> bool {
        matches!(
            self,
            Self::UnknownError
                | Self::UnknownOpcode
                | Self::DecodeError
                | Self::NotAuthenticated
                | Self::InvalidSequence
                | Self::RateLimited
                | Self::SessionTimedOut
        )
    }

    #[must_use]
    pub const fn is_fatal(self) -> bool {
        matches!(
            self,
            Self::AuthenticationFailed
                | Self::InvalidShard
                | Self::ShardingRequired
                | Self::InvalidApiVersion
                | Self::InvalidIntents
                | Self::DisallowedIntents
        )
    }
}

impl From<GatewayCloseCode> for u16 {
    fn from(code: GatewayCloseCode) -> Self {
        code as Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        assert!(GatewayError::connection_failed("test").is_recoverable());
        assert!(GatewayError::HeartbeatTimeout.is_recoverable());
        assert!(!GatewayError::auth_failed("test").is_recoverable());
        assert!(!GatewayError::ShuttingDown.is_recoverable());
    }

    #[test]
    fn test_close_code_mapping() {
        assert_eq!(
            GatewayCloseCode::from_u16(4004),
            Some(GatewayCloseCode::AuthenticationFailed)
        );
        assert!(GatewayCloseCode::AuthenticationFailed.is_fatal());
        assert!(!GatewayCloseCode::UnknownError.is_fatal());
        assert!(GatewayCloseCode::SessionTimedOut.is_resumable());
    }
}
