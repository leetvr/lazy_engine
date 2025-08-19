use std::{ffi::CString, str::FromStr};

use engine::{Engine, TickData};

#[derive(Default, Debug, Clone)]
struct State {
    herps: usize,
}

/// Example gameplay system
fn my_system(tick: &mut TickData) -> anyhow::Result<()> {
    let state = tick.get_state::<State>().unwrap();
    state.herps += 1;
    println!("Herps are now {}", state.herps);
    // mutate state, world, etc.
    Ok(())
}

/// Exported symbol the loader will call.
#[unsafe(no_mangle)]
pub extern "C" fn init(engine_ptr: *mut Engine) {
    // blegh
    let version = CString::from_str(engine::VERSION).unwrap();
    let engine = unsafe { Engine::from_ptr(engine_ptr, version.as_ptr()) }.unwrap();
    engine.register_system("my_system", my_system);
    engine.insert_state(State::default());
}
