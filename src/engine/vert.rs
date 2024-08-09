use std::mem::size_of;
// needed because buffers must be contiguous
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vert {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub trait VertexBufferLayoutDescriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

impl Vert {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0=>Float32x3, 1 => Float32x2];
    // i find this bit pretty interesting, essentially what is happening here is that we don't care
    // how the struct is defined in rust, what we care about here is how it's layout out in memory.

    /// Returns a rectangle starting at position with the appropriate width and height
    pub fn rect(position: (f32, f32, f32), size: (f32, f32)) -> (Vec<Vert>, Vec<u16>) {
        let vertices = vec![
            // top left
            Vert {
                position: [position.0, position.1, position.2],
                tex_coords: [0.0, 0.0],
            },
            // bottom right
            Vert {
                position: [position.0 + size.0, position.1 - size.1, position.2],
                tex_coords: [1.0, 1.0],
            },
            // bottom left
            Vert {
                position: [position.0, position.1 - size.1, position.2],
                tex_coords: [0.0, 1.0],
            },
            // top right
            Vert {
                position: [position.0 + size.0, position.1, position.2],
                tex_coords: [1.0, 0.0],
            },
        ];

        let indices: Vec<u16> = vec![0, 2, 3, 3, 2, 1];

        (vertices, indices)
    }

    pub fn rect_from_center(size: (f32, f32)) -> (Vec<Vert>, Vec<u16>) {
        let vertices = vec![
            // top left
            Vert {
                position: [-(size.0 / 2.0), size.1 / 2.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            // bottom right
            Vert {
                position: [size.0 / 2.0, -(size.1 / 2.0), 0.0],
                tex_coords: [1.0, 1.0],
            },
            // bottom left
            Vert {
                position: [-(size.0 / 2.0), -(size.1 / 2.0), 0.0],
                tex_coords: [0.0, 1.0],
            },
            // top right
            Vert {
                position: [size.0 / 2.0, size.1 / 2.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ];

        let indices: Vec<u16> = vec![0, 2, 3, 3, 2, 1];

        (vertices, indices)
    }

    pub fn cube(position: (f32, f32, f32), size: (f32, f32, f32)) -> (Vec<Vert>, Vec<u16>) {
        let vertices = vec![
            // 0 f top left
            Vert {
                position: [position.0, position.1, position.2],
                tex_coords: [0.0, 0.0],
            },
            // 1 f bottom right
            Vert {
                position: [position.0 + size.0, position.1 - size.1, position.2],
                tex_coords: [1.0, 1.0],
            },
            // 2 f bottom left
            Vert {
                position: [position.0, position.1 - size.1, position.2],
                tex_coords: [0.0, 1.0],
            },
            // 3 f top right
            Vert {
                position: [position.0 + size.0, position.1, position.2],
                tex_coords: [1.0, 0.0],
            },
            // 4 b top left
            Vert {
                position: [position.0, position.1, position.2 - size.2],
                tex_coords: [1.0, 0.0],
            },
            // 5 b bottom right
            Vert {
                position: [
                    position.0 + size.0,
                    position.1 - size.1,
                    position.2 - size.2,
                ],
                tex_coords: [0.0, 1.0],
            },
            // 6 b bottom left
            Vert {
                position: [position.0, position.1 - size.1, position.2 - size.2],
                tex_coords: [1.0, 1.0],
            },
            // 7 b top right
            Vert {
                position: [position.0 + size.0, position.1, position.2 - size.2],
                tex_coords: [0.0, 0.0],
            },
            // 8 left side top right
            Vert {
                position: [position.0, position.1, position.2],
                tex_coords: [1.0, 0.0],
            },
            // 9 left side bottom right
            Vert {
                position: [position.0, position.1 - size.1, position.2],
                tex_coords: [1.0, 1.0],
            },
            // 10 t bottom left
            Vert {
                position: [position.0, position.1, position.2],
                tex_coords: [0.0, 1.0],
            },
            // 11 t bottom right
            Vert {
                position: [position.0 + size.0, position.1, position.2],
                tex_coords: [1.0, 1.0],
            },
            // 12 t top left
            Vert {
                position: [position.0, position.1, position.2 - size.2],
                tex_coords: [0.0, 0.0],
            },
            // 13 t top right
            Vert {
                position: [position.0 + size.0, position.1, position.2 - size.2],
                tex_coords: [1.0, 0.0],
            },
            // 12 r top left
            Vert {
                position: [position.0 + size.0, position.1, position.2],
                tex_coords: [1.0, 0.0],
            },
        ];

        #[rustfmt::skip]
        let indices = vec![
            // front
            0, 2, 3, 3, 2, 1,
            //back
            4, 7, 6, 7, 5, 6,

            // left
            4, 6, 8, 8, 6, 9,

            // right
            3, 1, 5, 3, 5, 7,

            // top
            12, 10, 11, 12, 11, 13,
            // bottom
            2, 6, 5, 2, 5, 1
        ];

        (vertices, indices)
    }
}

impl VertexBufferLayoutDescriptor for Vert {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vert>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}
