mod gui;
mod yakui_renderer;
use crate::{gui::draw_gui, yakui_renderer::YakuiRenderer};
use component_registry::ComponentRegistry;
use engine::Engine;
use engine_types::PrefabInstance;
use engine_types::{
    InstanceID, InstanceNode, NodeID, Prefab, PrefabDefinition, Scene, components::Transform,
};
use hecs::Entity;
use lazy_vulkan::{LazyVulkan, StateFamily};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};
use system_loader::GameplayLib;
use winit::window::WindowAttributes;

static GAMEPLAY_LIB_PATH: &'static str = "target/debug";
static GAMEPLAY_LIB_NAME: &str = "demo_platformer";

pub struct RenderStateFamily;
impl StateFamily for RenderStateFamily {
    type For<'s> = RenderState<'s>;
}

pub struct RenderState<'s> {
    pub yak: &'s yakui::Yakui,
    pub world: &'s hecs::World,
    pub herps: usize,
}

struct AppState {
    window: winit::window::Window,
    lazy_vulkan: LazyVulkan<RenderStateFamily>,
    yakui_winit: yakui_winit::YakuiWinit,
    loaded_prefabs: HashMap<String, Prefab>,
    #[allow(unused)]
    component_registry: ComponentRegistry,
    scene: Scene,
    node_entity_map: HashMap<NodeID, Entity>,
    engine: Engine,
    #[allow(unused)]
    gameplay: GameplayLib,
    yak: yakui::Yakui,
    editor_texture: yakui::TextureId,
}

#[derive(Default)]
struct App {
    state: Option<AppState>,
    project_path: PathBuf,
}

impl App {
    fn new(project_path: String) -> Self {
        Self {
            state: None,
            project_path: PathBuf::from(project_path),
        }
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_maximized(true)
                    .with_title("Bonk"),
            )
            .unwrap();

        let mut lazy_vulkan: LazyVulkan<RenderStateFamily> =
            lazy_vulkan::LazyVulkan::from_window(&window);

        let format = lazy_vulkan.renderer.get_drawable_format();
        let mut editor_extent = lazy_vulkan.renderer.get_drawable_extent();
        editor_extent.width /= 2;

        let mut engine = Engine::new_headless(
            &self.project_path,
            lazy_vulkan.core.clone(),
            lazy_vulkan.context.clone(),
            editor_extent,
            format,
        );

        let mut yak = yakui::Yakui::new();

        let mut yakui_renderer = YakuiRenderer::new(lazy_vulkan.context.clone(), format, &mut yak);

        let editor_texture = yakui_renderer.create_engine_image(engine.get_headless_image());

        lazy_vulkan.add_sub_renderer(Box::new(yakui_renderer));

        let yakui_winit = yakui_winit::YakuiWinit::new(&window);
        let mut component_registry = get_component_registry();
        let mut loaded_prefabs =
            load_prefabs(self.project_path.join("prefabs"), &mut component_registry);

        let (scene, node_entity_map) = load_scene(
            &self.project_path.join("scenes").join("default.json"),
            &mut loaded_prefabs,
            &component_registry,
            engine.world_mut(),
        );

        let gameplay_code = unsafe {
            system_loader::GameplayLib::load(GAMEPLAY_LIB_PATH, GAMEPLAY_LIB_NAME, &mut engine)
        }
        .unwrap();

        self.state = Some(AppState {
            window,
            lazy_vulkan,
            yakui_winit,
            loaded_prefabs,
            component_registry,
            scene,
            node_entity_map,
            engine,
            gameplay: gameplay_code,
            yak,
            editor_texture,
        })
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent;
        let state = self.state.as_mut().unwrap();

        // The logic here is a little weird.
        // First, resized events are special: both we and yakui need to handle them
        if let WindowEvent::Resized(size) = event {
            state
                .yakui_winit
                .handle_window_event(&mut state.yak, &event);

            state.lazy_vulkan.resize(size.width, size.height);
            return;
        };

        // Next, we hand the event to yakui-winit to see if it wants it
        if state
            .yakui_winit
            .handle_window_event(&mut state.yak, &event)
        {
            return;
        }

        // Finally, we see if it's something else we care about
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let swapchain = state.lazy_vulkan.get_drawable();
                state.lazy_vulkan.begin_commands();
                state.engine.tick_headless();

                let herps = *state.engine.get_state::<usize>().unwrap();
                let scene_path = self.project_path.join("scenes").join("default.json");
                draw_gui(state, &scene_path, herps);
                state.lazy_vulkan.draw_to_drawable(
                    &RenderState {
                        yak: &state.yak,
                        world: state.engine.world_mut(),
                        herps,
                    },
                    &swapchain,
                );
                state.lazy_vulkan.submit_and_present(swapchain);
            }
            _ => {}
        }

        unsafe { state.gameplay.check_reload(&mut state.engine).unwrap() };
    }

    fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        let state = self.state.as_mut().unwrap();
        state.window.request_redraw();
    }
}

fn load_scene(
    path: &PathBuf,
    loaded_prefabs: &mut HashMap<String, Prefab>,
    component_registry: &ComponentRegistry,
    world: &mut hecs::World,
) -> (Scene, HashMap<NodeID, Entity>) {
    let mut node_entity_map = HashMap::new();

    if !path.exists() {
        log::info!("Trying to read scene from {path:?}");
        std::fs::write(
            path,
            serde_json::to_string_pretty(&Scene::default()).unwrap(),
        )
        .unwrap();

        return (Scene::default(), node_entity_map);
    }

    let scene: Scene =
        serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(path).unwrap()))
            .unwrap();

    // Walk through each instance and spawn entities for each node
    for instance in &scene.instances {
        let prefab_name = &instance.prefab;
        let prefab = loaded_prefabs.get_mut(prefab_name).unwrap();

        for (node_index, instance_node) in &instance.nodes {
            let node = prefab.nodes.get_mut(*node_index).unwrap();
            let entity = spawn_entity_for_node(world, node);
            node_entity_map.insert(instance_node.node_id, entity);

            let mut entity_builder = hecs::EntityBuilderClone::new();
            for (component_name, component) in &instance_node.overrides {
                component_registry.add_component_to_builder(
                    component_name,
                    component.clone(),
                    &mut entity_builder,
                );
            }

            world.insert(entity, &entity_builder.build()).unwrap();

            // IMPORTANT: reset our IDs
            NEXT_NODE_ID.fetch_max(instance_node.node_id.as_raw(), Ordering::Relaxed);
        }

        // IMPORTANT: reset our IDs
        NEXT_INSTANCE_ID.fetch_max(instance.instance_id.as_raw(), Ordering::Relaxed);
    }

    NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed);
    NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);

    (scene, node_entity_map)
}

fn get_component_registry() -> ComponentRegistry {
    let registry = ComponentRegistry::default();
    // TODO: Register user components
    registry
}

fn spawn_prefab(
    name: &str,
    prefab: &mut Prefab,
    scene: &mut Scene,
    world: &mut hecs::World,
    node_entity_map: &mut HashMap<NodeID, Entity>,
) {
    let mut nodes = HashMap::new();
    for node in &mut prefab.nodes {
        let node_id = next_node_id();
        let entity = spawn_entity_for_node(world, node);
        node_entity_map.insert(node_id, entity);

        nodes.insert(
            node.index,
            InstanceNode {
                node_index: node.index,
                node_id,
                overrides: Default::default(),
            },
        );
    }

    let instance_id = NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed);

    scene.instances.push(PrefabInstance {
        instance_id: InstanceID::new(instance_id),
        prefab: name.to_string(),
        nodes,
    })
}

pub fn spawn_entity_for_node(
    world: &mut hecs::World,
    node: &mut engine_types::PrefabNode,
) -> Entity {
    let entity = world.spawn(&node.builder);
    world.insert_one(entity, Transform::default()).unwrap();
    entity
}

fn next_node_id() -> NodeID {
    NodeID::new(NEXT_NODE_ID.fetch_add(1, Ordering::Relaxed))
}

static NEXT_INSTANCE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));
static NEXT_NODE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));

fn load_prefabs(
    prefabs_path: PathBuf,
    component_registry: &mut ComponentRegistry,
) -> HashMap<String, Prefab> {
    log::info!("Loading prefabs from path: {:?}", prefabs_path);
    let mut prefabs = HashMap::new();

    for entry in std::fs::read_dir(prefabs_path).unwrap() {
        let entry = entry.unwrap();

        // BLEGH
        if !entry
            .path()
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .contains("json")
        {
            continue;
        }

        // Blegh!
        let file_name = entry
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let reader = std::fs::File::open(entry.path()).unwrap();
        let definition: PrefabDefinition = serde_json::from_reader(reader).unwrap();
        let prefab = prefab_compiler::compile(&definition, component_registry);

        prefabs.insert(file_name, prefab);
    }

    log::info!("Successfully loaded {} prefabs!", prefabs.len());

    prefabs
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the project
    #[arg(short, long)]
    project_path: String,
}

fn main() {
    env_logger::init();
    log::info!("::BONK SYSTEMS ONLINE::");
    log::info!("::READY TO BONK::");
    use clap::Parser;
    let args = Args::parse();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(args.project_path);

    event_loop.run_app(&mut app).unwrap()
}
