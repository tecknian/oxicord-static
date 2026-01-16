use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    WaitingForHello,
    Identifying,
    Resuming,
    Connected,
    Reconnecting {
        attempt: u32,
    },
    ShuttingDown,
}

impl ConnectionState {
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Connecting
                | Self::WaitingForHello
                | Self::Identifying
                | Self::Resuming
                | Self::Connected
        )
    }

    #[must_use]
    pub const fn can_send(&self) -> bool {
        matches!(self, Self::Connected)
    }

    #[must_use]
    pub const fn is_reconnecting(&self) -> bool {
        matches!(self, Self::Reconnecting { .. })
    }

    #[must_use]
    pub const fn reconnect_attempt(&self) -> Option<u32> {
        if let Self::Reconnecting { attempt } = self {
            Some(*attempt)
        } else {
            None
        }
    }
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::WaitingForHello => write!(f, "Waiting for Hello"),
            Self::Identifying => write!(f, "Identifying"),
            Self::Resuming => write!(f, "Resuming"),
            Self::Connected => write!(f, "Connected"),
            Self::Reconnecting { attempt } => write!(f, "Reconnecting (attempt {attempt})"),
            Self::ShuttingDown => write!(f, "Shutting Down"),
        }
    }
}

pub struct GatewayState {
    connection: ConnectionState,
    last_heartbeat_sent: Option<Instant>,
    last_heartbeat_ack: Option<Instant>,
    heartbeat_interval_ms: Option<u64>,
    latency_ms: Option<u64>,
    started_at: Option<Instant>,
    reconnect_attempts: u32,
}

impl GatewayState {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            last_heartbeat_sent: None,
            last_heartbeat_ack: None,
            heartbeat_interval_ms: None,
            latency_ms: None,
            started_at: None,
            reconnect_attempts: 0,
        }
    }

    #[must_use]
    pub const fn connection(&self) -> ConnectionState {
        self.connection
    }

    pub fn set_connection(&mut self, state: ConnectionState) {
        self.connection = state;
        if state == ConnectionState::Connected {
            self.started_at = Some(Instant::now());
            self.reconnect_attempts = 0;
        } else if state == ConnectionState::Disconnected {
            self.last_heartbeat_sent = None;
            self.last_heartbeat_ack = None;
        }
    }

    pub const fn transition_to_connecting(&mut self) {
        self.connection = ConnectionState::Connecting;
    }

    pub const fn transition_to_waiting_hello(&mut self) {
        self.connection = ConnectionState::WaitingForHello;
    }

    pub const fn transition_to_identifying(&mut self) {
        self.connection = ConnectionState::Identifying;
    }

    pub const fn transition_to_resuming(&mut self) {
        self.connection = ConnectionState::Resuming;
    }

    pub fn transition_to_connected(&mut self) {
        self.connection = ConnectionState::Connected;
        self.started_at = Some(Instant::now());
        self.reconnect_attempts = 0;
    }

    pub const fn transition_to_reconnecting(&mut self) -> u32 {
        self.reconnect_attempts += 1;
        self.connection = ConnectionState::Reconnecting {
            attempt: self.reconnect_attempts,
        };
        self.reconnect_attempts
    }

    pub const fn transition_to_disconnected(&mut self) {
        self.connection = ConnectionState::Disconnected;
        self.last_heartbeat_sent = None;
        self.last_heartbeat_ack = None;
    }

    pub const fn transition_to_shutdown(&mut self) {
        self.connection = ConnectionState::ShuttingDown;
    }

    pub const fn set_heartbeat_interval(&mut self, interval_ms: u64) {
        self.heartbeat_interval_ms = Some(interval_ms);
    }

    #[must_use]
    pub const fn heartbeat_interval_ms(&self) -> Option<u64> {
        self.heartbeat_interval_ms
    }

    pub fn record_heartbeat_sent(&mut self) {
        self.last_heartbeat_sent = Some(Instant::now());
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn record_heartbeat_ack(&mut self) {
        let now = Instant::now();
        if let Some(sent) = self.last_heartbeat_sent {
            self.latency_ms = Some(now.duration_since(sent).as_millis() as u64);
        }
        self.last_heartbeat_ack = Some(now);
    }

    #[must_use]
    pub const fn latency_ms(&self) -> Option<u64> {
        self.latency_ms
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn is_heartbeat_overdue(&self) -> bool {
        let Some(interval) = self.heartbeat_interval_ms else {
            return false;
        };

        let Some(last_ack) = self.last_heartbeat_ack else {
            if let Some(sent) = self.last_heartbeat_sent {
                let timeout = interval as f64 * super::constants::HEARTBEAT_TIMEOUT_MULTIPLIER;
                return sent.elapsed().as_millis() as f64 > timeout;
            }
            return false;
        };

        let timeout = interval as f64 * super::constants::HEARTBEAT_TIMEOUT_MULTIPLIER;
        last_ack.elapsed().as_millis() as f64 > timeout
    }

    #[must_use]
    pub fn uptime(&self) -> Option<std::time::Duration> {
        self.started_at.map(|start| start.elapsed())
    }

    #[must_use]
    pub const fn reconnect_attempts(&self) -> u32 {
        self.reconnect_attempts
    }

    pub const fn reset_reconnect_attempts(&mut self) {
        self.reconnect_attempts = 0;
    }

    pub const fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for GatewayState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_display() {
        assert_eq!(ConnectionState::Connected.to_string(), "Connected");
        assert_eq!(
            ConnectionState::Reconnecting { attempt: 3 }.to_string(),
            "Reconnecting (attempt 3)"
        );
    }

    #[test]
    fn test_connection_state_checks() {
        assert!(ConnectionState::Connected.is_connected());
        assert!(ConnectionState::Connected.can_send());
        assert!(!ConnectionState::Connecting.can_send());
        assert!(ConnectionState::Reconnecting { attempt: 1 }.is_reconnecting());
    }

    #[test]
    fn test_gateway_state_transitions() {
        let mut state = GatewayState::new();
        assert_eq!(state.connection(), ConnectionState::Disconnected);

        state.transition_to_connecting();
        assert_eq!(state.connection(), ConnectionState::Connecting);

        state.transition_to_connected();
        assert!(state.connection().is_connected());
        assert_eq!(state.reconnect_attempts(), 0);
    }

    #[test]
    fn test_reconnect_attempts() {
        let mut state = GatewayState::new();
        assert_eq!(state.transition_to_reconnecting(), 1);
        assert_eq!(state.transition_to_reconnecting(), 2);
        state.reset_reconnect_attempts();
        assert_eq!(state.reconnect_attempts(), 0);
    }
}
