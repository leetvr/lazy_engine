mod gui;
mod yakui_renderer;

use std::collections::HashMap;

use component_registry::ComponentRegistry;
use engine_types::{Prefab, PrefabDefinition, Scene};
use lazy_vulkan::{LazyVulkan, SubRenderer};
use winit::window::WindowAttributes;

use crate::{gui::draw_gui, yakui_renderer::YakuiRenderer};

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
}

#[derive(Default)]
struct App {
    state: Option<AppState>,
    project_path: String,
}

impl App {
    fn new(project_path: String) -> Self {
        Self {
            state: None,
            project_path,
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

        let lazy_vulkan = lazy_vulkan::LazyVulkan::from_window(&window);
        let mut yak = yakui::Yakui::new();

        let sub_renderers = vec![YakuiRenderer::new(
            lazy_vulkan.context.clone(),
            lazy_vulkan.renderer.get_drawable_format(),
            &mut yak,
        )];

        let yakui_winit = yakui_winit::YakuiWinit::new(&window);
        let mut component_registry = get_component_registry();
        let loaded_prefabs = load_prefabs(&self.project_path, &mut component_registry);

        self.state = Some(AppState {
            window,
            lazy_vulkan,
            sub_renderers,
            yakui_winit,
            render_state: RenderState {
                yak,
                world: Default::default(),
            },
            loaded_prefabs,
            component_registry,
            scene: Default::default(),
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
                draw_gui(state);
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

fn get_component_registry() -> ComponentRegistry {
    let mut registry = ComponentRegistry::default();
    use engine_types::components::*;
    registry.register_component::<GLTFAsset>();
    registry.register_component::<Transform>();
    registry
}

fn load_prefabs(
    project_path: &str,
    component_registry: &mut ComponentRegistry,
) -> HashMap<String, Prefab> {
    let prefabs_path = std::path::Path::new(project_path).join("prefabs");
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
    /// Name of the person to greet
    #[arg(short, long, default_value = "./")]
    project_path: String,
}

fn main() {
    use clap::Parser;

    let args = Args::parse();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(args.project_path);

    event_loop.run_app(&mut app).unwrap()
}
