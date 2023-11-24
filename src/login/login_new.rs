use std::fmt;

use anyhow::anyhow;
use matrix_sdk::{
    self,
    ruma::api::client::session::get_login_types::v3::{IdentityProvider, LoginType},
    Client,
};

/// The initial device name when logging in with a device for the first time.
const INITIAL_DEVICE_DISPLAY_NAME: &str = "login client";

use std::path::Path;

use tokio::fs;

use crate::{
    login::persist_session::{build_client, FullSession},
    ui_elements::{
        info_popup::{info_popup, Type},
        input_popup::input_popup,
    },
};

#[derive(Debug)]
enum LoginChoice {
    /// Login with username and password.
    Password,

    /// Login with SSO.
    Sso,

    /// Login with a specific SSO identity provider.
    SsoIdp(IdentityProvider),
}

impl LoginChoice {
    /// Login with this login choice.
    async fn login(&self, client: &Client) -> anyhow::Result<()> {
        match self {
            LoginChoice::Password => login_with_password(client).await,
            LoginChoice::Sso => login_with_sso(client, None).await,
            LoginChoice::SsoIdp(idp) => login_with_sso(client, Some(idp)).await,
        }
    }
}

impl fmt::Display for LoginChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoginChoice::Password => write!(f, "Username and password"),
            LoginChoice::Sso => write!(f, "SSO"),
            LoginChoice::SsoIdp(idp) => write!(f, "SSO via {}", idp.name),
        }
    }
}

/// Log in to the given homeserver and sync.
pub async fn login_new(data_dir: &Path, session_file: &Path) -> anyhow::Result<Client> {
    let (client, client_session) = build_client(data_dir).await?;

    let matrix_auth = client.matrix_auth();
    // First, let's figure out what login types are supported by the homeserver.
    let mut choices = Vec::new();
    let login_types = matrix_auth.get_login_types().await?.flows;

    for login_type in login_types {
        match login_type {
            LoginType::Password(_) => {
                choices.push(LoginChoice::Password)
            }
            LoginType::Sso(sso) => {
                if sso.identity_providers.is_empty() {
                    choices.push(LoginChoice::Sso)
                } else {
                    choices.extend(sso.identity_providers.into_iter().map(LoginChoice::SsoIdp))
                }
            }
            // This is used for SSO, so it's not a separate choice.
            LoginType::Token(_) |
            // This is only for application services, ignore it here.
            LoginType::ApplicationService(_) => {},
            // We don't support unknown login types.
            _ => {},
        }
    }

    match choices.len() {
        0 => {
            return Err(anyhow!(
                "Homeserver login types incompatible with this client"
            ))
        }
        1 => choices[0].login(&client).await?,
        _ => offer_choices_and_login(&client, choices).await?,
    }

    // Persist the session to reuse it later.
    // This is not very secure, for simplicity. If the system provides a way of
    // storing secrets securely, it should be used instead.
    // Note that we could also build the user session from the login response.
    let user_session = matrix_auth
        .session()
        .expect("A logged-in client should have a session");
    let serialized_session = serde_json::to_string(&FullSession {
        client_session,
        user_session,
        sync_token: None,
    })?;
    fs::write(session_file, serialized_session).await?;

    println!("Session persisted in {}", session_file.to_string_lossy());

    // After logging in, you might want to verify this session with another one (see
    // the `emoji_verification` example), or bootstrap cross-signing if this is your
    // first session with encryption, or if you need to reset cross-signing because
    // you don't have access to your old sessions (see the
    // `cross_signing_bootstrap` example).

    Ok(client)
}

/// Offer the given choices to the user and login with the selected option.
async fn offer_choices_and_login(client: &Client, choices: Vec<LoginChoice>) -> anyhow::Result<()> {
    let choice = loop {
        let mut body = vec!["Several options are available to login with this homeserver:".into()];
        for (idx, login_choice) in choices.iter().enumerate() {
            body.push(format!("{idx}) {login_choice}"));
        }
        let header = "Enter your choice:";

        let choice_str = input_popup(header, body.join("\n").as_str())?;
        match choice_str.trim().parse::<usize>() {
            Ok(choice) => {
                if choice >= choices.len() {
                    info_popup(Type::Error, "Error", "This is not a valid choice")?;
                } else {
                    break choice;
                }
            }
            Err(_) => info_popup(
                Type::Error,
                "Error",
                "This is not a valid choice. Try again.",
            )?,
        };
    };

    choices[choice].login(client).await?;

    Ok(())
}

/// Login with a username and password.
async fn login_with_password(client: &Client) -> anyhow::Result<()> {
    let body = "Logging in with username and password…";

    loop {
        let header = "Username:";
        let username = input_popup(header, body)?.trim().to_owned();

        let header = "Password:";
        let password = input_popup(header, body)?.trim().to_owned();

        match client
            .matrix_auth()
            .login_username(&username, &password)
            .initial_device_display_name(INITIAL_DEVICE_DISPLAY_NAME)
            .await
        {
            Ok(_) => {
                info_popup(
                    Type::Informaton,
                    "Login successful!",
                    format!("Logged in as {username}").as_str(),
                )?;
                break;
            }
            Err(_error) => info_popup(Type::Error, "Error", "Please try again.")?,
        }
    }

    Ok(())
}
/// Login with SSO.
async fn login_with_sso(client: &Client, idp: Option<&IdentityProvider>) -> anyhow::Result<()> {
    println!("Logging in with SSO…");

    let mut login_builder = client.matrix_auth().login_sso(|url| async move {
        open::that(&url)?;

        println!("\nOpen this URL in your browser: {url}\n");
        println!("Waiting for login token…");
        Ok(())
    });

    if let Some(idp) = idp {
        login_builder = login_builder.identity_provider_id(&idp.id);
    }

    let _response = login_builder.send().await?;
    // auth.restore_session((&response).into()).await?;

    println!("Logged in as {}", client.user_id().unwrap());

    Ok(())
}
