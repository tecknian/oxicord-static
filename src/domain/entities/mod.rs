//! Domain entity definitions.

mod channel;
mod forum;
mod guild;
#[cfg(feature = "image")]
mod image;
#[cfg(not(feature = "image"))]
mod image_stub;

mod member;
mod message;
mod permissions;
mod read_state;
mod relationship;
mod role;
mod token;
mod user;
mod user_cache;

pub use channel::{
    Channel, ChannelFlags, ChannelId, ChannelKind, OverwriteType, PermissionOverwrite,
    ThreadMetadata, VideoQualityMode,
};
pub use forum::ForumThread;
pub use guild::{Guild, GuildFolder, GuildId, NsfwLevel, PremiumTier, VerificationLevel};
#[cfg(feature = "image")]
pub use image::{ImageId, ImageMetadata, ImageSource, ImageStatus, LoadedImage};

#[cfg(not(feature = "image"))]
pub use image_stub::{ImageId, ImageMetadata, ImageSource, ImageStatus, LoadedImage};

pub use member::Member;
pub use message::{
    Attachment, Embed, EmbedAuthor, EmbedField, EmbedFooter, EmbedImage, EmbedProvider,
    EmbedThumbnail, EmbedVideo, Message, MessageAuthor, MessageFlags, MessageId, MessageKind,
    MessageReference, Reaction, ReactionEmoji,
};
pub use permissions::Permissions;
pub use read_state::ReadState;
pub use relationship::{Relationship, RelationshipState, RelationshipType};
pub use role::{Role, RoleId};
pub use token::AuthToken;
pub use user::{PremiumType, User, UserFlags, UserId};
pub use user_cache::{CachedUser, UserCache};
