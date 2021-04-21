use launcher_extension_api::{LauncherExtension, Result, Context};
use launcher_extension_api::launcher::message::{ClientMessage, ServerMessage};
use launcher_extension_api::connection::Client;
use crate::config::{Blocker, Config};
use std::sync::Mutex;
use launcher_extension_api::launcher::config::Configurable;

mod config;

#[no_mangle]
pub extern "Rust" fn new_extension() -> (String, Box<dyn LauncherExtension>) {
    ("auth_blocker".to_string(), Box::new(AuthBlockerExtension::default()))
}

struct AuthBlockerExtension {
    blocker: Mutex<Option<Blocker>>
}

impl Default for AuthBlockerExtension {
    fn default() -> Self {
        AuthBlockerExtension {
            blocker: Mutex::new(None)
        }
    }
}

impl LauncherExtension for AuthBlockerExtension {
    fn init(&self) -> Result<()> {
        let mut result = self.blocker.lock().ok().context("Can't init blocker")?;
        *result = Some(Config::get_config("config/blocker.json".as_ref())?.get_blocker());
        Ok(())
    }

    fn handle_message(&self, message: &ClientMessage, client: &mut Client) -> Result<Option<ServerMessage>> {
        let _ = match message {
            ClientMessage::Auth(auth) => {
                auth
            }
            _ => return Ok(None)
        };
        let mut guard = self.blocker.lock().map_err(|_| launcher_extension_api::anyhow!("Can't lock blocker"))?;
        let blocker = guard.as_mut().context("Can't found blocker")?;
        if !blocker.limit(&client.ip) {
            Ok(None)
        } else {
            Err(launcher_extension_api::anyhow!("Limit!!!"))
        }
    }
}