use std::{
    any::{self, Any, TypeId},
    collections::HashMap,
    ffi::CStr,
};

type StateMap = HashMap<TypeId, Box<dyn Any>>;

pub struct Engine {
    systems: HashMap<String, SystemFn>,
    state: StateManager,
}

pub const VERSION: &str = git_version::git_version!();
pub type SystemFn = fn(&mut TickData) -> anyhow::Result<()>;

pub struct TickData<'a> {
    pub dt: f32,
    state: &'a mut StateManager,
}

impl<'a> TickData<'a> {
    pub fn get_state<S: 'static>(&mut self) -> Result<&mut S, EngineError> {
        self.state.get_state()
    }
}

#[derive(Debug, Clone)]
pub enum EngineError {
    BadPointer,
    UninitialisedState(&'static str),
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            systems: Default::default(),
            state: Default::default(),
        }
    }

    /// Will explode if you do something dumb
    pub unsafe fn from_ptr<'a>(
        ptr: *mut Engine,
        engine_version: *const std::os::raw::c_char,
    ) -> Result<&'a mut Engine, EngineError> {
        let theirs = unsafe { CStr::from_ptr(engine_version) }.to_str().unwrap();
        assert_eq!(VERSION, theirs);
        unsafe { ptr.as_mut().ok_or(EngineError::BadPointer) }
    }

    pub fn register_system(&mut self, name: impl Into<String>, system: SystemFn) {
        self.systems.insert(name.into(), system);
    }

    pub fn tick(&mut self) {
        let mut tick_data = TickData {
            dt: 0.,
            state: &mut self.state,
        };

        for (system_name, system) in &mut self.systems {
            log::trace!("[{system_name}] system starting..");
            if let Err(e) = system(&mut tick_data) {
                log::error!("[{system_name}]: {e:?}");
                return;
            }
            log::trace!("[{system_name}] system complete");
        }
    }

    pub fn insert_state<S: 'static>(&mut self, state: S) {
        self.state.insert_state(state);
    }
}

#[derive(Debug, Default)]
struct StateManager {
    inner: StateMap,
}

impl StateManager {
    pub fn get_state<S: 'static>(&mut self) -> Result<&mut S, EngineError> {
        // heheeheh
        self.inner
            .get_mut(&TypeId::of::<S>())
            .map(|s| s.downcast_mut())
            .flatten()
            .ok_or_else(|| EngineError::UninitialisedState(any::type_name::<S>()))
    }

    pub fn insert_state<S: 'static>(&mut self, state: S) {
        let type_id = TypeId::of::<S>();
        self.inner.insert(type_id, Box::new(state));
    }
}
