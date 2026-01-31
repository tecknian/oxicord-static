use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
    pub created_at: Instant,
    pub displayed_at: Option<Instant>,
    pub duration: Duration,
}

impl Notification {
    #[must_use]
    pub fn new(
        level: NotificationLevel,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level,
            title: title.into(),
            message: message.into(),
            created_at: Instant::now(),
            displayed_at: None,
            duration: Duration::from_secs(5),
        }
    }

    #[must_use]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.displayed_at
            .is_some_and(|start| start.elapsed() > self.duration)
    }

    pub fn mark_displayed(&mut self) {
        if self.displayed_at.is_none() {
            self.displayed_at = Some(Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let n = Notification::new(NotificationLevel::Info, "Title", "Message");
        assert_eq!(n.level, NotificationLevel::Info);
        assert_eq!(n.title, "Title");
        assert_eq!(n.message, "Message");
        assert_eq!(n.duration, Duration::from_secs(5));
    }

    #[test]
    fn test_notification_expiry() {
        let mut n = Notification::new(NotificationLevel::Info, "Title", "Message")
            .with_duration(Duration::from_nanos(1));
        n.mark_displayed();
        std::thread::sleep(Duration::from_millis(1));
        assert!(n.is_expired());
    }
}
