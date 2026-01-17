//! Frustum Render
//!
//! GPU rendering backend for Frustum using wgpu.

pub mod audit;
pub mod font;
pub mod invariants;
pub mod metrics;
pub mod primitives;

use bytemuck::{Pod, Zeroable};
use frustum_core::Scene;
use glam::Mat4;
use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use thiserror::Error;
use wgpu::util::DeviceExt;

pub use audit::AuditBundle;
pub use invariants::{compare_for_regression, RegressionResult, RegressionTolerance};
pub use primitives::{ExpandedLabel, SimpleVertex, TextVertex};

/// Errors that can occur during rendering.
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to create GPU adapter")]
    AdapterCreation,
    #[error("Failed to create GPU device: {0}")]
    DeviceCreation(#[from] wgpu::RequestDeviceError),
    #[error("Failed to encode PNG: {0}")]
    PngEncoding(#[from] image::ImageError),
    #[error("Buffer mapping failed")]
    BufferMapping,
}

/// Render configuration.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Background color as RGBA (0.0 to 1.0).
    pub background: [f32; 4],
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            background: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Vertex with position, normal, and color.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Uniform buffer for view-projection matrix and lighting.
/// Aligned to WGSL rules: vec3 has 16-byte alignment.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],  // 64 bytes
    light_dir: [f32; 4],       // 16 bytes: xyz = direction, w = intensity
    light_config: [f32; 4],    // 16 bytes: x = enabled (0 or 1), yzw unused
}

/// Render metadata for debugging/reproducibility.
#[derive(Debug, Clone)]
pub struct RenderMetadata {
    pub backend: String,
    pub adapter_name: String,
}

/// Internal renderer state with mesh, point, line, and text pipelines.
struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    mesh_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    point_pipeline: primitives::PointPipeline,
    line_pipeline: primitives::LinePipeline,
    text_pipeline: primitives::TextPipeline,
    metadata: RenderMetadata,
}

impl Renderer {
    async fn new() -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::AdapterCreation)?;

        let adapter_info = adapter.get_info();
        let metadata = RenderMetadata {
            backend: format!("{:?}", adapter_info.backend),
            adapter_name: adapter_info.name.clone(),
        };

        log::info!(
            "Using adapter: {} (backend: {:?})",
            adapter_info.name,
            adapter_info.backend
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await?;

        // Load shader
        let shader_source = include_str!("shaders/basic.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic Shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_source)),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout and bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create point, line, and text pipelines
        let point_pipeline = primitives::PointPipeline::new(&device);
        let line_pipeline = primitives::LinePipeline::new(&device);
        let text_pipeline = primitives::TextPipeline::new(&device, &queue);

        Ok(Self {
            device,
            queue,
            mesh_pipeline: pipeline,
            uniform_buffer,
            uniform_bind_group,
            point_pipeline,
            line_pipeline,
            text_pipeline,
            metadata,
        })
    }

    fn render_vertices(
        &self,
        vertices: &[Vertex],
        indices: Option<&[u32]>,
        view_proj: Mat4,
        light: Option<&frustum_core::Light>,
        config: &RenderConfig,
    ) -> Result<Vec<u8>, RenderError> {
        let width = config.width;
        let height = config.height;

        // Update uniform buffer with lighting from scene
        let (light_dir, intensity, enabled) = if let Some(l) = light {
            (l.direction, l.intensity, if l.enabled { 1.0 } else { 0.0 })
        } else {
            // No light: disabled
            ([0.0, 0.0, 1.0], 0.0, 0.0)
        };
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            light_dir: [light_dir[0], light_dir[1], light_dir[2], intensity],
            light_config: [enabled, 0.0, 0.0, 0.0],
        };
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Create vertex buffer
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // Create index buffer if provided
        let index_buffer = indices.map(|idx| {
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(idx),
                    usage: wgpu::BufferUsages::INDEX,
                })
        });

        // Create output texture
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create readback buffer
        let bytes_per_row = (width * 4).next_multiple_of(256);
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Create command encoder and render
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: config.background[0] as f64,
                            g: config.background[1] as f64,
                            b: config.background[2] as f64,
                            a: config.background[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if !vertices.is_empty() {
                render_pass.set_pipeline(&self.mesh_pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

                if let Some(ref idx_buf) = index_buffer {
                    render_pass.set_index_buffer(idx_buf.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..indices.unwrap().len() as u32, 0, 0..1);
                } else {
                    render_pass.draw(0..vertices.len() as u32, 0..1);
                }
            }
        }

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Map buffer and read data
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .unwrap()
            .map_err(|_| RenderError::BufferMapping)?;

        let data = buffer_slice.get_mapped_range();

        // Remove padding from rows
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            let start = (y * bytes_per_row) as usize;
            let end = start + (width * 4) as usize;
            pixels.extend_from_slice(&data[start..end]);
        }

        drop(data);
        output_buffer.unmap();

        Ok(pixels)
    }

    /// Render meshes, points, lines, and text using all pipelines.
    fn render_scene(
        &self,
        mesh_vertices: &[Vertex],
        point_vertices: &[SimpleVertex],
        line_vertices: &[SimpleVertex],
        labels: &[primitives::ExpandedLabel],
        point_size: f32,
        view_proj: Mat4,
        camera_right: glam::Vec3,
        camera_up: glam::Vec3,
        light: Option<&frustum_core::Light>,
        config: &RenderConfig,
    ) -> Result<Vec<u8>, RenderError> {
        let width = config.width;
        let height = config.height;

        // Update mesh uniform buffer with lighting from scene
        let (light_dir, intensity, enabled) = if let Some(l) = light {
            (l.direction, l.intensity, if l.enabled { 1.0 } else { 0.0 })
        } else {
            // No light: disabled (flat colors)
            ([0.0, 0.0, 1.0], 0.0, 0.0)
        };
        let uniforms = Uniforms {
            view_proj: view_proj.to_cols_array_2d(),
            light_dir: [light_dir[0], light_dir[1], light_dir[2], intensity],
            light_config: [enabled, 0.0, 0.0, 0.0],
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Create mesh vertex buffer
        let mesh_vertex_buffer = if !mesh_vertices.is_empty() {
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Mesh Vertex Buffer"),
                contents: bytemuck::cast_slice(mesh_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }))
        } else {
            None
        };

        // Create textures
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bytes_per_row = (width * 4).next_multiple_of(256);
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: config.background[0] as f64,
                            g: config.background[1] as f64,
                            b: config.background[2] as f64,
                            a: config.background[3] as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render meshes
            if let Some(ref vb) = mesh_vertex_buffer {
                render_pass.set_pipeline(&self.mesh_pipeline);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vb.slice(..));
                render_pass.draw(0..mesh_vertices.len() as u32, 0..1);
            }

            // Render points (billboarded quads)
            self.point_pipeline.render(&mut render_pass, &self.queue, &self.device, point_vertices, view_proj, point_size, camera_right, camera_up);

            // Render lines
            self.line_pipeline.render(&mut render_pass, &self.queue, &self.device, line_vertices, view_proj);

            // Render text labels (billboarded textured quads)
            self.text_pipeline.render(&mut render_pass, &self.queue, &self.device, labels, view_proj, camera_right, camera_up);
        }

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &output_buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bytes_per_row), rows_per_image: Some(height) } },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| { tx.send(result).unwrap(); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().map_err(|_| RenderError::BufferMapping)?;

        let data = buffer_slice.get_mapped_range();
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..height {
            let start = (y * bytes_per_row) as usize;
            let end = start + (width * 4) as usize;
            pixels.extend_from_slice(&data[start..end]);
        }
        drop(data);
        output_buffer.unmap();

        Ok(pixels)
    }
}

/// Render a scene to a PNG image.
///
/// This is the primary entry point for headless rendering.
pub fn render_to_png(scene: &Scene, config: &RenderConfig) -> Result<Vec<u8>, RenderError> {
    pollster::block_on(render_to_png_async(scene, config))
}

async fn render_to_png_async(scene: &Scene, config: &RenderConfig) -> Result<Vec<u8>, RenderError> {
    let renderer = Renderer::new().await?;

    let aspect_ratio = config.width as f32 / config.height as f32;
    let view_proj = scene.camera.view_projection_matrix(aspect_ratio);

    // Compute camera basis vectors for billboarding
    let (camera_right, camera_up) = compute_camera_basis(&scene.camera);

    // Convert scene elements to vertices
    let (mesh_vertices, point_vertices, line_vertices, labels, point_size) = scene_to_vertices(scene);

    let pixels = renderer.render_scene(&mesh_vertices, &point_vertices, &line_vertices, &labels, point_size, view_proj, camera_right, camera_up, scene.light.as_ref(), config)?;

    // Encode to PNG
    encode_png(&pixels, config.width, config.height)
}

/// Render a scene and produce an audit bundle for sanity checking.
///
/// Returns both the PNG data and a structured audit bundle containing
/// metadata, geometry probes, image metrics, and invariant check results.
pub fn render_with_audit(
    scene: &Scene,
    config: &RenderConfig,
) -> Result<(Vec<u8>, AuditBundle), RenderError> {
    pollster::block_on(render_with_audit_async(scene, config))
}

async fn render_with_audit_async(
    scene: &Scene,
    config: &RenderConfig,
) -> Result<(Vec<u8>, AuditBundle), RenderError> {
    use crate::audit::*;
    use frustum_core::scene::SceneElement;

    let renderer = Renderer::new().await?;

    let aspect_ratio = config.width as f32 / config.height as f32;
    let view_proj = scene.camera.view_projection_matrix(aspect_ratio);

    // Compute camera basis vectors for billboarding
    let (camera_right, camera_up) = compute_camera_basis(&scene.camera);

    // Convert scene elements to vertices
    let (mesh_vertices, point_vertices, line_vertices, labels, point_size) = scene_to_vertices(scene);

    // Compute primitive counts
    let mut primitive_counts = PrimitiveCounts::default();
    for element in &scene.elements {
        match element {
            SceneElement::Mesh(mesh) => {
                primitive_counts.meshes += 1;
                primitive_counts.total_triangles += mesh.indices.len() as u32 / 3;
                primitive_counts.total_vertices += mesh.positions.len() as u32 / 3;
            }
            SceneElement::PointCloud(pc) => {
                primitive_counts.point_clouds += 1;
                primitive_counts.total_points += pc.positions.len() as u32 / 3;
            }
            SceneElement::Polyline(line) => {
                primitive_counts.polylines += 1;
                let vertex_count = line.positions.len() / 3;
                if vertex_count > 1 {
                    primitive_counts.total_line_segments += (vertex_count - 1) as u32;
                }
            }
            SceneElement::Axes(axes) => {
                // Axes expand into polylines
                let (polylines, _labels) = axes.expand();
                primitive_counts.polylines += polylines.len() as u32;
                for line in &polylines {
                    let vertex_count = line.positions.len() / 3;
                    if vertex_count > 1 {
                        primitive_counts.total_line_segments += (vertex_count - 1) as u32;
                    }
                }
            }
        }
    }

    // Compute scene hash
    let scene_json = scene.to_json().unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    scene_json.hash(&mut hasher);
    let scene_hash = format!("{:016x}", hasher.finish());

    // Build metadata
    let metadata = RenderMetadata {
        scene_hash,
        schema_version: "frustum/scene/v1".to_string(),
        renderer_version: env!("CARGO_PKG_VERSION").to_string(),
        backend: renderer.metadata.backend.clone(),
        adapter: renderer.metadata.adapter_name.clone(),
        resolution: [config.width, config.height],
        camera: CameraSummary {
            projection: match scene.camera.projection {
                frustum_core::Projection::Perspective => "perspective".to_string(),
                frustum_core::Projection::Orthographic => "orthographic".to_string(),
            },
            position: scene.camera.position,
            target: scene.camera.target,
            near: scene.camera.near,
            far: scene.camera.far,
            fov_or_height: scene.camera.fov_or_height,
        },
        world_bounds: BoundsSummary {
            min: scene.bounds.min,
            max: scene.bounds.max,
            center: [
                (scene.bounds.min[0] + scene.bounds.max[0]) / 2.0,
                (scene.bounds.min[1] + scene.bounds.max[1]) / 2.0,
                (scene.bounds.min[2] + scene.bounds.max[2]) / 2.0,
            ],
            extent: [
                scene.bounds.max[0] - scene.bounds.min[0],
                scene.bounds.max[1] - scene.bounds.min[1],
                scene.bounds.max[2] - scene.bounds.min[2],
            ],
        },
        primitive_counts,
    };

    // Render
    let pixels = renderer.render_scene(&mesh_vertices, &point_vertices, &line_vertices, &labels, point_size, view_proj, camera_right, camera_up, scene.light.as_ref(), config)?;

    // Compute geometry probes (simplified for now)
    let geometry = GeometryProbes {
        ndc_bounds: None, // TODO: compute from projected vertices
        depth_stats: DepthStats {
            min: 0.0,
            max: 1.0,
            mean: 0.5,
            far_plane_percentage: 0.0,
        },
        degenerate_count: 0,
        clipped_count: 0,
        backface_count: 0,
        geometry_visible: !mesh_vertices.is_empty() || !point_vertices.is_empty() || !line_vertices.is_empty(),
        has_invalid_values: false,
    };

    // Compute image metrics
    let image_metrics = metrics::compute_image_metrics(
        &pixels,
        config.width,
        config.height,
        config.background,
    );

    // Check invariants
    let invariants = invariants::check_all_invariants(scene, &metadata, &geometry, &image_metrics);

    // Build audit bundle
    let audit = AuditBundle {
        metadata,
        geometry,
        image_metrics,
        invariants,
    };

    // Encode to PNG
    let png_data = encode_png(&pixels, config.width, config.height)?;

    Ok((png_data, audit))
}

/// Render a hardcoded triangle for testing the pipeline.
pub fn render_test_triangle(config: &RenderConfig) -> Result<Vec<u8>, RenderError> {
    pollster::block_on(render_test_triangle_async(config))
}

async fn render_test_triangle_async(config: &RenderConfig) -> Result<Vec<u8>, RenderError> {
    let renderer = Renderer::new().await?;

    log::info!(
        "Render metadata: backend={}, adapter={}",
        renderer.metadata.backend,
        renderer.metadata.adapter_name
    );

    // Hardcoded triangle in clip space (no transformation needed)
    // Normal points toward viewer (+Z)
    let normal = [0.0, 0.0, 1.0];
    let vertices = vec![
        Vertex {
            position: [0.0, 0.5, 0.0],
            normal,
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.0],
            normal,
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            normal,
            color: [0.0, 0.0, 1.0],
        },
    ];

    // Identity matrix for clip-space vertices
    let view_proj = Mat4::IDENTITY;

    let pixels = renderer.render_vertices(&vertices, None, view_proj, None, config)?;

    encode_png(&pixels, config.width, config.height)
}

/// Compute camera basis vectors (right, up) for billboarding.
fn compute_camera_basis(camera: &frustum_core::Camera) -> (glam::Vec3, glam::Vec3) {
    let position = glam::Vec3::from_array(camera.position);
    let target = glam::Vec3::from_array(camera.target);
    let world_up = glam::Vec3::Y;

    let forward = (target - position).normalize();
    let right = forward.cross(world_up).normalize();
    let up = right.cross(forward);

    (right, up)
}

/// Get color for a scalar value using a material's colormap.
fn scalar_to_color(
    scalar: f32,
    material: &frustum_core::ScalarMappedMaterial,
) -> [f32; 3] {
    use frustum_core::Colormap;

    if !scalar.is_finite() {
        return [material.missing_color[0], material.missing_color[1], material.missing_color[2]];
    }

    let [min, max] = material.range;
    let t = if max > min {
        (scalar - min) / (max - min)
    } else {
        0.5
    };

    let t = if material.clamp { t.clamp(0.0, 1.0) } else { t };

    if let Some(cmap) = Colormap::from_name(&material.colormap) {
        cmap.sample(t)
    } else {
        // Unknown colormap, use grayscale
        let v = t.clamp(0.0, 1.0);
        [v, v, v]
    }
}

/// Get solid color from a material (RGB).
fn get_solid_color(material: &frustum_core::Material) -> [f32; 3] {
    match material {
        frustum_core::Material::Solid(m) => [m.color[0], m.color[1], m.color[2]],
        frustum_core::Material::ScalarMapped(m) => {
            // For scalar-mapped without scalars, use middle of range
            scalar_to_color((m.range[0] + m.range[1]) / 2.0, m)
        }
    }
}

/// Convert scene elements to separate vertex arrays for meshes, points, lines, and labels.
fn scene_to_vertices(scene: &Scene) -> (Vec<Vertex>, Vec<SimpleVertex>, Vec<SimpleVertex>, Vec<primitives::ExpandedLabel>, f32) {
    use frustum_core::scene::SceneElement;
    use glam::Vec3;

    let mut mesh_vertices = Vec::new();
    let mut point_vertices = Vec::new();
    let mut line_vertices = Vec::new();
    let mut labels = Vec::new();
    let mut max_point_size = 4.0f32;

    // Default colors for primitives without materials
    let default_mesh_color = [0.7, 0.7, 0.7];
    let default_point_color = [1.0, 0.5, 0.0]; // Orange
    let default_line_color = [0.2, 0.8, 0.2];  // Green
    let default_axis_color = [0.8, 0.8, 0.8];  // Light gray
    let default_label_color = [0.9, 0.9, 0.9]; // Light gray for labels

    for element in &scene.elements {
        match element {
            SceneElement::Mesh(mesh) => {
                // Get material color or use scalar mapping
                let material = mesh.material_id.as_ref().and_then(|id| scene.get_material(id));
                let has_scalars = mesh.scalars.is_some();
                let use_scalar_color = has_scalars && matches!(material, Some(frustum_core::Material::ScalarMapped(_)));

                for chunk in mesh.indices.chunks(3) {
                    let i0 = chunk[0] as usize;
                    let i1 = chunk[1] as usize;
                    let i2 = chunk[2] as usize;

                    let p0 = Vec3::new(mesh.positions[i0 * 3], mesh.positions[i0 * 3 + 1], mesh.positions[i0 * 3 + 2]);
                    let p1 = Vec3::new(mesh.positions[i1 * 3], mesh.positions[i1 * 3 + 1], mesh.positions[i1 * 3 + 2]);
                    let p2 = Vec3::new(mesh.positions[i2 * 3], mesh.positions[i2 * 3 + 1], mesh.positions[i2 * 3 + 2]);

                    let edge1 = p1 - p0;
                    let edge2 = p2 - p0;
                    let face_normal = edge1.cross(edge2).normalize_or_zero();
                    let use_mesh_normals = mesh.normals.is_some();

                    for &index in chunk {
                        let i = index as usize;
                        let position = [mesh.positions[i * 3], mesh.positions[i * 3 + 1], mesh.positions[i * 3 + 2]];
                        let normal = if use_mesh_normals {
                            let normals = mesh.normals.as_ref().unwrap();
                            [normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]]
                        } else {
                            face_normal.to_array()
                        };

                        let color = if use_scalar_color {
                            let scalars = mesh.scalars.as_ref().unwrap();
                            let scalar = scalars.get(i).copied().unwrap_or(0.0);
                            if let Some(frustum_core::Material::ScalarMapped(sm)) = material {
                                scalar_to_color(scalar, sm)
                            } else {
                                default_mesh_color
                            }
                        } else if let Some(mat) = material {
                            get_solid_color(mat)
                        } else {
                            default_mesh_color
                        };

                        mesh_vertices.push(Vertex { position, normal, color });
                    }
                }
            }
            SceneElement::PointCloud(pc) => {
                max_point_size = max_point_size.max(pc.point_size);
                let point_count = pc.positions.len() / 3;

                let material = pc.material_id.as_ref().and_then(|id| scene.get_material(id));
                let has_scalars = pc.scalars.is_some();
                let use_scalar_color = has_scalars && matches!(material, Some(frustum_core::Material::ScalarMapped(_)));

                for i in 0..point_count {
                    let color = if use_scalar_color {
                        let scalars = pc.scalars.as_ref().unwrap();
                        let scalar = scalars.get(i).copied().unwrap_or(0.0);
                        if let Some(frustum_core::Material::ScalarMapped(sm)) = material {
                            scalar_to_color(scalar, sm)
                        } else {
                            default_point_color
                        }
                    } else if let Some(mat) = material {
                        get_solid_color(mat)
                    } else {
                        default_point_color
                    };

                    point_vertices.push(SimpleVertex {
                        position: [pc.positions[i * 3], pc.positions[i * 3 + 1], pc.positions[i * 3 + 2]],
                        color,
                    });
                }
            }
            SceneElement::Polyline(line) => {
                let vertex_count = line.positions.len() / 3;

                let material = line.material_id.as_ref().and_then(|id| scene.get_material(id));
                let color = material.map(|m| get_solid_color(m)).unwrap_or(default_line_color);

                for i in 0..(vertex_count.saturating_sub(1)) {
                    line_vertices.push(SimpleVertex {
                        position: [line.positions[i * 3], line.positions[i * 3 + 1], line.positions[i * 3 + 2]],
                        color,
                    });
                    line_vertices.push(SimpleVertex {
                        position: [line.positions[(i + 1) * 3], line.positions[(i + 1) * 3 + 1], line.positions[(i + 1) * 3 + 2]],
                        color,
                    });
                }
            }
            SceneElement::Axes(axes) => {
                let (polylines, axis_labels) = axes.expand();
                let color = default_axis_color; // Axes always use default color (per spec: SolidMaterial only)

                for line in polylines {
                    let vertex_count = line.positions.len() / 3;
                    for i in 0..(vertex_count.saturating_sub(1)) {
                        line_vertices.push(SimpleVertex {
                            position: [line.positions[i * 3], line.positions[i * 3 + 1], line.positions[i * 3 + 2]],
                            color,
                        });
                        line_vertices.push(SimpleVertex {
                            position: [line.positions[(i + 1) * 3], line.positions[(i + 1) * 3 + 1], line.positions[(i + 1) * 3 + 2]],
                            color,
                        });
                    }
                }

                // Convert axis labels to expanded labels for rendering
                // Default label size based on scene bounds extent
                let extent = [
                    scene.bounds.max[0] - scene.bounds.min[0],
                    scene.bounds.max[1] - scene.bounds.min[1],
                    scene.bounds.max[2] - scene.bounds.min[2],
                ];
                let label_size = extent.iter().cloned().fold(0.0f32, f32::max) * 0.03;

                for label in axis_labels {
                    labels.push(primitives::ExpandedLabel {
                        position: label.position,
                        text: label.text,
                        size: label_size,
                        color: default_label_color,
                    });
                }
            }
        }
    }

    (mesh_vertices, point_vertices, line_vertices, labels, max_point_size)
}

fn encode_png(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>, RenderError> {
    use image::{ImageBuffer, Rgba};

    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, pixels.to_vec()).expect("Invalid image dimensions");

    let mut png_data = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_data);
    img.write_to(&mut cursor, image::ImageFormat::Png)?;

    Ok(png_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_test_triangle() {
        let _ = env_logger::builder().is_test(true).try_init();

        let config = RenderConfig {
            width: 256,
            height: 256,
            background: [0.1, 0.1, 0.1, 1.0],
        };

        let png_data = render_test_triangle(&config).expect("Failed to render triangle");

        // PNG should start with magic bytes
        assert!(png_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]));

        // Should be non-trivial size
        assert!(png_data.len() > 1000, "PNG seems too small");

        println!("Generated PNG: {} bytes", png_data.len());
    }
}
