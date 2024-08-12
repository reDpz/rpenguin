use rand::{thread_rng, Rng, RngCore};
use rayon::prelude::*;

use crate::VertexBufferLayoutDescriptor;

pub struct Particle {
    pub position: glam::Vec2,
    pub velocity: glam::Vec2,
    pub color: glam::Vec3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleInstance {
    pub position: glam::Vec2,
    pub color: glam::Vec3,
}

impl VertexBufferLayoutDescriptor for ParticleInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl Particle {
    pub fn to_instance(&self) -> ParticleInstance {
        ParticleInstance {
            position: self.position,
            color: self.color,
        }
    }

    #[inline]
    pub fn distance_to(&self, other: &Self) -> f32 {
        (self.position - other.position).length()
    }

    #[inline]
    pub fn distance_to_squared(&self, other: &Self) -> f32 {
        (self.position - other.position).length_squared()
    }

    #[inline]
    pub fn is_colliding_with(&self, with: &Self, radius: f32) -> bool {
        self.distance_to(with) <= radius
    }

    #[inline]
    pub fn is_colliding_with_squared(&self, with: &Self, radius_squared: f32) -> bool {
        self.distance_to_squared(with) <= radius_squared
    }
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
    pub radius: f32,
}

impl Default for NBodySimulation {
    fn default() -> Self {
        let mut particles = Vec::new();
        for _ in 0..10 {
            particles.push(Particle::default())
        }
        Self {
            particles,
            radius: 15.0,
        }
    }
}

impl NBodySimulation {
    /// Produces a grid of particles with the same amount of particles per row/column
    pub fn grid(particle_count: usize, spacing: f32, radius: f32) -> Self {
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

        Self { particles, radius }
    }

    // TODO: Add the color

    // NOTE: yes this is indeed slow however i dont think there is a faster method of doing it

    /// gives you a mat4x4 of the translations of each particle, uses rayon to parallelise this
    /// process
    pub fn instances(&self) -> Vec<ParticleInstance> {
        self.particles
            .par_iter()
            .map(Particle::to_instance)
            .collect::<Vec<_>>()
    }

    pub fn update(&mut self, delta: f32) {
        // actual nbody sim
        // let len = self.particles.len();
        // apply velocities
        self.particles.par_iter_mut().for_each(|p| {
            let to_center = glam::Vec2::ZERO - p.position;
            let length = to_center.length();
            if length > 0.0 {
                let pull_strength = 0.25 / length;
                p.velocity += to_center * delta * length * pull_strength;
            }
            p.position += p.velocity * delta;
        });
    }

    pub fn center(&self) -> glam::Vec2 {
        let mut center = glam::Vec2::ZERO;
        let len = self.particles.len();
        for i in 0..len {
            center += self.particles[i].position;
        }

        center / len as f32
    }
}
