use bevy::camera::primitives::Aabb;
use bevy::mesh::VertexAttributeValues;
use eds_bevy_common::physics::{
    AngularDamping, ColliderConstructor, CollisionLayers, Friction, LinearDamping, Restitution,
    RigidBody,
};
use rustc_hash::FxHashSet;

use bevy::prelude::*;
use eds_bevy_common::prelude::*;

pub struct VoxelizePlugin;

impl Plugin for VoxelizePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            handle_effect, //.run_if(in_state(LevelState::Configuring)),
        );
    }
}

/// Request to take a mesh and emit a new component with
/// a submesh for each connected part.
///
/// If set to 0.0, this is ignored.
#[derive(Debug, Component, Clone, Reflect)]
// #[require(ConfigureBeforePlaying)]
#[reflect(Component, Clone, Default)]
#[type_path = "game"]
pub struct SeparateDisconnected {
    /// Size in meters of each cube
    pub size: f32,
}

impl Default for SeparateDisconnected {
    fn default() -> Self {
        Self { size: 0.25 }
    }
}

fn handle_effect(
    effect_q: Query<(Entity, &SeparateDisconnected)>,
    child_q: Query<&Children>,
    info_q: Query<
        (
            Entity,
            &Mesh3d,
            &MeshMaterial3d<StandardMaterial>,
            &Transform,
            &Aabb,
            Option<&Name>,
            Option<&Friction>,
            Option<&RigidBody>,
            Option<&Restitution>,
            Option<&LinearDamping>,
            Option<&AngularDamping>,
            Option<&CollisionLayers>,
            Option<&AssignDetailNormal>,
        ),
        Added<Aabb>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let mut count = 0;
    for (root, eff) in effect_q.iter() {
        for ent in child_q.iter_descendants(root) {
            let Ok((
                ent,
                mesh,
                mat,
                xfrm,
                aabb,
                name_opt,
                rigid_opt,
                friction_opt,
                rest_opt,
                ldamp_opt,
                adamp_opt,
                layers_opt,
                adn_opt,
            )) = info_q.get(ent)
            else {
                continue;
            };

            let Some(mesh) = meshes.get(&mesh.0) else {
                continue;
            };
            let mut mesh = mesh.clone();

            // Ensure we capture the whole range of points, in local space.
            let (mesh_min, mesh_max) = compute_extents(&mesh);
            let full_extents = mesh_max - mesh_min;

            let Some(partial_meshes) = extract_meshes(&mut mesh) else {
                continue
            };
            for (partial_mesh, cube_center) in partial_meshes
            {
                let mut ent_commands = commands.spawn((
                    ChildOf(root),
                    Mesh3d(meshes.add(partial_mesh)),
                    Transform::from_translation(cube_center),
                    Name::new(if let Some(name) = name_opt {
                        format!("{name}.{count}")
                    } else {
                        format!("<piece>.{count}")
                    }),
                    ColliderConstructor::ConvexHullFromMesh,
                ));
                count += 1;

                ent_commands.try_insert(mat.clone());

                // GROAN copy all the other components.
                // TODO: figure out how to use EntityCloner...
                if let Some(c) = friction_opt {
                    ent_commands.try_insert(*c);
                }
                if let Some(c) = rigid_opt {
                    ent_commands.try_insert(*c);
                }
                if let Some(c) = friction_opt {
                    ent_commands.try_insert(*c);
                }
                if let Some(c) = rest_opt {
                    ent_commands.try_insert(*c);
                }
                if let Some(c) = ldamp_opt {
                    ent_commands.try_insert(*c);
                }
                if let Some(c) = adamp_opt {
                    ent_commands.try_insert(*c);
                }
                if let Some(c) = layers_opt {
                    ent_commands.try_insert(*c);
                }
                if let Some(c) = adn_opt {
                    ent_commands.try_insert(c.clone());
                }

                info!("Split {ent} into {count} parts");

                // Remove the original large object.
                let mut ent_commands = commands.entity(ent);
                ent_commands.despawn();
            }

        }
    }
}

fn compute_extents(mesh: &Mesh) -> (Vec3, Vec3) {
    let mut min = Vec3::INFINITY;
    let mut max = Vec3::NEG_INFINITY;

    let full_pos = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    for pt in full_pos {
        let v = Vec3::new(pt[0], pt[1], pt[2]);
        min = min.min(v);
        max = max.max(v);
    }
    dbg!((min, max))
}

/// Provide one Mesh per each connected bit in `mesh`.
/// Must be indexed mesh,
fn extract_meshes(
    mesh: &mut Mesh,
) -> Option<Vec<(Mesh, Vec3)>> {

    // {
    //     mesh.duplicate_vertices();
    //     let mut orig_pos = mesh
    //         .attribute(Mesh::ATTRIBUTE_POSITION)?.as_float3().unwrap().to_owned();
    //     const QUANT: f32 = 0.001;
    //     let quant = |v| (v / QUANT) * QUANT;
    //     for pos in orig_pos.iter_mut() {
    //         pos[0] = quant(pos[0]);
    //         pos[1] = quant(pos[1]);
    //         pos[2] = quant(pos[2]);
    //     }
    //     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, orig_pos);
    //     mesh.merge_duplicate_vertices().unwrap();
    // }

    let Some(inds) = mesh.indices() else {
        return None
    };

    let topo = dbg!(mesh.primitive_topology());

    // Extract the attributes we care to replicate.
    let full_pos = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)?
        .as_float3()
        .unwrap();
    let full_normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)?.as_float3().unwrap();
    let full_uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0)? {
        VertexAttributeValues::Float32x2(values) => values,
        _ => return None,
    };

    // let center = center.clone();
    // let transform_pt = |ptarr: [f32; 3], center: Vec3| -> [f32; 3] {
    //     [
    //         ptarr[0] - center.x,
    //         ptarr[1] - center.y,
    //         ptarr[2] - center.z,
    //     ]
    // };

    let mut pos = Vec::<Vec3>::with_capacity(full_pos.len());
    let mut normals = Vec::<Vec3>::with_capacity(full_pos.len());
    let mut uvs: Vec::<Vec2> = Vec::with_capacity(full_pos.len());
    let mut indices: Vec::<u32> = Vec::with_capacity(full_pos.len());

    let tris = match dbg!(topo) {
        wgpu::PrimitiveTopology::PointList |
        wgpu::PrimitiveTopology::LineList |
        wgpu::PrimitiveTopology::LineStrip => vec![],
        wgpu::PrimitiveTopology::TriangleList => inds.iter().array_chunks().map(|[a,b,c]|  [a as u32, b as u32, c as u32]).collect::<Vec<_>>(),
        wgpu::PrimitiveTopology::TriangleStrip => {
            panic!("wrong ordering");
            inds.iter().map_windows(|&[a,b,c]|  [a as u32, b as u32, c as u32]).collect::<Vec<_>>()
        }
    };

    // We generate a mesh potentially for every triangle,
    // unless we find that other triangles use the same indices,
    // i.e. are connected.
    let mut to_visit_tris = FxHashSet::<u32>::from_iter(0..tris.len() as u32);

    let mut meshes = vec![];

    // As long as we have at least one triangle left, make a new mesh.
    while !to_visit_tris.is_empty() {

        let mut visited_indices = FxHashSet::<u32>::default();
        // Add each triangle touching the first.
        let to_visit_list = to_visit_tris.clone();
        for tri_idx in to_visit_list {
            let [ind0, ind1, ind2] = tris[tri_idx as usize].clone();

            let any_shared_verts = visited_indices.contains(&ind0)
                || visited_indices.contains(&ind1)
                || visited_indices.contains(&ind2);

            // Add an entry if we're starting this mesh,
            // and add any that share verts.
            if !visited_indices.is_empty() && !any_shared_verts {
                // Leave this triangle for later.
                continue;
            }

                // dbg!(tri_idx);
            to_visit_tris.remove(&tri_idx);

            // Add this triangle to a mesh.
            visited_indices.insert(ind0);
            visited_indices.insert(ind1);
            visited_indices.insert(ind2);
        }

        dbg!(visited_indices.len());

        /*
        for index in visited_indices.iter()
        {
            let index = *index;
            let (ind0, ind1, ind2) = (ind0 as usize, ind1 as usize, ind2 as usize);
            let pos0 = full_pos[ind0];
            let pos1 = full_pos[ind1];
            let pos2 = full_pos[ind2];

            let l = pos.len() as u32;
            indices.push([l, l + 1, l + 2]);

            pos.push(transform_pt(pos0, center));
            pos.push(transform_pt(pos1, center));
            pos.push(transform_pt(pos2, center));

            normals.push(full_normals[ind0]);
            normals.push(full_normals[ind1]);
            normals.push(full_normals[ind2]);

            uvs.push(full_uvs[ind0]);
            uvs.push(full_uvs[ind1]);
            uvs.push(full_uvs[ind2]);
        }

        // Export it.
        let mut mesh = Mesh::new(
            wgpu::PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(pos.clone()),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));

        if let Err(err) = mesh.generate_tangents() {
            warn!("failed to generate tangents: {err}");
        }

        meshes.push(mesh);
 */
        visited_indices.clear();
    }

    Some(meshes)
}

fn contains_pt(pt: &[f32; 3], center: Vec3, half_size: Vec3) -> bool {
    pt[0] >= center.x - half_size.x
        && pt[0] <= center.x + half_size.x
        && pt[1] >= center.y - half_size.y
        && pt[1] <= center.y + half_size.y
        && pt[2] >= center.z - half_size.z
        && pt[2] <= center.z + half_size.z
}
