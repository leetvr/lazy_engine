use std::path::Path;

use engine::Engine;
use libloading::{Library, Symbol};

/// # SAFETY
/// You MUST keep this struct in scope in order for your systems to execute correctly.
pub struct GameplayLib {
    #[allow(unused)]
    lib: Library,
}

impl GameplayLib {
    pub unsafe fn load(
        path: impl AsRef<Path>,
        lib_name: &str,
        engine: &mut Engine,
    ) -> anyhow::Result<Self> {
        let lib_filename = format!(
            "{}{lib_name}{}",
            std::env::consts::DLL_PREFIX,
            std::env::consts::DLL_SUFFIX
        );
        let lib_path = path.as_ref().join(&lib_filename);
        let lib = unsafe { Library::new(lib_path) }?;
        // get the exported function
        let init: Symbol<unsafe extern "C" fn(*mut Engine)> = unsafe { lib.get(b"init\0") }?;
        // call it with our engine pointer
        unsafe { init(engine as *mut Engine) };
        Ok(Self { lib })
    }
}
