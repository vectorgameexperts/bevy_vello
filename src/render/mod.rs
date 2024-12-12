//! Components and logic for rendering.

use bevy::{
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
            VertexBufferLayout, VertexFormat, VertexStepMode,
        },
        renderer::RenderDevice,
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dKey},
};
use std::sync::{Arc, Mutex};
use vello::{AaConfig, AaSupport};

mod plugin;
mod systems;

pub(crate) mod extract;
pub(crate) mod prepare;

pub(crate) use plugin::VelloRenderPlugin;

/// A handle to the screen space render target shader.
pub const SSRT_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(2314894693238056781);

#[derive(Component, Default, Debug, Clone, Deref, DerefMut, PartialEq, Eq)]
pub struct VelloCanvasMaterialHandle(Handle<VelloCanvasMaterial>);

/// A canvas material, with a shader that samples a texture with view-independent UV coordinates.
#[derive(AsBindGroup, TypePath, Asset, Clone)]
pub struct VelloCanvasMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for VelloCanvasMaterial {
    fn vertex_shader() -> ShaderRef {
        SSRT_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        SSRT_SHADER_HANDLE.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let formats = vec![
            // Position
            VertexFormat::Float32x3,
            VertexFormat::Float32x2,
        ];

        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct VelloRenderer(Arc<Mutex<vello::Renderer>>);

impl VelloRenderer {
    pub fn try_new(
        device: &vello::wgpu::Device,
        settings: &VelloRenderSettings,
    ) -> Result<Self, vello::Error> {
        vello::Renderer::new(
            device,
            vello::RendererOptions {
                surface_format: None,
                use_cpu: settings.use_cpu,
                // TODO: Vello doesn't currently allow adding additional AA support after initialization, so we need to use all support modes here instead.
                antialiasing_support: AaSupport::all(),
                num_init_threads: None,
            },
        )
        .map(Mutex::new)
        .map(Arc::new)
        .map(VelloRenderer)
    }
}

impl FromWorld for VelloRenderer {
    fn from_world(world: &mut World) -> Self {
        match VelloRenderer::try_new(
            world.get_resource::<RenderDevice>().unwrap().wgpu_device(),
            world.get_resource::<VelloRenderSettings>().unwrap(),
        ) {
            Ok(r) => r,
            Err(e) => {
                error!("Attempting safe-mode fallback, failed to initialize renderer: {e:}");
                {
                    let mut settings = world.get_resource_mut::<VelloRenderSettings>().unwrap();
                    settings.use_cpu = true;
                    settings.antialiasing = AaConfig::Area;
                }
                match VelloRenderer::try_new(
                    world.get_resource::<RenderDevice>().unwrap().wgpu_device(),
                    world.get_resource::<VelloRenderSettings>().unwrap(),
                ) {
                    Ok(r) => r,
                    Err(e) => panic!("Failed to start vello: {e}"),
                }
            }
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
#[cfg(feature = "lottie")]
pub struct VelatoRenderer(velato::Renderer);

/// Render settings for Vello.
#[derive(Resource, Clone)]
pub struct VelloRenderSettings {
    /// Use CPU instead of GPU
    pub use_cpu: bool,

    /// Which antialiasing strategy to use
    pub antialiasing: AaConfig,
}

impl Default for VelloRenderSettings {
    fn default() -> Self {
        Self {
            use_cpu: false,
            antialiasing: AaConfig::Area,
        }
    }
}

/// Canvas settings for Vello.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub(crate) struct VelloCanvasSettings {
    /// The render layers that will be used for the Vello canvas mesh.
    pub render_layers: RenderLayers,
}

/// Add this to any renderable vello asset to skip encoding that renderable.
#[derive(Component, Debug, Clone, Copy)]
pub struct SkipEncoding;
