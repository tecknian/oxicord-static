//! Main application orchestrator.

use std::sync::Arc;

use crossterm::event::{Event, KeyEvent};
use ratatui::{DefaultTerminal, Frame};
use tracing::{debug, error, info};

use crate::application::dto::{LoginRequest, TokenSource};
use crate::application::use_cases::{LoginUseCase, ResolveTokenUseCase};
use crate::domain::entities::User;
use crate::domain::errors::AuthError;
use crate::domain::ports::{AuthPort, TokenStoragePort};
use crate::presentation::events::{EventHandler, EventResult};
use crate::presentation::ui::{LoginScreen, MainScreen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppState {
    Login,
    Main,
    Exiting,
}

enum CurrentScreen {
    Login(LoginScreen),
    Main(MainScreen),
}

/// Main application.
pub struct App {
    state: AppState,
    screen: CurrentScreen,
    login_use_case: LoginUseCase,
    resolve_token_use_case: ResolveTokenUseCase,
    event_handler: EventHandler,
    pending_token: Option<(String, TokenSource)>,
}

impl App {
    /// Creates new application.
    #[must_use]
    pub fn new(auth_port: Arc<dyn AuthPort>, storage_port: Arc<dyn TokenStoragePort>) -> Self {
        let login_use_case = LoginUseCase::new(auth_port, storage_port.clone());
        let resolve_token_use_case = ResolveTokenUseCase::new(storage_port);

        Self {
            state: AppState::Login,
            screen: CurrentScreen::Login(LoginScreen::new()),
            login_use_case,
            resolve_token_use_case,
            event_handler: EventHandler::new(),
            pending_token: None,
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

        let request = LoginRequest::new(token, source);
        match self.login_use_case.execute(request).await {
            Ok(response) => {
                info!(user = %response.user.display_name(), "Auto-login successful");
                self.transition_to_main(response.user);
            }
            Err(e) => {
                error!(error = %e, "Auto-login failed");
                if let CurrentScreen::Login(ref mut login_screen) = self.screen {
                    login_screen.set_error(e.to_string());
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        match &self.screen {
            CurrentScreen::Login(screen) => {
                frame.render_widget(screen, frame.area());
            }
            CurrentScreen::Main(screen) => {
                frame.render_widget(screen, frame.area());
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

        match &mut self.screen {
            CurrentScreen::Login(screen) => {
                if screen.handle_key(key) {
                    self.handle_login_submit().await;
                }
            }
            CurrentScreen::Main(_) => {
                if EventHandler::is_quit_event(&key) {
                    return EventResult::Exit;
                }
            }
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

        let mut request = LoginRequest::new(token, TokenSource::UserInput);
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
                self.transition_to_main(response.user);
            }
            Err(e) => {
                error!(error = %e, "Login failed");
                self.handle_login_error(&e);
            }
        }
    }

    fn transition_to_main(&mut self, user: User) {
        self.state = AppState::Main;
        self.screen = CurrentScreen::Main(MainScreen::new(user));
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
    use crate::domain::ports::mocks::{MockAuthPort, MockTokenStorage};

    #[test]
    fn test_app_creation() {
        let auth = Arc::new(MockAuthPort::new(true));
        let storage = Arc::new(MockTokenStorage::new());
        let app = App::new(auth, storage);

        assert_eq!(app.state, AppState::Login);
    }
}
