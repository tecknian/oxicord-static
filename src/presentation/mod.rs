//! Presentation layer with UI components and event handling.

/// Command handling.
pub mod commands;
/// Event handling.
pub mod events;
/// UI screens.
pub mod ui;
/// Reusable widgets.
pub mod widgets;

pub use ui::App;
