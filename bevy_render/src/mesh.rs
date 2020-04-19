use crate::{
    pipeline::{
        state_descriptors::{IndexFormat, PrimitiveTopology},
        VertexBufferDescriptor, VertexFormat,
    },
    render_resource::AssetBatchers,
    Renderable,
};
use bevy_asset::Handle;
use glam::*;
use legion::prelude::*;
use std::borrow::Cow;
use thiserror::Error;
use zerocopy::AsBytes;

pub const VERTEX_BUFFER_ASSET_INDEX: usize = 0;
pub const INDEX_BUFFER_ASSET_INDEX: usize = 1;
#[derive(Clone, Debug)]
pub enum VertexAttributeValues {
    Float(Vec<f32>),
    Float2(Vec<[f32; 2]>),
    Float3(Vec<[f32; 3]>),
    Float4(Vec<[f32; 4]>),
}

impl VertexAttributeValues {
    pub fn len(&self) -> usize {
        match *self {
            VertexAttributeValues::Float(ref values) => values.len(),
            VertexAttributeValues::Float2(ref values) => values.len(),
            VertexAttributeValues::Float3(ref values) => values.len(),
            VertexAttributeValues::Float4(ref values) => values.len(),
        }
    }

    // TODO: add vertex format as parameter here and perform type conversions
    pub fn get_bytes(&self) -> &[u8] {
        match *self {
            VertexAttributeValues::Float(ref values) => values.as_bytes(),
            VertexAttributeValues::Float2(ref values) => values.as_bytes(),
            VertexAttributeValues::Float3(ref values) => values.as_bytes(),
            VertexAttributeValues::Float4(ref values) => values.as_bytes(),
        }
    }
}

impl From<&VertexAttributeValues> for VertexFormat {
    fn from(values: &VertexAttributeValues) -> Self {
        match values {
            VertexAttributeValues::Float(_) => VertexFormat::Float,
            VertexAttributeValues::Float2(_) => VertexFormat::Float2,
            VertexAttributeValues::Float3(_) => VertexFormat::Float3,
            VertexAttributeValues::Float4(_) => VertexFormat::Float4,
        }
    }
}

#[derive(Debug)]
pub struct VertexAttribute {
    pub name: Cow<'static, str>,
    pub values: VertexAttributeValues,
}

impl VertexAttribute {
    pub const POSITION: &'static str = "Vertex_Position";
    pub const NORMAL: &'static str = "Vertex_Normal";
    pub const UV: &'static str = "Vertex_Uv";

    pub fn position(positions: Vec<[f32; 3]>) -> Self {
        VertexAttribute {
            name: Self::POSITION.into(),
            values: VertexAttributeValues::Float3(positions),
        }
    }

    pub fn normal(normals: Vec<[f32; 3]>) -> Self {
        VertexAttribute {
            name: Self::NORMAL.into(),
            values: VertexAttributeValues::Float3(normals),
        }
    }

    pub fn uv(uvs: Vec<[f32; 2]>) -> Self {
        VertexAttribute {
            name: Self::UV.into(),
            values: VertexAttributeValues::Float2(uvs),
        }
    }
}

#[derive(Error, Debug)]
pub enum MeshToVertexBufferError {
    #[error("VertexBufferDescriptor requires a VertexBufferAttribute this Mesh does not contain.")]
    MissingVertexAttribute { attribute_name: Cow<'static, str> },
    #[error("Mesh VertexAttribute VertexFormat is incompatible with VertexBufferDescriptor VertexAttribute VertexFormat.")]
    IncompatibleVertexAttributeFormat {
        attribute_name: Cow<'static, str>,
        descriptor_format: VertexFormat,
        mesh_format: VertexFormat,
    },
}

#[derive(Debug)]
pub struct Mesh {
    pub primitive_topology: PrimitiveTopology,
    pub attributes: Vec<VertexAttribute>,
    pub indices: Option<Vec<u32>>,
}

impl Mesh {
    pub fn new(primitive_topology: PrimitiveTopology) -> Self {
        Mesh {
            primitive_topology,
            attributes: Vec::new(),
            indices: None,
        }
    }

    pub fn get_vertex_buffer_bytes(
        &self,
        vertex_buffer_descriptor: &VertexBufferDescriptor,
    ) -> Result<Vec<u8>, MeshToVertexBufferError> {
        let length = self.attributes.first().map(|a| a.values.len()).unwrap_or(0);
        let mut bytes = vec![0; vertex_buffer_descriptor.stride as usize * length];

        for vertex_attribute in vertex_buffer_descriptor.attributes.iter() {
            match self
                .attributes
                .iter()
                .find(|a| vertex_attribute.name == a.name)
            {
                Some(mesh_attribute) => {
                    let attribute_bytes = mesh_attribute.values.get_bytes();
                    let attribute_size = vertex_attribute.format.get_size() as usize;
                    for (i, vertex_slice) in attribute_bytes.chunks(attribute_size).enumerate() {
                        let vertex_offset = vertex_buffer_descriptor.stride as usize * i;
                        let attribute_offset = vertex_offset + vertex_attribute.offset as usize;
                        bytes[attribute_offset..attribute_offset + attribute_size]
                            .copy_from_slice(vertex_slice);
                    }
                }
                None => {
                    return Err(MeshToVertexBufferError::MissingVertexAttribute {
                        attribute_name: vertex_attribute.name.clone(),
                    })
                }
            }
        }

        Ok(bytes)
    }

    pub fn get_index_buffer_bytes(&self, index_format: IndexFormat) -> Option<Vec<u8>> {
        self.indices.as_ref().map(|indices| match index_format {
            IndexFormat::Uint16 => indices
                .iter()
                .map(|i| *i as u16)
                .collect::<Vec<u16>>()
                .as_bytes()
                .to_vec(),
            IndexFormat::Uint32 => indices.as_bytes().to_vec(),
        })
    }
}

pub mod shape {
    use super::{Mesh, VertexAttribute};
    use crate::pipeline::state_descriptors::PrimitiveTopology;
    use glam::*;

    pub struct Cube;

    impl From<Cube> for Mesh {
        fn from(_: Cube) -> Self {
            let vertices = &[
                // top (0., 0., 1.)
                ([-1., -1., 1.], [0., 0., 1.], [0., 0.]),
                ([1., -1., 1.], [0., 0., 1.], [1., 0.]),
                ([1., 1., 1.], [0., 0., 1.], [1., 1.]),
                ([-1., 1., 1.], [0., 0., 1.], [0., 1.]),
                // bottom (0., 0., -1.)
                ([-1., 1., -1.], [0., 0., -1.], [1., 0.]),
                ([1., 1., -1.], [0., 0., -1.], [0., 0.]),
                ([1., -1., -1.], [0., 0., -1.], [0., 1.]),
                ([-1., -1., -1.], [0., 0., -1.], [1., 1.]),
                // right (1., 0., 0.)
                ([1., -1., -1.], [1., 0., 0.], [0., 0.]),
                ([1., 1., -1.], [1., 0., 0.], [1., 0.]),
                ([1., 1., 1.], [1., 0., 0.], [1., 1.]),
                ([1., -1., 1.], [1., 0., 0.], [0., 1.]),
                // left (-1., 0., 0.)
                ([-1., -1., 1.], [-1., 0., 0.], [1., 0.]),
                ([-1., 1., 1.], [-1., 0., 0.], [0., 0.]),
                ([-1., 1., -1.], [-1., 0., 0.], [0., 1.]),
                ([-1., -1., -1.], [-1., 0., 0.], [1., 1.]),
                // front (0., 1., 0.)
                ([1., 1., -1.], [0., 1., 0.], [1., 0.]),
                ([-1., 1., -1.], [0., 1., 0.], [0., 0.]),
                ([-1., 1., 1.], [0., 1., 0.], [0., 1.]),
                ([1., 1., 1.], [0., 1., 0.], [1., 1.]),
                // back (0., -1., 0.)
                ([1., -1., 1.], [0., -1., 0.], [0., 0.]),
                ([-1., -1., 1.], [0., -1., 0.], [1., 0.]),
                ([-1., -1., -1.], [0., -1., 0.], [1., 1.]),
                ([1., -1., -1.], [0., -1., 0.], [0., 1.]),
            ];

            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            for (position, normal, uv) in vertices.iter() {
                positions.push(position.clone());
                normals.push(normal.clone());
                uvs.push(uv.clone());
            }

            let indices = vec![
                0, 1, 2, 2, 3, 0, // top
                4, 5, 6, 6, 7, 4, // bottom
                8, 9, 10, 10, 11, 8, // right
                12, 13, 14, 14, 15, 12, // left
                16, 17, 18, 18, 19, 16, // front
                20, 21, 22, 22, 23, 20, // back
            ];

            Mesh {
                primitive_topology: PrimitiveTopology::TriangleStrip,
                attributes: vec![
                    VertexAttribute::position(positions),
                    VertexAttribute::normal(normals),
                    VertexAttribute::uv(uvs),
                ],
                indices: Some(indices),
            }
        }
    }

    pub struct Quad {
        pub size: Vec2,
    }

    impl From<Quad> for Mesh {
        fn from(quad: Quad) -> Self {
            let extent_x = quad.size.x() / 2.0;
            let extent_y = quad.size.y() / 2.0;

            let north_west = vec2(-extent_x, extent_y);
            let north_east = vec2(extent_x, extent_y);
            let south_west = vec2(-extent_x, -extent_y);
            let south_east = vec2(extent_x, -extent_y);
            let vertices = &[
                (
                    [south_west.x(), south_west.y(), 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 1.0],
                ),
                (
                    [north_west.x(), north_west.y(), 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0],
                ),
                (
                    [north_east.x(), north_east.y(), 0.0],
                    [0.0, 0.0, 1.0],
                    [1.0, 0.0],
                ),
                (
                    [south_east.x(), south_east.y(), 0.0],
                    [0.0, 0.0, 1.0],
                    [1.0, 1.0],
                ),
            ];

            let indices = vec![0, 2, 1, 0, 3, 2];

            let mut positions = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            for (position, normal, uv) in vertices.iter() {
                positions.push(position.clone());
                normals.push(normal.clone());
                uvs.push(uv.clone());
            }

            Mesh {
                primitive_topology: PrimitiveTopology::TriangleStrip,
                attributes: vec![
                    VertexAttribute::position(positions),
                    VertexAttribute::normal(normals),
                    VertexAttribute::uv(uvs),
                ],
                indices: Some(indices),
            }
        }
    }

    pub struct Plane {
        pub size: f32,
    }

    impl From<Plane> for Mesh {
        fn from(plane: Plane) -> Self {
            Quad {
                size: Vec2::new(plane.size, plane.size),
            }
            .into()
        }
    }
}

pub fn mesh_batcher_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("mesh_batcher")
        .write_resource::<AssetBatchers>()
        .with_query(
            <(Read<Handle<Mesh>>, Read<Renderable>)>::query().filter(changed::<Handle<Mesh>>()),
        )
        .build(|_, world, asset_batchers, query| {
            for (entity, (mesh_handle, _renderable)) in query.iter_entities(world) {
                asset_batchers.set_entity_handle(entity, *mesh_handle);
            }
        })
}

#[cfg(test)]
mod tests {
    use crate::{Vertex, pipeline::state_descriptors::PrimitiveTopology, shader::AsUniforms};
    use super::{Mesh, VertexAttribute};
    use zerocopy::AsBytes;

    #[test]
    fn test_get_vertex_bytes() {
        let vertices = &[
            ([0., 0., 0.], [1., 1., 1.], [2., 2.]),
            ([3., 3., 3.], [4., 4., 4.], [5., 5.]),
            ([6., 6., 6.], [7., 7., 7.], [8., 8.]),
        ];

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        for (position, normal, uv) in vertices.iter() {
            positions.push(*position);
            normals.push(*normal);
            uvs.push(*uv);
        }

        let mesh = Mesh {
            primitive_topology: PrimitiveTopology::TriangleStrip,
            attributes: vec![
                VertexAttribute::position(positions),
                VertexAttribute::normal(normals),
                VertexAttribute::uv(uvs),
            ],
            indices: None,
        };

        let expected_vertices = &[
            Vertex {
                position: [0., 0., 0.],
                normal: [1., 1., 1.],
                uv: [2., 2.],
            },
            Vertex {
                position: [3., 3., 3.],
                normal: [4., 4., 4.],
                uv: [5., 5.],
            },
            Vertex {
                position: [6., 6., 6.],
                normal: [7., 7., 7.],
                uv: [8., 8.],
            },
        ];

        let descriptor = Vertex::get_vertex_buffer_descriptor().unwrap();
        assert_eq!(mesh.get_vertex_buffer_bytes(descriptor).unwrap(), expected_vertices.as_bytes(), "buffer bytes are equal");
    }
}