use launcher_extension_api::LauncherExtension;
use launcher_extension_api::command::{CommandRegister};
use crate::command::ProfileGenerationCommand;

mod command;
mod artifact;
mod download;
mod minecraft;
mod util;
mod validator;
mod generator;

#[no_mangle]
pub extern "Rust" fn new_extension() -> (String, Box<dyn LauncherExtension>) {
    ("profile_generator".to_string(), Box::new(ProfileGeneratorExtension))
}

struct ProfileGeneratorExtension;

impl LauncherExtension for ProfileGeneratorExtension {
    fn register_command(&self, register: &mut CommandRegister) {
        register.register("profilegen", "Generate profile", Box::new(ProfileGenerationCommand::default()))
    }
}
