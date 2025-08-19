use crate::{RenderState, RenderStateFamily};
use engine_types::components::{GLTFAsset, Transform};
use glam::Quat;
use lazy_vulkan::{BufferAllocation, ImageManager, LazyVulkan, Pipeline, SubRenderer, ash::vk};
use lazy_vulkan_gltf::LoadedAsset;
use std::{collections::HashMap, f32::consts::TAU, path::PathBuf};

static VERTEX_SHADER_PATH: &'static str = "bonk/shaders/main.vert.spv";
static FRAGMENT_SHADER_PATH: &'static str = "bonk/shaders/main.frag.spv";

pub struct SceneRenderer {
    pipeline: Pipeline,
    index_buffer: BufferAllocation<u32>,
    assets: HashMap<String, LoadedAsset>,
    asset_path: PathBuf,
}

impl SceneRenderer {
    pub fn new(
        lazy_vulkan: &mut LazyVulkan<RenderStateFamily>,
        asset_path: PathBuf,
    ) -> SceneRenderer {
        let pipeline = lazy_vulkan
            .renderer
            .create_pipeline::<Registers>(VERTEX_SHADER_PATH, FRAGMENT_SHADER_PATH);
        let index_buffer = lazy_vulkan
            .renderer
            .allocator
            .allocate_buffer(1024 * 1000, vk::BufferUsageFlags::INDEX_BUFFER);

        SceneRenderer {
            pipeline,
            index_buffer,
            assets: Default::default(),
            asset_path,
        }
    }
}

impl<'a> SubRenderer<'a> for SceneRenderer {
    type State = RenderState<'a>;

    fn stage_transfers(
        &mut self,
        state: &Self::State,
        allocator: &mut lazy_vulkan::Allocator,
        image_manager: &mut ImageManager,
    ) {
        for (_, asset) in state.world.query::<&GLTFAsset>().iter() {
            let key = &asset.path;
            if self.assets.contains_key(key) {
                continue;
            }

            let path = self.asset_path.join(&asset.path);
            log::debug!("Asset {key} has not been loaded, trying to load at {path:?}..");
            let loaded = lazy_vulkan_gltf::load_asset(
                path,
                allocator,
                image_manager,
                &mut self.index_buffer,
            )
            .unwrap();

            self.assets.insert(key.clone(), loaded);
            log::debug!("Done! Asset [{key}] loaded");
        }
    }

    fn draw_opaque(
        &mut self,
        state: &Self::State,
        context: &lazy_vulkan::Context,
        params: lazy_vulkan::DrawParams,
    ) {
        self.begin_rendering(context, &self.pipeline);

        let device = &context.device;
        let command_buffer = context.draw_command_buffer;
        let world = &state.world;

        let drawable_extent = params.drawable.extent;

        // We're only drawing on the *right* hand side of the screen.
        let extent = vk::Extent2D {
            width: drawable_extent.width / 2,
            height: drawable_extent.height,
        };

        let mvp = build_mvp(extent);
        unsafe {
            device.cmd_set_scissor(
                command_buffer,
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D {
                        x: extent.width as _,
                        ..Default::default()
                    },
                    extent,
                }],
            );
            device.cmd_set_viewport(
                command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(extent.width as _)
                    .height(extent.height as _)
                    .x(extent.width as _)
                    .max_depth(1.)],
            );
        };

        for (_, (asset, transform)) in world.query::<(&GLTFAsset, &Transform)>().iter() {
            let Some(asset) = self.assets.get(&asset.path) else {
                log::debug!("Asset {:?} does not exist yet", &asset.path);
                continue;
            };

            for mesh in &asset.meshes {
                for primitive in &mesh.primitives {
                    let registers = Registers {
                        mvp: mvp * transform,
                        vertex_buffer: primitive.vertex_buffer.device_address,
                        material_buffer: primitive.material,
                    };
                    self.pipeline.update_registers(&registers);

                    unsafe {
                        device.cmd_bind_index_buffer(
                            command_buffer,
                            self.index_buffer.handle,
                            primitive.index_buffer_offset,
                            vk::IndexType::UINT32,
                        );
                        device.cmd_draw_indexed(command_buffer, primitive.index_count, 1, 0, 0, 0);
                    };
                }
            }
        }
    }

    fn label(&self) -> &'static str {
        "Mesh Renderer"
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Registers {
    mvp: glam::Mat4,
    vertex_buffer: vk::DeviceAddress,
    material_buffer: vk::DeviceAddress,
}

unsafe impl bytemuck::Zeroable for Registers {}
unsafe impl bytemuck::Pod for Registers {}

fn build_mvp(extent: vk::Extent2D) -> glam::Mat4 {
    // Build up the perspective matrix
    let aspect_ratio = extent.width as f32 / extent.height as f32;
    let mut perspective =
        glam::Mat4::perspective_infinite_reverse_rh(60_f32.to_radians(), aspect_ratio, 0.01);

    // WULKAN
    perspective.y_axis *= -1.0;

    // Get view_from_world
    // TODO: camera
    let world_from_view = glam::Affine3A::from_rotation_translation(
        Quat::from_euler(glam::EulerRot::YXZ, TAU * 0.1, -TAU * 0.1, 0.),
        glam::Vec3::new(4., 4., 4.),
    );
    let view_from_world = world_from_view.inverse();

    perspective * view_from_world
}

#[cfg(target_vendor = "apple")]
static VULKAN_VERSION: &'static str = "vulkan1.2";
#[cfg(not(target_vendor = "apple"))]
static VULKAN_VERSION: &'static str = "vulkan1.3";

pub fn compile_shaders() {
    let _ = std::process::Command::new("glslc")
        .arg("bonk/shaders/main.vert")
        .arg(format!("--target-env={VULKAN_VERSION}"))
        .arg("-g")
        .arg("-o")
        .arg(VERTEX_SHADER_PATH)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let _ = std::process::Command::new("glslc")
        .arg("bonk/shaders/main.frag")
        .arg(format!("--target-env={VULKAN_VERSION}"))
        .arg("-g")
        .arg("-o")
        .arg(FRAGMENT_SHADER_PATH)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}
