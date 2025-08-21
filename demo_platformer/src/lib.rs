use std::{any::TypeId, ffi::CString, str::FromStr};

use engine::{Engine, TickData, components::Transform};
use glam::Quat;

/// Example gameplay system
fn my_system(tick: &mut TickData) -> anyhow::Result<()> {
    for (_, transform) in tick.world.query::<&mut Transform>().iter() {
        transform.rotation *= Quat::from_rotation_y(0.02);
        transform.position.x += 0.001;
    }

    Ok(())
}

/// Called by the loader on init
#[unsafe(no_mangle)]
pub extern "C" fn init(engine_ptr: *mut Engine) {
    println!("INIT!");
    let transform_type_id = TypeId::of::<engine_types::components::Transform>();
    println!("INIT: Transform type_id: {transform_type_id:?}");
    let engine = get_engine(engine_ptr);
    engine.register_system("my_system", my_system);
    engine.insert_state(0 as usize);
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
