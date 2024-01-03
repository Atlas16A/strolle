use std::mem;

use bevy::ecs::system::{Res, ResMut};
use bevy::render::mesh::{Mesh as BevyMesh, VertexAttributeValues};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::texture::Image as BevyImage;

use crate::images::Images;
use crate::instances::Instances;
use crate::lights::Lights;
use crate::materials::Materials;
use crate::meshes::Meshes;
use crate::{
    ExtractedImageData, ExtractedImages, ExtractedInstances, ExtractedLights,
    ExtractedMaterials, ExtractedMeshes, ExtractedSun, Image, ImageData,
    Instance, Mesh, MeshTriangle, Sun,
};

pub(crate) fn meshes(
    mut meshes: ResMut<Meshes>,
    mut extracted: ResMut<ExtractedMeshes>,
) {
    for handle in mem::take(&mut extracted.removed)
        .into_iter()
        .chain(extracted.changed.iter().map(|mesh| mesh.handle))
    {
        meshes.remove(handle);
    }

    for entry in mem::take(&mut extracted.changed) {
        if entry.mesh.primitive_topology() != PrimitiveTopology::TriangleList {
            continue;
        }

        let mesh_positions = entry
            .mesh
            .attribute(BevyMesh::ATTRIBUTE_POSITION)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no positions", entry.handle);
            });

        let mesh_normals = entry
            .mesh
            .attribute(BevyMesh::ATTRIBUTE_NORMAL)
            .and_then(VertexAttributeValues::as_float3)
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no normals", entry.handle);
            });

        let mesh_uvs = entry
            .mesh
            .attribute(BevyMesh::ATTRIBUTE_UV_0)
            .map(|uvs| match uvs {
                VertexAttributeValues::Float32x2(uvs) => uvs,
                _ => {
                    panic!(
                        "mesh {:?} has unsupported format for UVs",
                        entry.handle
                    )
                }
            })
            .map(|uvs| uvs.as_slice())
            .unwrap_or(&[]);

        let mesh_tans = entry
            .mesh
            .attribute(BevyMesh::ATTRIBUTE_TANGENT)
            .map(|uvs| match uvs {
                VertexAttributeValues::Float32x4(tangents) => tangents,
                _ => panic!(
                    "mesh {:?} has unsupported format for tangents",
                    entry.handle
                ),
            })
            .map(|tangents| tangents.as_slice())
            .unwrap_or(&[]);

        let mesh_indices: Vec<_> = entry
            .mesh
            .indices()
            .unwrap_or_else(|| {
                panic!("mesh {:?} has no indices", entry.handle);
            })
            .iter()
            .collect();

        let mesh_triangles: Vec<_> = mesh_indices
            .chunks(3)
            .map(|vs| {
                let position0 = mesh_positions[vs[0]];
                let position1 = mesh_positions[vs[1]];
                let position2 = mesh_positions[vs[2]];

                let normal0 = mesh_normals[vs[0]];
                let normal1 = mesh_normals[vs[1]];
                let normal2 = mesh_normals[vs[2]];

                let uv0 = mesh_uvs.get(vs[0]).copied().unwrap_or_default();
                let uv1 = mesh_uvs.get(vs[1]).copied().unwrap_or_default();
                let uv2 = mesh_uvs.get(vs[2]).copied().unwrap_or_default();

                let tan0 = mesh_tans.get(vs[0]).copied().unwrap_or_default();
                let tan1 = mesh_tans.get(vs[1]).copied().unwrap_or_default();
                let tan2 = mesh_tans.get(vs[2]).copied().unwrap_or_default();

                MeshTriangle::default()
                    .with_positions([position0, position1, position2])
                    .with_normals([normal0, normal1, normal2])
                    .with_uvs([uv0, uv1, uv2])
                    .with_tangents([tan0, tan1, tan2])
            })
            .collect();

        meshes.add(entry.handle, Mesh::new(mesh_triangles));
    }
}

pub(crate) fn materials(
    mut materials: ResMut<Materials>,
    mut extracted: ResMut<ExtractedMaterials>,
) {
    for handle in mem::take(&mut extracted.removed) {
        materials.remove(handle);
    }

    for entry in mem::take(&mut extracted.changed) {
        materials.add(entry.handle, entry.material);
    }
}

pub(crate) fn images(
    mut images: ResMut<Images>,
    textures: Res<RenderAssets<BevyImage>>,
    mut extracted: ResMut<ExtractedImages>,
) {
    for handle in mem::take(&mut extracted.removed) {
        images.remove(handle);
    }

    for entry in mem::take(&mut extracted.changed) {
        if entry.texture_descriptor.dimension != wgpu::TextureDimension::D2 {
            continue;
        }

        let data = match entry.data {
            ExtractedImageData::Raw { data } => ImageData::Raw { data },

            ExtractedImageData::Texture { is_dynamic } => ImageData::Texture {
                texture: textures
                    .get(entry.handle.get())
                    .unwrap()
                    .texture
                    .clone(),
                is_dynamic,
            },
        };

        // TODO we should add only those images which are used by at least one
        //      material, since otherwise we'll .add() textures that are
        //      related solely to UI, for instance
        //
        //      (conversely, we should remove those images which are not in use
        //      by any material)
        //
        //      that's not so easy though because it can happen that an image is
        //      loaded first *and then* (e.g. in next frame) it's used by some
        //      material, in which case a simple condition right here will not
        //      be sufficient
        images.add(
            entry.handle,
            Image::new(
                data,
                entry.texture_descriptor,
                entry.sampler_descriptor,
            ),
        );
    }
}

pub(crate) fn instances(
    mut instances: ResMut<Instances>,
    mut extracted: ResMut<ExtractedInstances>,
) {
    for handle in mem::take(&mut extracted.removed) {
        instances.remove(handle);
    }

    for entry in mem::take(&mut extracted.changed) {
        instances.add(
            entry.handle,
            Instance::new(
                entry.mesh_handle,
                entry.material_handle,
                entry.xform,
            ),
        );
    }
}

pub(crate) fn lights(
    mut lights: ResMut<Lights>,
    mut extracted: ResMut<ExtractedLights>,
) {
    for handle in mem::take(&mut extracted.removed) {
        lights.remove(handle);
    }

    for entry in mem::take(&mut extracted.changed) {
        lights.add(entry.handle, entry.light);
    }
}

pub(crate) fn sun(mut sun: ResMut<Sun>, mut extracted: ResMut<ExtractedSun>) {
    if let Some(extracted_sun) = extracted.sun.take() {
        *sun = extracted_sun;
    }
}
