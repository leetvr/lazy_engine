use std::sync::Arc;

use lazy_vulkan::{SubRenderer, ash::vk};

use crate::RenderState;

pub struct YakuiRenderer {
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

impl SubRenderer for YakuiRenderer {
    type State = RenderState;

    fn stage_transfers(&mut self, render_state: &Self::State, _: &mut lazy_vulkan::Allocator) {
        let vulkan_context = &ctx(&self.context);

        // You *MUST* have called `yak.paint() this frame`
        let paint = render_state.yak.paint_dom();

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

fn ctx<'a>(context: &'a lazy_vulkan::Context) -> yakui_vulkan::VulkanContext<'a> {
    yakui_vulkan::VulkanContext {
        device: &context.device,
        queue: context.graphics_queue,
        memory_properties: context.memory_properties,
        properties: context.device_properties,
    }
}
