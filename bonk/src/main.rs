use std::sync::Arc;

use lazy_vulkan::{LazyVulkan, SubRenderer, ash::vk};
use winit::window::WindowAttributes;

struct YakuiRenderer {
    yakui_vulkan: yakui_vulkan::YakuiVulkan,
    // Because we do some more involved transfer operations, we need to stash a context reference
    context: Arc<lazy_vulkan::Context>,
}

impl YakuiRenderer {
    pub fn new<'a>(
        context: Arc<lazy_vulkan::Context>,
        image_format: vk::Format,
        yak: &'a mut yakui::Yakui,
    ) -> Box<dyn SubRenderer<State = RenderState>> {
        // Get our yakui vulkan businesss together
        let vulkan_context = &ctx(&context);
        let mut yakui_vulkan = yakui_vulkan::YakuiVulkan::new(
            vulkan_context,
            yakui_vulkan::Options {
                dynamic_rendering_format: Some(image_format),
                render_pass: vk::RenderPass::null(),
                subpass: 0,
            },
        );

        yakui_vulkan.transfers_submitted();
        yakui_vulkan.set_paint_limits(vulkan_context, yak);

        Box::new(Self {
            yakui_vulkan,
            context,
        })
    }
}

#[derive(Debug, Clone, Default)]
struct RenderState {
    // This seems equal parts insane and reasonable, I'm not sure why
    paint_dom: *const yakui::paint::PaintDom,
}

impl SubRenderer for YakuiRenderer {
    type State = RenderState;

    fn stage_transfers(&mut self, render_state: &Self::State, _: &mut lazy_vulkan::Allocator) {
        let vulkan_context = &ctx(&self.context);

        // SAFETY: You *MUST* have called `yak.paint()` and set paint_dom accordingly
        let paint = unsafe { render_state.paint_dom.as_ref().unwrap() };

        unsafe {
            self.yakui_vulkan.transfers_finished(vulkan_context);
            self.yakui_vulkan
                .transfer(paint, vulkan_context, self.context.draw_command_buffer);
        };
    }

    fn draw_layer(
        &mut self,
        render_state: &Self::State,
        context: &lazy_vulkan::Context,
        params: lazy_vulkan::DrawParams,
    ) {
        let vulkan_context = &ctx(&context);

        // SAFETY: You *MUST* have called `yak.paint()` and set paint_dom accordingly
        let paint = unsafe { render_state.paint_dom.as_ref().unwrap() };

        let device = &context.device;
        let command_buffer = context.draw_command_buffer;

        unsafe {
            let render_area = params.drawable.extent;
            context.cmd_begin_rendering(
                command_buffer,
                &vk::RenderingInfo::default()
                    .render_area(render_area.into())
                    .layer_count(1)
                    .color_attachments(&[vk::RenderingAttachmentInfo::default()
                        .image_view(params.drawable.view)
                        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .load_op(vk::AttachmentLoadOp::CLEAR)
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .clear_value(vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.1, 0.1, 0.1, 1.0],
                            },
                        })]),
            );

            // Set the dynamic state
            device.cmd_set_scissor(command_buffer, 0, &[render_area.into()]);
            device.cmd_set_viewport(
                command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(render_area.width as _)
                    .height(render_area.height as _)
                    .max_depth(1.)],
            );

            // Paint the GUI
            self.yakui_vulkan.paint(
                paint,
                vulkan_context,
                self.context.draw_command_buffer,
                params.drawable.extent,
            );
            context.cmd_end_rendering(command_buffer);
            self.yakui_vulkan.transfers_submitted();
        };
    }

    fn label(&self) -> &'static str {
        "YakuiRenderer"
    }
}

struct AppState {
    window: winit::window::Window,
    lazy_vulkan: LazyVulkan,
    sub_renderers: Vec<Box<dyn SubRenderer<State = RenderState>>>,
    yak: yakui::Yakui,
    yakui_winit: yakui_winit::YakuiWinit,
}

#[derive(Default)]
struct App {
    state: Option<AppState>,
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

        self.state = Some(AppState {
            window,
            lazy_vulkan,
            sub_renderers,
            yak,
            yakui_winit,
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

        if state
            .yakui_winit
            .handle_window_event(&mut state.yak, &event)
        {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let paint_dom = draw_gui(&mut state.yak);
                let render_state = RenderState { paint_dom };
                state
                    .lazy_vulkan
                    .draw(&render_state, &mut state.sub_renderers);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
        let state = self.state.as_mut().unwrap();
        state.window.request_redraw();
    }
}

fn ctx<'a>(context: &'a lazy_vulkan::Context) -> yakui_vulkan::VulkanContext<'a> {
    yakui_vulkan::VulkanContext {
        device: &context.device,
        queue: context.graphics_queue,
        memory_properties: context.memory_properties,
        properties: context.device_properties,
    }
}

pub fn draw_gui(yak: &mut yakui::Yakui) -> &yakui::paint::PaintDom {
    yak.start();

    use yakui::{Color, button, column, label, row, use_state, widgets::Text};
    let clicked = use_state(|| false);
    column(|| {
        row(|| {
            label("Hello, world!");

            let mut text = Text::new(48.0, "colored text!");
            text.style.color = Color::RED;
            text.show();

            if button("click me!").clicked {
                clicked.set(!clicked.get());
            }

            if clicked.get() {
                label("I got clicked!");
            }
        });
    });

    yak.finish();
    yak.paint()
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::default();

    event_loop.run_app(&mut app).unwrap()
}
