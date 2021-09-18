use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use dlopen::symbor::Library;
use launcher_extension_api::{LauncherExtension, Result};
use launcher_extension_api::command::{CommandRegister, ExtensionCommandExecutor};


#[cfg(target_os = "linux")]
const FILE_EXTENSION: &str = "so";
#[cfg(target_os = "macos")]
const FILE_EXTENSION: &str = "dylib";
#[cfg(target_os = "windows")]
const FILE_EXTENSION: &str = "dll";

type GraalMain = unsafe extern "C" fn(c_int, *const *const c_char) -> c_int;

#[no_mangle]
pub extern "Rust" fn new_extension() -> (String, Box<dyn LauncherExtension>) {
    ("authlib_patcher".to_string(), Box::new(AuthlibPatcherExtension::default()))
}

#[derive(Default)]
struct AuthlibPatcherExtension;


impl LauncherExtension for AuthlibPatcherExtension {
    fn register_command(&self, register: &mut CommandRegister) {
        register.register("patch", "Patch authlib", Box::new(AuthlibPatchCommand::default()))
    }
}

pub struct AuthlibPatchCommand {
    lib: Library,
}

impl Default for AuthlibPatchCommand {
    fn default() -> Self {
        let lib = Library::open(format!("extensions/authlib-patcher/authlib-patcher-lib.{}", FILE_EXTENSION)).expect("Can't load patcher library");
        Self {
            lib
        }
    }
}

impl ExtensionCommandExecutor for AuthlibPatchCommand {
    fn execute(&self, args: &[&str]) {
        let mut args = args.iter()
            .map(|s| CString::new(s.to_string()))
            .filter_map(Result::ok)
            .collect::<Vec<CString>>();
        args.insert(0, CString::new("").expect("Can't parse"));
        let c_args = args
            .iter()
            .map(|cs| cs.as_ptr())
            .collect::<Vec<_>>();
        let argc = c_args.len() as i32;
        let argv = c_args
            .as_ptr();
        unsafe {
            let main = self.lib.symbol::<GraalMain>("run_main").expect("Can't find main function");
            main(argc, argv);
        }
    }
}
