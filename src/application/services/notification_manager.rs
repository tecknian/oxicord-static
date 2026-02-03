use std::collections::VecDeque;
use std::time::Duration;

use crate::domain::{Notification, NotificationLevel};

#[derive(Debug)]
pub struct NotificationManager {
    queue: VecDeque<Notification>,
    default_duration: Duration,
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}

impl NotificationManager {
    #[must_use]
    pub fn new(default_duration: Duration) -> Self {
        Self {
            queue: VecDeque::new(),
            default_duration,
        }
    }

    pub fn notify(
        &mut self,
        level: NotificationLevel,
        title: impl Into<String>,
        message: impl Into<String>,
    ) {
        let notification =
            Notification::new(level, title, message).with_duration(self.default_duration);
        self.queue.push_back(notification);
    }

    pub fn info(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.notify(NotificationLevel::Info, title, message);
    }

    pub fn warn(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.notify(NotificationLevel::Warn, title, message);
    }

    pub fn error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        self.notify(NotificationLevel::Error, title, message);
    }

    pub fn tick(&mut self) {
        if let Some(front) = self.queue.front_mut() {
            front.mark_displayed();
            if front.is_expired() {
                self.queue.pop_front();
                if let Some(next) = self.queue.front_mut() {
                    next.mark_displayed();
                }
            }
        }
    }

    #[must_use]
    pub fn current_notification(&self) -> Option<&Notification> {
        self.queue.front()
    }

    #[must_use]
    pub fn has_notifications(&self) -> bool {
        !self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_notification_flow() {
        let mut manager = NotificationManager::default();

        manager.info("Info", "Test message");
        assert!(manager.current_notification().is_some());

        manager.tick();
        assert!(manager.current_notification().is_some());
    }

    #[test]
    fn test_queueing() {
        let mut manager = NotificationManager::default();
        manager.info("1", "First");
        manager.info("2", "Second");

        assert_eq!(manager.current_notification().unwrap().title, "1");

        manager.tick();

        manager.queue.front_mut().unwrap().displayed_at =
            Some(Instant::now().checked_sub(Duration::from_secs(10)).unwrap());

        manager.tick();

        assert_eq!(manager.current_notification().unwrap().title, "2");

        let second = manager.current_notification().unwrap();
        assert!(second.displayed_at.unwrap().elapsed() < Duration::from_secs(1));
    }
}
