use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use engine::Engine;
use libloading::{Library, Symbol};

/// # SAFETY
/// You MUST keep this struct in scope in order for your systems to execute correctly.
pub struct GameplayLib {
    #[allow(unused)]
    lib: Library,
    last_modified: SystemTime,
    lib_name: String,
    lib_path: PathBuf,
    version: usize,
}

impl GameplayLib {
    pub unsafe fn load(
        lib_path: impl AsRef<Path>,
        lib_name: &str,
        engine: &mut Engine,
    ) -> anyhow::Result<Self> {
        let lib_path = lib_path.as_ref();
        let version = 0;
        let lib = copy_and_load_lib(lib_name, lib_path, version)?;

        // Get the exported function
        let init: Symbol<unsafe extern "C" fn(*mut Engine)> = unsafe { lib.get(b"init\0") }?;

        // Call it with our engine pointer
        unsafe { init(engine as *mut Engine) };

        let last_modified = get_real_lib_last_modified(lib_name, lib_path);

        Ok(Self {
            lib_path: lib_path.into(),
            lib,
            last_modified,
            lib_name: lib_name.into(),
            version,
        })
    }

    pub unsafe fn check_reload(&mut self, engine: &mut Engine) -> anyhow::Result<()> {
        let last_modified = get_real_lib_last_modified(&self.lib_name, &self.lib_path);
        if last_modified == self.last_modified {
            return Ok(());
        }

        self.last_modified = last_modified;

        log::info!(
            "{:?} != {:?}!! HOT RELOAD TIME BABY",
            self.last_modified,
            last_modified
        );

        self.version += 1;
        let lib = copy_and_load_lib(&self.lib_name, &self.lib_path, self.version)?;

        // Get the exported function
        let reload: Symbol<unsafe extern "C" fn(*mut Engine)> = unsafe { lib.get(b"reload\0") }?;

        // Call it with our engine pointer
        unsafe { reload(engine as *mut Engine) };

        // Stash the new library
        self.lib = lib;

        Ok(())
    }
}

fn get_real_lib_last_modified(lib_name: &str, lib_path: &Path) -> SystemTime {
    let real_lib_filename = format!(
        "{}{lib_name}{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let real_lib_path = lib_path.join(&real_lib_filename);
    fs::metadata(&real_lib_path).unwrap().modified().unwrap()
}

fn copy_and_load_lib(lib_name: &str, lib_path: &Path, version: usize) -> anyhow::Result<Library> {
    let real_lib_filename = format!(
        "{}{lib_name}{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let real_lib_path = lib_path.join(&real_lib_filename);

    let versioned_lib_filename = format!(
        "{}{lib_name}_{version}{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    );
    let versioned_lib_path = lib_path.join(&versioned_lib_filename);

    // First, copy the lib to its versioned path
    log::debug!("Copying {real_lib_path:?} to {versioned_lib_filename:?}..");
    fs::copy(real_lib_path, &versioned_lib_path).unwrap();

    // Next, load the libary
    log::debug!("..done! Opening {versioned_lib_filename:?}..");
    let lib = unsafe { Library::new(&versioned_lib_path) }?;
    log::debug!("..done!");

    Ok(lib)
}
