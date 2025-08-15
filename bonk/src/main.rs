mod gui;
mod scene_renderer;
mod yakui_renderer;
use crate::{gui::draw_gui, scene_renderer::SceneRenderer, yakui_renderer::YakuiRenderer};
use component_registry::ComponentRegistry;
use engine_types::PrefabInstance;
use engine_types::{
    InstanceID, InstanceNode, NodeID, Prefab, PrefabDefinition, Scene, components::Transform,
};
use hecs::Entity;
use lazy_vulkan::{LazyVulkan, SubRenderer};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        LazyLock,
        atomic::{AtomicUsize, Ordering},
    },
};
use winit::window::WindowAttributes;

pub struct RenderState {
    pub yak: yakui::Yakui,
    pub world: hecs::World,
}

struct AppState {
    window: winit::window::Window,
    lazy_vulkan: LazyVulkan,
    sub_renderers: Vec<Box<dyn SubRenderer<State = RenderState>>>,
    yakui_winit: yakui_winit::YakuiWinit,
    render_state: RenderState,
    loaded_prefabs: HashMap<String, Prefab>,
    #[allow(unused)]
    component_registry: ComponentRegistry,
    scene: Scene,
    node_entity_map: HashMap<NodeID, Entity>,
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

        let mut lazy_vulkan = lazy_vulkan::LazyVulkan::from_window(&window);
        let mut yak = yakui::Yakui::new();

        let asset_path = self.project_path.join("assets");

        let sub_renderers = vec![
            SceneRenderer::new(&mut lazy_vulkan, asset_path),
            YakuiRenderer::new(
                lazy_vulkan.context.clone(),
                lazy_vulkan.renderer.get_drawable_format(),
                &mut yak,
            ),
        ];

        let yakui_winit = yakui_winit::YakuiWinit::new(&window);
        let mut component_registry = get_component_registry();
        let mut loaded_prefabs =
            load_prefabs(self.project_path.join("prefabs"), &mut component_registry);

        let (scene, world, node_entity_map) = load_scene(
            &self.project_path.join("scenes").join("default.json"),
            &mut loaded_prefabs,
            &component_registry,
        );

        self.state = Some(AppState {
            window,
            lazy_vulkan,
            sub_renderers,
            yakui_winit,
            render_state: RenderState { yak, world },
            loaded_prefabs,
            component_registry,
            scene,
            node_entity_map,
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
                .handle_window_event(&mut state.render_state.yak, &event);

            state.lazy_vulkan.resize(size.width, size.height);
            return;
        };

        // Next, we hand the event to yakui-winit to see if it wants it
        if state
            .yakui_winit
            .handle_window_event(&mut state.render_state.yak, &event)
        {
            return;
        }

        // Finally, we see if it's something else we care about
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let scene_path = self.project_path.join("scenes").join("default.json");
                draw_gui(state, &scene_path);
                state
                    .lazy_vulkan
                    .draw(&state.render_state, &mut state.sub_renderers);
            }
            _ => {}
        }
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
) -> (Scene, hecs::World, HashMap<NodeID, Entity>) {
    let mut world = hecs::World::new();
    let mut node_entity_map = HashMap::new();

    if !path.exists() {
        println!("trying to read {path:?}");
        std::fs::write(
            path,
            serde_json::to_string_pretty(&Scene::default()).unwrap(),
        )
        .unwrap();

        return (Scene::default(), world, node_entity_map);
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
            let entity = spawn_entity_for_node(&mut world, node);
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

    println!("next_node_id: {}", NEXT_NODE_ID.load(Ordering::Relaxed));
    println!(
        "next_instance_id_id: {}",
        NEXT_INSTANCE_ID.load(Ordering::Relaxed)
    );

    (scene, world, node_entity_map)
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
    println!("Prefabs path: {:?}", prefabs_path);
    let mut prefabs = HashMap::new();

    for entry in std::fs::read_dir(prefabs_path).unwrap() {
        let entry = entry.unwrap();
        println!("Prefab entry: {:?}", entry);

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

        println!("yes?");

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

    println!("loaded {} prefabs", prefabs.len());

    prefabs
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the project
    #[arg(short, long, default_value = "./")]
    project_path: String,
}

fn main() {
    use clap::Parser;

    let args = Args::parse();
    scene_renderer::compile_shaders();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(args.project_path);

    event_loop.run_app(&mut app).unwrap()
}
