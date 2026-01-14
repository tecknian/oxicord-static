//! Main application orchestrator.

use std::sync::Arc;

use crossterm::event::{Event, KeyEvent};
use ratatui::{DefaultTerminal, Frame};
use tracing::{debug, error, info, warn};

use crate::application::dto::{LoginRequest, TokenSource};
use crate::application::use_cases::{LoginUseCase, ResolveTokenUseCase};
use crate::domain::entities::{AuthToken, ChannelId, GuildId, User};
use crate::domain::errors::AuthError;
use crate::domain::ports::{AuthPort, DiscordDataPort, FetchMessagesOptions, TokenStoragePort};
use crate::presentation::events::{EventHandler, EventResult};
use crate::presentation::ui::{ChatKeyResult, ChatScreen, ChatScreenState, LoginScreen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Login,
    Chat,
    Exiting,
}

enum CurrentScreen {
    Login(LoginScreen),
    Chat(Box<ChatScreenState>),
}

/// Main application.
pub struct App {
    state: AppState,
    screen: CurrentScreen,
    login_use_case: LoginUseCase,
    resolve_token_use_case: ResolveTokenUseCase,
    discord_data: Arc<dyn DiscordDataPort>,
    event_handler: EventHandler,
    pending_token: Option<(String, TokenSource)>,
    current_token: Option<AuthToken>,
}

impl App {
    /// Creates new application.
    #[must_use]
    pub fn new(
        auth_port: Arc<dyn AuthPort>,
        discord_data: Arc<dyn DiscordDataPort>,
        storage_port: Arc<dyn TokenStoragePort>,
    ) -> Self {
        let login_use_case = LoginUseCase::new(auth_port, storage_port.clone());
        let resolve_token_use_case = ResolveTokenUseCase::new(storage_port);

        Self {
            state: AppState::Login,
            screen: CurrentScreen::Login(LoginScreen::new()),
            login_use_case,
            resolve_token_use_case,
            discord_data,
            event_handler: EventHandler::new(),
            pending_token: None,
            current_token: None,
        }
    }

    /// Runs the application.
    ///
    /// # Errors
    /// Returns error if terminal or token resolution fails.
    pub async fn run(
        mut self,
        terminal: &mut DefaultTerminal,
        cli_token: Option<String>,
    ) -> color_eyre::Result<()> {
        if let Some(resolved) = self.resolve_token_use_case.execute(cli_token).await? {
            info!(source = %resolved.source, "Found existing token");
            self.pending_token = Some((resolved.token.as_str().to_string(), resolved.source));
        }

        if let Some((token, source)) = self.pending_token.take() {
            self.attempt_auto_login(token, source).await;
        }

        while self.state != AppState::Exiting {
            terminal.draw(|frame| self.render(frame))?;

            if let Some(event) = self.event_handler.poll()? {
                self.handle_event(event).await;
            }
        }

        info!("Application exiting normally");
        Ok(())
    }

    async fn attempt_auto_login(&mut self, token: String, source: TokenSource) {
        debug!("Attempting automatic login");

        if let CurrentScreen::Login(ref mut login_screen) = self.screen {
            login_screen.set_validating();
        }

        let request = LoginRequest::new(token.clone(), source);
        match self.login_use_case.execute(request).await {
            Ok(response) => {
                info!(user = %response.user.display_name(), "Auto-login successful");
                if let Some(auth_token) = AuthToken::new(&token) {
                    self.current_token = Some(auth_token);
                }
                self.transition_to_chat(response.user).await;
            }
            Err(e) => {
                error!(error = %e, "Auto-login failed");
                if let CurrentScreen::Login(ref mut login_screen) = self.screen {
                    login_screen.set_error(e.to_string());
                }
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        match &mut self.screen {
            CurrentScreen::Login(screen) => {
                frame.render_widget(&*screen, frame.area());
            }
            CurrentScreen::Chat(state) => {
                frame.render_stateful_widget(ChatScreen::new(), frame.area(), state);
            }
        }
    }

    async fn handle_event(&mut self, event: Event) {
        let result = match event {
            Event::Key(key) => self.handle_key(key).await,
            _ => EventResult::Continue,
        };

        if result == EventResult::Exit {
            self.state = AppState::Exiting;
        }
    }

    async fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if EventHandler::is_quit_event(&key) && self.state == AppState::Login {
            return EventResult::Exit;
        }

        let result = match &mut self.screen {
            CurrentScreen::Login(screen) => {
                if screen.handle_key(key) {
                    self.handle_login_submit().await;
                }
                return EventResult::Continue;
            }
            CurrentScreen::Chat(state) => state.handle_key(key),
        };

        match result {
            ChatKeyResult::Quit => return EventResult::Exit,
            ChatKeyResult::Logout => {
                self.transition_to_login();
            }
            ChatKeyResult::CopyToClipboard(text) => {
                debug!(text = %text, "Copy to clipboard requested");
            }
            ChatKeyResult::LoadGuildChannels(guild_id) => {
                self.load_guild_channels(guild_id).await;
            }
            ChatKeyResult::LoadChannelMessages(channel_id) => {
                self.load_channel_messages(channel_id).await;
            }
            ChatKeyResult::ReplyToMessage {
                message_id,
                mention,
            } => {
                debug!(message_id = %message_id, mention = mention, "Reply to message requested");
            }
            ChatKeyResult::EditMessage(message_id) => {
                debug!(message_id = %message_id, "Edit message requested");
            }
            ChatKeyResult::DeleteMessage(message_id) => {
                debug!(message_id = %message_id, "Delete message requested");
            }
            ChatKeyResult::OpenAttachments(message_id) => {
                debug!(message_id = %message_id, "Open attachments requested");
            }
            ChatKeyResult::JumpToMessage(message_id) => {
                debug!(message_id = %message_id, "Jump to message requested");
            }
            ChatKeyResult::Consumed => {}
        }

        EventResult::Continue
    }

    async fn handle_login_submit(&mut self) {
        let (token, persist) = if let CurrentScreen::Login(ref screen) = self.screen {
            match screen.token() {
                Some(t) => (t.to_string(), screen.should_persist()),
                None => return,
            }
        } else {
            return;
        };

        if let CurrentScreen::Login(ref mut screen) = self.screen {
            screen.set_validating();
        }

        let mut request = LoginRequest::new(token.clone(), TokenSource::UserInput);
        if !persist {
            request = request.without_persistence();
        }

        match self.login_use_case.execute(request).await {
            Ok(response) => {
                info!(
                    user = %response.user.display_name(),
                    persisted = response.token_persisted,
                    "Login successful"
                );
                if let Some(auth_token) = AuthToken::new(&token) {
                    self.current_token = Some(auth_token);
                }
                self.transition_to_chat(response.user).await;
            }
            Err(e) => {
                error!(error = %e, "Login failed");
                self.handle_login_error(&e);
            }
        }
    }

    async fn transition_to_chat(&mut self, user: User) {
        self.state = AppState::Chat;
        let mut chat_state = ChatScreenState::new(user);

        if let Some(ref token) = self.current_token {
            self.load_discord_data(&mut chat_state, token).await;
        }

        self.screen = CurrentScreen::Chat(Box::new(chat_state));
    }

    async fn load_discord_data(&self, state: &mut ChatScreenState, token: &AuthToken) {
        let guilds_future = self.discord_data.fetch_guilds(token);
        let dms_future = self.discord_data.fetch_dm_channels(token);

        let (guilds_result, dms_result) = tokio::join!(guilds_future, dms_future);

        if let Ok(dm_channels) = dms_result {
            let dm_users: Vec<(String, String)> = dm_channels
                .into_iter()
                .map(|dm| (dm.channel_id, dm.recipient_name))
                .collect();
            state.set_dm_users(dm_users);
            debug!(
                count = state.guilds_tree_data().dm_users().len(),
                "Loaded DM channels"
            );
        }

        match guilds_result {
            Ok(guilds) => {
                info!(count = guilds.len(), "Loaded guilds from Discord");
                state.set_guilds(guilds);
            }
            Err(e) => {
                warn!(error = %e, "Failed to load guilds from Discord");
            }
        }
    }

    async fn load_guild_channels(&mut self, guild_id: GuildId) {
        let channels = if let Some(ref token) = self.current_token {
            match self
                .discord_data
                .fetch_channels(token, guild_id.as_u64())
                .await
            {
                Ok(channels) => {
                    debug!(guild_id = %guild_id, count = channels.len(), "Loaded channels for guild");
                    Some(channels)
                }
                Err(e) => {
                    warn!(guild_id = %guild_id, error = %e, "Failed to load channels for guild");
                    None
                }
            }
        } else {
            None
        };

        if let (Some(channels), CurrentScreen::Chat(state)) = (channels, &mut self.screen) {
            state.set_channels(guild_id, channels);
        }
    }

    async fn load_channel_messages(&mut self, channel_id: ChannelId) {
        let messages = if let Some(ref token) = self.current_token {
            let options = FetchMessagesOptions::default().with_limit(50);
            match self
                .discord_data
                .fetch_messages(token, channel_id.as_u64(), options)
                .await
            {
                Ok(messages) => {
                    debug!(channel_id = %channel_id, count = messages.len(), "Loaded messages for channel");
                    Some(messages)
                }
                Err(e) => {
                    warn!(channel_id = %channel_id, error = %e, "Failed to load messages for channel");
                    if let CurrentScreen::Chat(state) = &mut self.screen {
                        state.set_message_error(e.to_string());
                    }
                    None
                }
            }
        } else {
            None
        };

        if let (Some(messages), CurrentScreen::Chat(state)) = (messages, &mut self.screen) {
            state.set_messages(messages);
        }
    }

    fn transition_to_login(&mut self) {
        self.state = AppState::Login;
        self.current_token = None;
        self.screen = CurrentScreen::Login(LoginScreen::new());
    }

    fn handle_login_error(&mut self, error: &AuthError) {
        if let CurrentScreen::Login(ref mut screen) = self.screen {
            let message = match error {
                AuthError::InvalidTokenFormat { .. } => {
                    "Invalid token format. Please check your token.".to_string()
                }
                AuthError::TokenRejected { .. } => {
                    "Token rejected. It may be invalid or expired.".to_string()
                }
                AuthError::NetworkError { message } => {
                    format!("Network error: {message}")
                }
                AuthError::RateLimited { retry_after_ms } => {
                    format!("Rate limited. Try again in {}s", retry_after_ms / 1000)
                }
                _ => error.to_string(),
            };
            screen.set_error(message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::Guild;
    use crate::domain::ports::{
        DirectMessageChannel, FetchMessagesOptions,
        mocks::{MockAuthPort, MockTokenStorage},
    };

    struct MockDiscordData;

    #[async_trait::async_trait]
    impl DiscordDataPort for MockDiscordData {
        async fn fetch_guilds(&self, _token: &AuthToken) -> Result<Vec<Guild>, AuthError> {
            Ok(vec![Guild::new(1_u64, "Test Guild")])
        }

        async fn fetch_channels(
            &self,
            _token: &AuthToken,
            _guild_id: u64,
        ) -> Result<Vec<crate::domain::entities::Channel>, AuthError> {
            Ok(vec![])
        }

        async fn fetch_dm_channels(
            &self,
            _token: &AuthToken,
        ) -> Result<Vec<DirectMessageChannel>, AuthError> {
            Ok(vec![])
        }

        async fn fetch_messages(
            &self,
            _token: &AuthToken,
            _channel_id: u64,
            _options: FetchMessagesOptions,
        ) -> Result<Vec<crate::domain::entities::Message>, AuthError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_app_creation() {
        let auth = Arc::new(MockAuthPort::new(true));
        let data = Arc::new(MockDiscordData);
        let storage = Arc::new(MockTokenStorage::new());
        let app = App::new(auth, data, storage);

        assert_eq!(app.state, AppState::Login);
    }
}
