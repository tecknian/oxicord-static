//! System notifications with conditional compilation.

use crate::domain::ports::NotificationPort;

/// Desktop notification service.
#[cfg(feature = "notify")]
mod notify_impl {
    use super::*;
    use notify_rust::Notification;

    #[derive(Debug, Clone, Default)]
    pub struct DesktopNotificationService {
        enabled: bool,
    }

    impl DesktopNotificationService {
        #[must_use]
        pub fn new(enabled: bool) -> Self {
            Self { enabled }
        }
    }

    impl NotificationPort for DesktopNotificationService {
        fn send(&self, title: &str, body: &str) {
            if !self.enabled {
                return;
            }

            let title = title.to_string();
            let body = body.to_string();

            tokio::task::spawn_blocking(move || {
                if let Err(e) = Notification::new()
                    .summary(&title)
                    .body(&body)
                    .appname("Oxicord")
                    .show()
                {
                    tracing::warn!("Failed to show notification: {}", e);
                }
            });
        }
    }
}

/// Stub notification service when notify feature is disabled.
#[cfg(not(feature = "notify"))]
mod stub_impl {
    use super::*;

    #[derive(Debug, Clone, Default)]
    pub struct DesktopNotificationService {
        _enabled: bool,
    }

    impl DesktopNotificationService {
        #[must_use]
        pub fn new(_enabled: bool) -> Self {
            Self { _enabled: false }
        }
    }

    impl NotificationPort for DesktopNotificationService {
        fn send(&self, _title: &str, _body: &str) {
            // Notifications disabled - do nothing
        }
    }
}

#[cfg(feature = "notify")]
pub use notify_impl::DesktopNotificationService;
#[cfg(not(feature = "notify"))]
pub use stub_impl::DesktopNotificationService;
