use clap::{Args, Subcommand};

use crate::auth::{oauth, token_store};
use crate::config::Config;
use crate::error::Result;

#[derive(Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub action: AuthAction,
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Authenticate with Google (opens browser)
    Login,
    /// Remove saved credentials
    Logout,
    /// Show authentication status
    Status,
}

pub async fn handle(args: AuthArgs, config: &Config) -> Result<()> {
    match args.action {
        AuthAction::Login => {
            oauth::login(config).await?;
        }
        AuthAction::Logout => {
            if config.token_path.exists() {
                std::fs::remove_file(&config.token_path)?;
                eprintln!("Token removed.");
            } else {
                eprintln!("No token found.");
            }
        }
        AuthAction::Status => match token_store::load_token(&config.token_path)? {
            Some(token) => {
                if token.is_expired() {
                    eprintln!("Status: Token expired (will refresh on next request)");
                } else {
                    eprintln!("Status: Authenticated");
                }
                if let Some(expires_at) = token.expires_at {
                    eprintln!("Expires: {expires_at}");
                }
                eprintln!("Has refresh token: {}", token.refresh_token.is_some());
            }
            None => {
                eprintln!("Status: Not authenticated");
                eprintln!("Run `gtm auth login` to authenticate.");
            }
        },
    }
    Ok(())
}
