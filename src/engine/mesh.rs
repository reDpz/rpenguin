use crate::engine::vert::Vert;
use crate::engine::vert::VertexBufferLayoutDescriptor;

#[derive(Clone)]
pub struct Mesh<V: VertexBufferLayoutDescriptor> {
    vertices: Vec<V>,
    indices: Vec<u16>,
}

impl<V: VertexBufferLayoutDescriptor + Clone> Mesh<V> {
    /// Given a reference to an array of Meshes will output a vertex array and index array
    // TODO: possibly refactor?
    pub fn to_vertex_indices(meshes: &[Mesh<V>]) -> (Vec<V>, Vec<u16>) {
        let mut indices;
        let mut vertices;
        // avoid resizing which may be expensive
        {
            let mut index_capacity = 0;
            let mut vertex_capacity = 0;
            for mesh in meshes {
                vertex_capacity += mesh.vertices.len();
                index_capacity += mesh.indices.len();
            }
            indices = Vec::with_capacity(index_capacity);
            vertices = Vec::with_capacity(vertex_capacity);
        };

        // just add the meshes
        for mesh in meshes {
            let len = vertices.len();
            let mut mesh_indices = mesh.indices.clone();
            if len != 0 {
                // this will add the len of vertices to this offsetting the indices correctly
                mesh_indices.iter_mut().for_each(|i| *i += len as u16)
            }

            indices.append(&mut mesh_indices);
            vertices.append(&mut mesh.vertices.clone());
        }

        (vertices, indices)
    }
}

impl Mesh<Vert> {
    pub fn cube(position: (f32, f32, f32), size: (f32, f32, f32)) -> Mesh<Vert> {
        let (vertices, indices) = Vert::cube(position, size);
        Mesh { vertices, indices }
    }
}
