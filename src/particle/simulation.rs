use rand::{thread_rng, Rng, RngCore};
use rayon::prelude::*;

use crate::VertexBufferLayoutDescriptor;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub position: glam::Vec2,
    pub velocity: glam::Vec2,
    pub color: glam::Vec3,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: glam::Vec2::splat(0.0),
            velocity: glam::Vec2::splat(0.0),
            color: glam::Vec3::splat(1.0),
        }
    }
}

pub struct NBodySimulation {
    pub particles: Vec<Particle>,
}

impl Default for NBodySimulation {
    fn default() -> Self {
        let mut particles = Vec::new();
        for _ in 0..10 {
            particles.push(Particle::default())
        }
        Self { particles }
    }
}

impl NBodySimulation {
    /// Produces a grid of particles with the same amount of particles per row/column
    pub fn grid(particle_count: usize, spacing: f32) -> Self {
        let mut rng = thread_rng();
        // my first attempt at using iterators (i barely understand any of what im doing)
        // when i finish my functional programming course i should understand what this is doing
        // and what it is under the hood
        let mut particles = Vec::new();
        for x in 0..particle_count {
            for y in 0..particle_count {
                particles.push(Particle {
                    position: glam::Vec2 {
                        x: x as f32 * spacing,
                        y: y as f32 * spacing,
                    },
                    velocity: glam::Vec2 {
                        x: rng.gen_range(-1.0..1.0),
                        y: rng.gen_range(-1.0..1.0),
                    }
                    .normalize()
                        * 5.0,
                    color: glam::Vec3 {
                        x: rng.next_u32() as f32,
                        y: rng.next_u32() as f32,
                        z: rng.next_u32() as f32,
                    }
                    .normalize(),
                })
            }
        }

        Self { particles }
    }

    // TODO: Add the color

    // NOTE: yes this is indeed slow however i dont think there is a faster method of doing it

    /// gives you a mat4x4 of the translations of each particle, uses rayon to parallelise this
    /// process
    pub fn instances(&self) -> Vec<glam::Mat4> {
        self.particles
            .par_iter()
            .map(|p| {
                glam::Mat4::from_translation(glam::Vec3 {
                    x: p.position[0],
                    y: p.position[1],
                    z: 0.0,
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn update(&mut self, delta: f32) {
        self.particles.par_iter_mut().for_each(|p| {
            p.position += p.velocity * delta;
        })
    }
}

impl VertexBufferLayoutDescriptor for glam::Mat4 {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<glam::Mat4>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
