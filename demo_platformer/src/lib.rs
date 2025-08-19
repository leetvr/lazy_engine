use std::{ffi::CString, str::FromStr};

use engine::{Engine, TickData};

#[derive(Default)]
struct FakeComponent {
    derps: usize,
}

/// Example gameplay system
fn my_system(tick: &mut TickData) -> anyhow::Result<()> {
    let herps = tick.get_state::<usize>().unwrap();
    *herps += 1;

    for (_, fake) in tick.world.query::<&mut FakeComponent>().iter() {
        fake.derps += 5;
    }

    Ok(())
}

/// Called by the loader on init
#[unsafe(no_mangle)]
pub extern "C" fn init(engine_ptr: *mut Engine) {
    println!("INIT!");
    let engine = get_engine(engine_ptr);
    engine.register_system("my_system", my_system);
    engine.insert_state(0 as usize);

    engine.world_mut().spawn((FakeComponent::default(),));
    engine.world_mut().spawn((FakeComponent::default(),));
    engine.world_mut().spawn((FakeComponent::default(),));
}

/// Called by the loader on reload
#[unsafe(no_mangle)]
pub extern "C" fn reload(engine_ptr: *mut Engine) {
    println!("RELOAD!");
    let engine = get_engine(engine_ptr);
    // overwrites previous one
    engine.register_system("my_system", my_system);
}

fn get_engine<'a>(engine_ptr: *mut Engine) -> &'a mut Engine {
    // blegh
    let version = CString::from_str(engine::VERSION).unwrap();
    let engine = unsafe { Engine::from_ptr(engine_ptr, version.as_ptr()) }.unwrap();
    engine
}
