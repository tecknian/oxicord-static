//! Discord relationship entity.
//!
//! Tracks user relationships (friends, blocked, pending, etc.) from Gateway events.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;

use super::UserId;

/// Discord relationship types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RelationshipType {
    None = 0,
    Friend = 1,
    Blocked = 2,
    PendingIncoming = 3,
    PendingOutgoing = 4,
}

impl From<u8> for RelationshipType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Friend,
            2 => Self::Blocked,
            3 => Self::PendingIncoming,
            4 => Self::PendingOutgoing,
            _ => Self::None,
        }
    }
}

impl RelationshipType {
    #[must_use]
    pub const fn is_blocked(self) -> bool {
        matches!(self, Self::Blocked)
    }
}

/// A single relationship entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub user_id: UserId,
    pub relationship_type: RelationshipType,
}

impl Relationship {
    #[must_use]
    pub const fn new(user_id: UserId, relationship_type: RelationshipType) -> Self {
        Self {
            user_id,
            relationship_type,
        }
    }

    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        self.relationship_type.is_blocked()
    }
}

/// Thread-safe state manager for tracking blocked user IDs.
///
/// Uses a `RwLock<HashSet<UserId>>` for O(1) lookup performance.
/// This follows the Observer pattern - Gateway events update this state,
/// and the `MessagePane` queries it during rendering.
#[derive(Debug, Clone, Default)]
pub struct RelationshipState {
    blocked_users: Arc<RwLock<HashSet<UserId>>>,
}

impl RelationshipState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            blocked_users: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    #[must_use]
    pub fn with_blocked_users(blocked: HashSet<UserId>) -> Self {
        Self {
            blocked_users: Arc::new(RwLock::new(blocked)),
        }
    }

    #[must_use]
    pub fn is_blocked(&self, user_id: UserId) -> bool {
        self.blocked_users.read().contains(&user_id)
    }

    #[must_use]
    pub fn is_blocked_str(&self, user_id: &str) -> bool {
        if let Ok(id) = user_id.parse::<u64>() {
            self.is_blocked(UserId(id))
        } else {
            false
        }
    }

    pub fn block_user(&self, user_id: UserId) {
        self.blocked_users.write().insert(user_id);
    }

    pub fn unblock_user(&self, user_id: UserId) {
        self.blocked_users.write().remove(&user_id);
    }

    pub fn update_relationship(&self, user_id: UserId, relationship_type: RelationshipType) {
        if relationship_type.is_blocked() {
            self.block_user(user_id);
        } else {
            self.unblock_user(user_id);
        }
    }

    pub fn clear(&self) {
        self.blocked_users.write().clear();
    }

    #[must_use]
    pub fn blocked_count(&self) -> usize {
        self.blocked_users.read().len()
    }

    pub fn initialize_from_relationships(&self, relationships: &[Relationship]) {
        let mut blocked = self.blocked_users.write();
        blocked.clear();
        for rel in relationships {
            if rel.is_blocked() {
                blocked.insert(rel.user_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_type_from_u8() {
        assert_eq!(RelationshipType::from(0), RelationshipType::None);
        assert_eq!(RelationshipType::from(1), RelationshipType::Friend);
        assert_eq!(RelationshipType::from(2), RelationshipType::Blocked);
        assert_eq!(RelationshipType::from(3), RelationshipType::PendingIncoming);
        assert_eq!(RelationshipType::from(4), RelationshipType::PendingOutgoing);
        assert_eq!(RelationshipType::from(255), RelationshipType::None);
    }

    #[test]
    fn test_relationship_is_blocked() {
        assert!(RelationshipType::Blocked.is_blocked());
        assert!(!RelationshipType::Friend.is_blocked());
        assert!(!RelationshipType::None.is_blocked());
    }

    #[test]
    fn test_relationship_state_block_unblock() {
        let state = RelationshipState::new();
        let user_id = UserId(12345);

        assert!(!state.is_blocked(user_id));

        state.block_user(user_id);
        assert!(state.is_blocked(user_id));
        assert_eq!(state.blocked_count(), 1);

        state.unblock_user(user_id);
        assert!(!state.is_blocked(user_id));
        assert_eq!(state.blocked_count(), 0);
    }

    #[test]
    fn test_relationship_state_update() {
        let state = RelationshipState::new();
        let user_id = UserId(12345);

        state.update_relationship(user_id, RelationshipType::Blocked);
        assert!(state.is_blocked(user_id));

        state.update_relationship(user_id, RelationshipType::Friend);
        assert!(!state.is_blocked(user_id));
    }

    #[test]
    fn test_relationship_state_is_blocked_str() {
        let state = RelationshipState::new();
        state.block_user(UserId(12345));

        assert!(state.is_blocked_str("12345"));
        assert!(!state.is_blocked_str("99999"));
        assert!(!state.is_blocked_str("invalid"));
    }

    #[test]
    fn test_relationship_state_initialize_from_relationships() {
        let state = RelationshipState::new();
        let relationships = vec![
            Relationship::new(UserId(111), RelationshipType::Friend),
            Relationship::new(UserId(222), RelationshipType::Blocked),
            Relationship::new(UserId(333), RelationshipType::Blocked),
            Relationship::new(UserId(444), RelationshipType::PendingIncoming),
        ];

        state.initialize_from_relationships(&relationships);

        assert!(!state.is_blocked(UserId(111)));
        assert!(state.is_blocked(UserId(222)));
        assert!(state.is_blocked(UserId(333)));
        assert!(!state.is_blocked(UserId(444)));
        assert_eq!(state.blocked_count(), 2);
    }

    #[test]
    fn test_relationship_state_thread_safety() {
        use std::thread;

        let state = RelationshipState::new();
        let state_clone = state.clone();

        let handle = thread::spawn(move || {
            for i in 0..100 {
                state_clone.block_user(UserId(i));
            }
        });

        for _ in 0..50 {
            let _ = state.is_blocked(UserId(50));
        }

        handle.join().unwrap();
        assert_eq!(state.blocked_count(), 100);
    }
}
