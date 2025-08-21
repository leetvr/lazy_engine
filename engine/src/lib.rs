use hecs::CommandBuffer;
use lazy_vulkan::{IntoExtent, LazyVulkan, StateFamily, ash::vk};
use std::{
    any::{self, Any, TypeId},
    collections::HashMap,
    ffi::CStr,
    path::PathBuf,
    sync::Arc,
};

pub use engine_types::components;

use crate::sub_renderers::SceneRenderer;
mod sub_renderers;

type StateMap = HashMap<TypeId, Box<dyn Any>>;

pub struct Engine {
    systems: HashMap<String, SystemFn>,
    state: StateManager,
    world: hecs::World,
    lazy_vulkan: LazyVulkan<TickDataFamily>,
    #[allow(unused)]
    project_path: PathBuf,
}

pub const VERSION: &str = git_version::git_version!();
pub type SystemFn = fn(&mut TickData) -> anyhow::Result<()>;

pub struct TickData<'a> {
    pub dt: f32,
    pub command_buffer: CommandBuffer,
    pub world: &'a hecs::World,
    state: &'a mut StateManager,
}

struct TickDataFamily;
impl StateFamily for TickDataFamily {
    type For<'s> = TickData<'s>;
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
    pub fn new_headless(
        project_path: impl Into<PathBuf>,
        core: Arc<lazy_vulkan::Core>,
        context: Arc<lazy_vulkan::Context>,
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> Engine {
        let mut lazy_vulkan = LazyVulkan::headless(core, context, extent, format);
        let project_path = project_path.into();
        let scene_renderer = SceneRenderer::new(&mut lazy_vulkan, project_path.join("assets"));
        lazy_vulkan.add_sub_renderer(Box::new(scene_renderer));

        Engine {
            systems: Default::default(),
            state: Default::default(),
            world: Default::default(),
            lazy_vulkan,
            project_path,
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

    pub fn tick_headless(&mut self) {
        let command_buffer = CommandBuffer::new();
        let mut tick_data = TickData {
            dt: 0.,
            state: &mut self.state,
            world: &self.world,
            command_buffer,
        };

        for (system_name, system) in &mut self.systems {
            log::trace!("[{system_name}] system starting..");
            if let Err(e) = system(&mut tick_data) {
                log::error!("[{system_name}]: {e:?}");
                return;
            }
            log::trace!("[{system_name}] system complete");
        }

        let drawable = self.lazy_vulkan.get_drawable();
        self.lazy_vulkan.draw_to_drawable(&tick_data, &drawable);

        tick_data.command_buffer.run_on(&mut self.world);
    }

    pub fn tick(&mut self) {
        let command_buffer = CommandBuffer::new();
        let mut tick_data = TickData {
            dt: 0.,
            state: &mut self.state,
            world: &self.world,
            command_buffer,
        };

        for (system_name, system) in &mut self.systems {
            log::trace!("[{system_name}] system starting..");
            if let Err(e) = system(&mut tick_data) {
                log::error!("[{system_name}]: {e:?}");
                return;
            }
            log::trace!("[{system_name}] system complete");
        }

        self.lazy_vulkan.draw(&tick_data);

        tick_data.command_buffer.run_on(&mut self.world);
    }

    pub fn insert_state<S: 'static>(&mut self, state: S) {
        self.state.insert_state(state);
    }

    pub fn get_state<S: 'static>(&mut self) -> Result<&mut S, EngineError> {
        self.state.get_state()
    }

    pub fn world_mut(&mut self) -> &mut hecs::World {
        &mut self.world
    }

    pub fn get_headless_image(&self) -> lazy_vulkan::HeadlessSwapchainImage {
        self.lazy_vulkan
            .renderer
            .get_headless_image()
            .expect("You're not in headless mode, idiot")
    }

    pub fn resize(&mut self, new_extent: impl IntoExtent) {
        self.lazy_vulkan.resize(new_extent);
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
