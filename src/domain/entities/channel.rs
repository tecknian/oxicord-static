//! Discord channel entity.

use serde::{Deserialize, Serialize};

use super::GuildId;

/// Unique identifier for a Discord channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub u64);

impl ChannelId {
    /// Returns the underlying u64 value.
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ChannelId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<&str> for ChannelId {
    fn from(value: &str) -> Self {
        Self(value.parse().unwrap_or(0))
    }
}

/// Discord channel type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum ChannelKind {
    /// Text channel.
    #[default]
    Text = 0,
    /// Direct message channel.
    Dm = 1,
    /// Voice channel.
    Voice = 2,
    /// Group direct message channel.
    GroupDm = 3,
    /// Category channel.
    Category = 4,
    /// Announcement channel.
    Announcement = 5,
    /// Announcement thread channel.
    AnnouncementThread = 10,
    /// Public thread channel.
    PublicThread = 11,
    /// Private thread channel.
    PrivateThread = 12,
    /// Stage voice channel.
    StageVoice = 13,
    /// Directory channel.
    Directory = 14,
    /// Forum channel.
    Forum = 15,
    /// Media channel.
    Media = 16,
}

impl ChannelKind {
    /// Returns true if this channel type supports text messages.
    #[must_use]
    pub fn is_text_based(self) -> bool {
        matches!(
            self,
            Self::Text
                | Self::Dm
                | Self::GroupDm
                | Self::Announcement
                | Self::AnnouncementThread
                | Self::PublicThread
                | Self::PrivateThread
        )
    }

    /// Returns true if this is a category channel.
    #[must_use]
    pub fn is_category(self) -> bool {
        matches!(self, Self::Category)
    }

    /// Returns true if this is a voice channel.
    #[must_use]
    pub fn is_voice(self) -> bool {
        matches!(self, Self::Voice | Self::StageVoice)
    }

    /// Returns the display prefix for this channel type.
    #[must_use]
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Voice | Self::StageVoice => "🔊",
            Self::Dm | Self::GroupDm => "@",
            Self::Category => "",
            Self::Forum | Self::Media => "📋",
            _ => "#",
        }
    }
}

impl From<u8> for ChannelKind {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Dm,
            2 => Self::Voice,
            3 => Self::GroupDm,
            4 => Self::Category,
            5 => Self::Announcement,
            10 => Self::AnnouncementThread,
            11 => Self::PublicThread,
            12 => Self::PrivateThread,
            13 => Self::StageVoice,
            14 => Self::Directory,
            15 => Self::Forum,
            16 => Self::Media,
            _ => Self::Text,
        }
    }
}

/// Discord channel information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Channel {
    id: ChannelId,
    guild_id: Option<GuildId>,
    name: String,
    kind: ChannelKind,
    parent_id: Option<ChannelId>,
    position: i32,
    topic: Option<String>,
    has_unread: bool,
}

impl Channel {
    /// Creates a new channel with the given ID, name, and type.
    #[must_use]
    pub fn new(id: impl Into<ChannelId>, name: impl Into<String>, kind: ChannelKind) -> Self {
        Self {
            id: id.into(),
            guild_id: None,
            name: name.into(),
            kind,
            parent_id: None,
            position: 0,
            topic: None,
            has_unread: false,
        }
    }

    /// Sets the guild ID for this channel.
    #[must_use]
    pub fn with_guild(mut self, guild_id: impl Into<GuildId>) -> Self {
        self.guild_id = Some(guild_id.into());
        self
    }

    /// Sets the parent category ID for this channel.
    #[must_use]
    pub fn with_parent(mut self, parent_id: impl Into<ChannelId>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Sets the position of this channel in the channel list.
    #[must_use]
    pub fn with_position(mut self, position: i32) -> Self {
        self.position = position;
        self
    }

    /// Sets the topic for this channel.
    #[must_use]
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Sets whether this channel has unread messages.
    #[must_use]
    pub fn with_unread(mut self, has_unread: bool) -> Self {
        self.has_unread = has_unread;
        self
    }

    /// Returns the channel ID.
    #[must_use]
    pub fn id(&self) -> ChannelId {
        self.id
    }

    /// Returns the guild ID, if this is a guild channel.
    #[must_use]
    pub fn guild_id(&self) -> Option<GuildId> {
        self.guild_id
    }

    /// Returns the channel name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the channel type.
    #[must_use]
    pub fn kind(&self) -> ChannelKind {
        self.kind
    }

    /// Returns the parent category ID, if any.
    #[must_use]
    pub fn parent_id(&self) -> Option<ChannelId> {
        self.parent_id
    }

    /// Returns the channel position in the channel list.
    #[must_use]
    pub fn position(&self) -> i32 {
        self.position
    }

    /// Returns the channel topic, if set.
    #[must_use]
    pub fn topic(&self) -> Option<&str> {
        self.topic.as_deref()
    }

    /// Returns whether this channel has unread messages.
    #[must_use]
    pub fn has_unread(&self) -> bool {
        self.has_unread
    }

    /// Sets whether this channel has unread messages.
    pub fn set_unread(&mut self, has_unread: bool) {
        self.has_unread = has_unread;
    }

    /// Returns the display name with the channel type prefix.
    #[must_use]
    pub fn display_name(&self) -> String {
        format!("{}{}", self.kind.prefix(), self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_creation() {
        let channel = Channel::new(123_u64, "general", ChannelKind::Text);

        assert_eq!(channel.id().as_u64(), 123);
        assert_eq!(channel.name(), "general");
        assert_eq!(channel.display_name(), "#general");
    }

    #[test]
    fn test_channel_with_parent() {
        let channel = Channel::new(123_u64, "chat", ChannelKind::Text)
            .with_guild(456_u64)
            .with_parent(789_u64);

        assert_eq!(channel.guild_id(), Some(GuildId(456)));
        assert_eq!(channel.parent_id(), Some(ChannelId(789)));
    }

    #[test]
    fn test_voice_channel_display() {
        let channel = Channel::new(123_u64, "Voice", ChannelKind::Voice);
        assert_eq!(channel.display_name(), "🔊Voice");
    }

    #[test]
    fn test_channel_kind_is_text_based() {
        assert!(ChannelKind::Text.is_text_based());
        assert!(ChannelKind::Dm.is_text_based());
        assert!(!ChannelKind::Voice.is_text_based());
        assert!(!ChannelKind::Category.is_text_based());
    }
}
