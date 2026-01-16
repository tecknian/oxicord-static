#[derive(Debug, Clone, Default)]
pub struct SessionInfo {
    session_id: Option<String>,
    resume_gateway_url: Option<String>,
    sequence: Option<u64>,
    user_id: Option<String>,
}

impl SessionInfo {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            session_id: None,
            resume_gateway_url: None,
            sequence: None,
            user_id: None,
        }
    }

    pub fn set_session(&mut self, session_id: String, resume_url: Option<String>) {
        self.session_id = Some(session_id);
        self.resume_gateway_url = resume_url;
    }

    pub const fn set_sequence(&mut self, sequence: u64) {
        self.sequence = Some(sequence);
    }

    pub const fn update_sequence(&mut self, sequence: Option<u64>) {
        if let Some(seq) = sequence {
            self.sequence = Some(seq);
        }
    }

    pub fn set_user_id(&mut self, user_id: String) {
        self.user_id = Some(user_id);
    }

    #[must_use]
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    #[must_use]
    pub fn resume_gateway_url(&self) -> Option<&str> {
        self.resume_gateway_url.as_deref()
    }

    #[must_use]
    pub const fn sequence(&self) -> Option<u64> {
        self.sequence
    }

    #[must_use]
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    #[must_use]
    pub const fn can_resume(&self) -> bool {
        self.session_id.is_some() && self.sequence.is_some()
    }

    pub fn clear(&mut self) {
        self.session_id = None;
        self.resume_gateway_url = None;
        self.sequence = None;
    }

    pub fn clear_all(&mut self) {
        self.clear();
        self.user_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_info_creation() {
        let session = SessionInfo::new();
        assert!(session.session_id().is_none());
        assert!(!session.can_resume());
    }

    #[test]
    fn test_session_can_resume() {
        let mut session = SessionInfo::new();
        session.set_session("test_session".into(), Some("wss://resume.url".into()));
        session.set_sequence(42);

        assert!(session.can_resume());
        assert_eq!(session.session_id(), Some("test_session"));
        assert_eq!(session.sequence(), Some(42));
    }

    #[test]
    fn test_session_clear() {
        let mut session = SessionInfo::new();
        session.set_session("test".into(), None);
        session.set_sequence(1);
        session.set_user_id("user123".into());

        session.clear();
        assert!(session.session_id().is_none());
        assert_eq!(session.user_id(), Some("user123"));

        session.clear_all();
        assert!(session.user_id().is_none());
    }
}
