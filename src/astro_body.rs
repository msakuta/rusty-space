use std::collections::HashMap;

use crate::parser::{eval, Arg, Command, Property};

use three_d::*;
use three_d_asset::geometry::TriMesh;
use three_d_asset::io::RawAssets;

fn parse_astro_command<'a, 'src>(
    command: &'a Command<'src>,
) -> Option<(String, &'a [Command<'src>])> {
    if let Command::Com(v) = command {
        if v.len() < 3 || !matches!(v[0], Arg::Str("astro")) {
            return None;
        }
        let block = &v[2];
        let name = &v[1];
        let name = if let Arg::Str(s) = *name {
            s.to_owned()
        } else {
            return None;
        };
        let block = if let Arg::Block(s) = block {
            s
        } else {
            return None;
        };
        Some((name, block))
    } else {
        None
    }
}

pub(crate) fn scan_textures(
    command: &Command,
    textures: &mut Vec<String>,
) -> Option<()> {
    let (_name, block) = parse_astro_command(command)?;
    for com in block {
        match com {
            Command::Prop("texture", Property::Str(value)) => {
                textures.push(value.clone());
            }
            Command::Com(_) => {
                scan_textures(&com, textures);
            }
            _ => (),
        }
    }
    None
}

pub(crate) enum Object {
    Color(Gm<Mesh, ColorMaterial>),
    Physical(Gm<Mesh, PhysicalMaterial>),
}

impl AsRef<dyn three_d::Object> for Object {
    fn as_ref(&self) -> &(dyn three_d::Object + 'static) {
        match self {
            Self::Color(gm) => gm,
            Self::Physical(gm) => gm,
        }
    }
}

pub(crate) struct AstroBody {
    #[allow(dead_code)]
    pub name: String,
    pub radius: f32,
    pub semimajor_axis: f32,
    pub omega: f32,
    pub rotation_omega: f32,
    pub model: Object,
    pub orbit_model: Option<Gm<Mesh, PhysicalMaterial>>,
    pub children: Vec<AstroBody>,
}

pub(crate) struct BodyContext<'a> {
    pub context: &'a Context,
    pub loaded: &'a mut RawAssets,
    pub mesh: &'a TriMesh,
    variables: HashMap<String, f64>,
}

impl<'a> BodyContext<'a> {
    pub fn new(
        context: &'a Context,
        loaded: &'a mut RawAssets,
        mesh: &'a TriMesh,
    ) -> Self {
        Self {
            context,
            loaded,
            mesh,
            variables: HashMap::new(),
        }
    }
}

pub(crate) fn load_astro_bodies(
    commands: &[Command],
    context: &mut BodyContext,
) -> Vec<AstroBody> {
    let mut bodies = vec![];
    for command in commands {
        if let Some(body) = load_astro_body(command, context) {
            bodies.push(body);
        }
        if let Command::Def(name, expr) = command {
            context
                .variables
                .insert((*name).to_owned(), eval(expr, &context.variables));
            println!("variables: {:?}", context.variables);
        }
    }
    bodies
}

pub(crate) fn load_astro_body(
    command: &Command,
    context: &mut BodyContext,
) -> Option<AstroBody> {
    let (name, block) = parse_astro_command(command)?;
    let mut texture = None;
    let mut radius = 0.1;
    let mut semimajor_axis = 1.;
    let mut omega = 1.;
    let mut rotation_omega = 0.;
    let mut star = false;
    let mut children = vec![];
    for com in block {
        match com {
            Command::Prop("texture", Property::Str(value)) => {
                texture = Some(value);
            }
            Command::Prop("radius", Property::Expr(ref value)) => {
                radius = eval(value, &context.variables) as f32;
            }
            Command::Prop("semimajor_axis", Property::Expr(ref value)) => {
                semimajor_axis = eval(value, &context.variables) as f32;
            }
            Command::Prop("orbit_period", Property::Expr(ref value)) => {
                omega = 2. * std::f32::consts::PI
                    / eval(value, &context.variables) as f32;
            }
            Command::Prop("rotation_period", Property::Expr(ref value)) => {
                rotation_omega = 2. * std::f32::consts::PI
                    / eval(value, &context.variables) as f32;
            }
            Command::Prop("star", Property::Expr(ref value)) => {
                star = eval(value, &context.variables) != 0.;
            }
            Command::Prop(prop, _) => {
                println!("Unknown property {prop:?}");
            }
            Command::Com(_) => {
                if let Some(child) = load_astro_body(&com, context) {
                    children.push(child);
                }
            }
            Command::Def(name, expr) => {
                context
                    .variables
                    .insert((*name).to_owned(), eval(expr, &context.variables));
                println!("variables: {:?}", context.variables);
            }
        }
    }

    let model = if let Some(texture) = texture {
        let mesh = Mesh::new(&context.context, &context.mesh);
        if star {
            let mut model = Gm::new(
                mesh,
                ColorMaterial {
                    texture: Some(
                        Texture2D::new(
                            &context.context,
                            &context.loaded.deserialize(texture).unwrap(),
                        )
                        .into(),
                    ),
                    ..Default::default()
                },
            );
            model.material.render_states.cull = Cull::Back;
            Object::Color(model)
        } else {
            let mut model = Gm::new(
                mesh,
                PhysicalMaterial::new(
                    &context.context,
                    &CpuMaterial {
                        roughness: 0.6,
                        metallic: 0.6,
                        lighting_model: LightingModel::Cook(
                            NormalDistributionFunction::TrowbridgeReitzGGX,
                            GeometryFunction::SmithSchlickGGX,
                        ),
                        albedo_texture: Some(
                            context.loaded.deserialize(texture).unwrap(),
                        ),
                        ..Default::default()
                    },
                ),
            );
            model.material.render_states.cull = Cull::Back;
            Object::Physical(model)
        }
    } else {
        let mut mesh_sun = uv_sphere(32);
        mesh_sun.transform(&Matrix4::from_scale(0.3)).unwrap();
        let mut model_sun = Gm::new(
            Mesh::new(&context.context, &mesh_sun),
            PhysicalMaterial::new(
                &context.context,
                &CpuMaterial {
                    roughness: 0.6,
                    metallic: 0.6,
                    lighting_model: LightingModel::Cook(
                        NormalDistributionFunction::TrowbridgeReitzGGX,
                        GeometryFunction::SmithSchlickGGX,
                    ),
                    ..Default::default()
                },
            ),
        );
        model_sun.material.render_states.cull = Cull::Back;
        Object::Physical(model_sun)
    };

    let orbit_model = if 0. < semimajor_axis {
        let mut orbit = Gm::new(
            Mesh::new(&context.context, &ring(64, 0.005)),
            PhysicalMaterial::new_transparent(
                &context.context,
                &CpuMaterial {
                    albedo: Color {
                        r: 0,
                        g: 255,
                        b: 0,
                        a: 200,
                    },
                    ..Default::default()
                },
            ),
        );
        orbit.set_transformation(
            Mat4::from_scale(semimajor_axis) * Mat4::from_angle_z(Deg(90.)),
        );
        Some(orbit)
    } else {
        None
    };

    println!(
        "Adding body {name} radius: {radius}, semimajor_axis: {semimajor_axis}, rotation_omega: {rotation_omega}"
    );
    Some(AstroBody {
        name,
        radius,
        semimajor_axis,
        omega,
        rotation_omega,
        model,
        orbit_model,
        children,
    })
}

pub(crate) fn apply_transform(
    body: &mut AstroBody,
    parent_transform: &Matrix4<f32>,
    frame_time: f64,
) {
    let rotation = parent_transform
        * Matrix4::from_angle_y(Rad(frame_time as f32 * body.omega))
        * Matrix4::from_translation(Vec3::new(body.semimajor_axis, 0., 0.));

    let origin = rotation
        .transform_point(Point3::new(0., 0., 0.))
        .to_homogeneous()
        .truncate();

    let revolution = Matrix4::from_translation(origin)
        * Matrix4::from_angle_y(Rad(frame_time as f32 * body.rotation_omega))
        * Matrix4::from_scale(body.radius)
        * Matrix4::from_angle_x(Deg(-90.));

    match &mut body.model {
        Object::Color(model) => model.set_transformation(revolution),
        Object::Physical(model) => model.set_transformation(revolution),
    }
    // println!("Applying transform to {}: {origin:?}", body.name);
    for child in &mut body.children {
        apply_transform(child, &rotation, frame_time);
    }

    if let Some(ref mut orbit) = body.orbit_model {
        orbit.set_transformation(
            parent_transform
                * Mat4::from_scale(body.semimajor_axis)
                * Mat4::from_angle_z(Deg(90.)),
        );
    }
}

///
/// Returns a sphere mesh with radius 1 and center in `(0, 0, 0)` with UV mapping as longitude and latitude.
///
pub(crate) fn uv_sphere(angle_subdivisions: u32) -> CpuMesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = vec![];

    positions.push(Vec3::new(0.0, 0.0, 1.0));
    normals.push(Vec3::new(0.0, 0.0, 1.0));
    uvs.push(Vec2::new(0., 0.));

    for j in 0..angle_subdivisions * 2 {
        let j1 = (j + 1) % (angle_subdivisions * 2);
        indices.push(0);
        indices.push((1 + j) as u16);
        indices.push((1 + j1) as u16);
    }

    for i in 0..angle_subdivisions - 1 {
        let i_wrap = (i + 1) as f32 / angle_subdivisions as f32;
        let theta =
            std::f32::consts::PI * (i + 1) as f32 / angle_subdivisions as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        let i0 = 1 + i * angle_subdivisions * 2;
        let i1 = 1 + (i + 1) * angle_subdivisions * 2;

        for j in 0..=angle_subdivisions * 2 {
            let j_wrap = j as f32 / (angle_subdivisions * 2) as f32;
            let phi =
                std::f32::consts::PI * j as f32 / angle_subdivisions as f32;
            let x = sin_theta * phi.cos();
            let y = sin_theta * phi.sin();
            let z = cos_theta;
            positions.push(Vec3::new(x, y, z));
            normals.push(Vec3::new(x, y, z));
            uvs.push(Vec2::new(j_wrap, i_wrap));

            if i != angle_subdivisions - 2 {
                let j1 = j + 1;
                indices.push((i0 + j) as u16);
                indices.push((i1 + j1) as u16);
                indices.push((i0 + j1) as u16);
                indices.push((i1 + j1) as u16);
                indices.push((i0 + j) as u16);
                indices.push((i1 + j) as u16);
            }
        }
    }
    positions.push(Vec3::new(0.0, 0.0, -1.0));
    normals.push(Vec3::new(0.0, 0.0, -1.0));
    uvs.push(Vec2::new(0., 0.));

    let i = 1 + (angle_subdivisions - 2) * angle_subdivisions * 2;
    for j in 0..angle_subdivisions * 2 {
        let j1 = (j + 1) % (angle_subdivisions * 2);
        indices.push((i + j) as u16);
        indices.push(
            ((angle_subdivisions - 1) * angle_subdivisions * 2 + 1) as u16,
        );
        indices.push((i + j1) as u16);
    }

    three_d_asset::geometry::TriMesh {
        indices: Indices::U16(indices),
        positions: Positions::F32(positions),
        normals: Some(normals),
        uvs: Some(uvs),
        ..Default::default()
    }
}

///
/// Returns a ring-like shape mesh around the x-axis in the range `[0..1]` and with radius 1.
/// It has "fake" thickness by a cross section with a shape of "+".
///
fn ring(angle_subdivisions: u32, ring_thickness: f32) -> TriMesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for i in 0..2 {
        let x = (i * 2 - 1) as f32 * ring_thickness;
        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32
                / angle_subdivisions as f32;

            positions.push(Vec3::new(x, angle.cos(), angle.sin()));
        }
    }
    for i in 0..2 {
        let r = (i * 2 - 1) as f32 * ring_thickness + 1.;
        for j in 0..angle_subdivisions {
            let angle = 2.0 * std::f32::consts::PI * j as f32
                / angle_subdivisions as f32;
            positions.push(Vec3::new(0., r * angle.cos(), r * angle.sin()));
        }
    }
    for j in 0..angle_subdivisions {
        indices.push(j as u16);
        indices.push(((j + 1) % angle_subdivisions) as u16);
        indices
            .push((angle_subdivisions + (j + 1) % angle_subdivisions) as u16);

        indices.push((j) as u16);
        indices
            .push((angle_subdivisions + (j + 1) % angle_subdivisions) as u16);
        indices.push(((1) * angle_subdivisions + j) as u16);
    }
    let offset = 2 * angle_subdivisions;
    for j in 0..angle_subdivisions {
        indices.push((j + offset) as u16);
        indices.push(((j + 1) % angle_subdivisions + offset) as u16);
        indices.push(
            (angle_subdivisions + (j + 1) % angle_subdivisions + offset) as u16,
        );

        indices.push((j + offset) as u16);
        indices.push(
            (angle_subdivisions + (j + 1) % angle_subdivisions + offset) as u16,
        );
        indices.push((angle_subdivisions + j + offset) as u16);
    }
    let mut mesh = TriMesh {
        positions: Positions::F32(positions),
        indices: Indices::U16(indices),
        ..Default::default()
    };
    mesh.compute_normals();
    mesh
}
