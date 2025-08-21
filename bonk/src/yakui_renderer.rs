use crate::RenderState;
use lazy_vulkan::{FULL_IMAGE, ImageManager, SubRenderer, ash::vk};
use std::sync::{Arc, Mutex, atomic::AtomicU64};
use yakui_vulkan::vk::Handle;

pub struct YakuiRenderer {
    yakui_vulkan: Arc<Mutex<yakui_vulkan::YakuiVulkan>>,
    // Because we do some more involved transfer operations, we need to stash a context reference
    context: Arc<lazy_vulkan::Context>,
    engine_image: Arc<AtomicU64>, // hilarious
}

impl YakuiRenderer {
    pub fn new<'a>(
        context: Arc<lazy_vulkan::Context>,
        yakui_vulkan: Arc<Mutex<yakui_vulkan::YakuiVulkan>>,
        engine_image: Arc<AtomicU64>,
    ) -> YakuiRenderer {
        Self {
            yakui_vulkan,
            context,
            engine_image,
        }
    }
}

impl<'a> SubRenderer<'a> for YakuiRenderer {
    type State = RenderState<'a>;

    fn stage_transfers(
        &mut self,
        render_state: &Self::State,
        _: &mut lazy_vulkan::Allocator,
        _: &mut ImageManager,
    ) {
        let vulkan_context = &ctx(&self.context);
        let mut yakui_vulkan = self.yakui_vulkan.lock().unwrap();

        // You *MUST* have called `yak.paint() this frame`
        let paint = render_state.yak.paint_dom();

        let context = &self.context;
        let command_buffer = context.draw_command_buffer;

        // Comedy
        let engine_image =
            vk::Image::from_raw(self.engine_image.load(std::sync::atomic::Ordering::Relaxed));

        // Transition the rendering attachments into their correct state
        unsafe {
            context.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&[
                    vk::ImageMemoryBarrier2::default()
                        .subresource_range(FULL_IMAGE)
                        .image(engine_image)
                        .src_access_mask(vk::AccessFlags2::NONE)
                        .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                        .dst_access_mask(vk::AccessFlags2::SHADER_READ)
                        .dst_stage_mask(vk::PipelineStageFlags2::FRAGMENT_SHADER)
                        .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL),
                ]),
            );
        }

        unsafe {
            yakui_vulkan.transfers_finished(vulkan_context);
            yakui_vulkan.transfer(paint, vulkan_context, self.context.draw_command_buffer);
        };
    }

    fn draw_layer(
        &mut self,
        render_state: &Self::State,
        context: &lazy_vulkan::Context,
        params: lazy_vulkan::DrawParams,
    ) {
        let vulkan_context = &ctx(&context);
        let mut yakui_vulkan = self.yakui_vulkan.lock().unwrap();

        // You *MUST* have called `yak.paint()` this frame
        let paint = render_state.yak.paint_dom();

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
                        .load_op(vk::AttachmentLoadOp::DONT_CARE)
                        .store_op(vk::AttachmentStoreOp::STORE)]),
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
            yakui_vulkan.paint(
                paint,
                vulkan_context,
                self.context.draw_command_buffer,
                params.drawable.extent,
            );
            context.cmd_end_rendering(command_buffer);
            yakui_vulkan.transfers_submitted();
        };
    }

    fn label(&self) -> &'static str {
        "YakuiRenderer"
    }
}

pub fn ctx<'a>(context: &'a lazy_vulkan::Context) -> yakui_vulkan::VulkanContext<'a> {
    yakui_vulkan::VulkanContext {
        device: &context.device,
        queue: context.graphics_queue,
        memory_properties: context.memory_properties,
        properties: context.device_properties,
    }
}
