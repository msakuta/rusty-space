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
pub(crate) struct AstroBody {
    #[allow(dead_code)]
    pub name: String,
    pub radius: f32,
    pub semimajor_axis: f32,
    pub omega: f32,
    pub rotation_omega: f32,
    pub model: Gm<Mesh, ColorMaterial>,
    pub children: Vec<AstroBody>,
}

pub(crate) struct BodyContext<'a> {
    pub context: &'a Context,
    pub loaded: &'a mut RawAssets,
    pub mesh: &'a TriMesh,
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
    let mut children = vec![];
    for com in block {
        match com {
            Command::Prop("texture", Property::Str(value)) => {
                texture = Some(value);
            }
            Command::Prop("radius", Property::Expr(ref value)) => {
                radius = eval(value) as f32;
            }
            Command::Prop("semimajor_axis", Property::Expr(ref value)) => {
                semimajor_axis = eval(value) as f32;
            }
            Command::Prop("omega", Property::Expr(ref value)) => {
                omega = eval(value) as f32;
            }
            Command::Prop("rotation_omega", Property::Expr(ref value)) => {
                rotation_omega = eval(value) as f32;
            }
            Command::Prop(prop, _) => {
                println!("Unknown property {prop:?}");
            }
            Command::Com(_) => {
                if let Some(child) = load_astro_body(&com, context) {
                    children.push(child);
                }
            }
        }
    }

    let mut model = if let Some(texture) = texture {
        Gm::new(
            Mesh::new(&context.context, &context.mesh),
            ColorMaterial {
                texture: Some(std::sync::Arc::new(Texture2D::new(
                    &context.context,
                    &context.loaded.deserialize(texture).unwrap(),
                ))),
                ..Default::default()
            },
        )
    } else {
        let mut mesh_sun = uv_sphere(32);
        mesh_sun.transform(&Matrix4::from_scale(0.3)).unwrap();
        let mut model_sun = Gm::new(
            Mesh::new(&context.context, &mesh_sun),
            ColorMaterial {
                color: Color::WHITE,
                ..Default::default()
            },
        );
        model_sun.material.render_states.cull = Cull::Back;
        model_sun
    };

    println!(
        "Adding body {name} radius: {radius}, semimajor_axis: {semimajor_axis}, rotation_omega: {rotation_omega}"
    );
    model.material.render_states.cull = Cull::Back;
    Some(AstroBody {
        name,
        radius,
        semimajor_axis,
        omega,
        rotation_omega,
        model,
        children,
    })
}

pub(crate) fn apply_transform(
    body: &mut AstroBody,
    parent_transform: &Matrix4<f32>,
    frame_time: f64,
) {
    let rotation = parent_transform
        * Matrix4::from_angle_y(Deg(frame_time as f32 * body.omega))
        * Matrix4::from_translation(Vec3::new(body.semimajor_axis, 0., 0.));

    let origin = rotation
        .transform_point(Point3::new(0., 0., 0.))
        .to_homogeneous()
        .truncate();

    let revolution = Matrix4::from_translation(origin)
        * Matrix4::from_angle_y(Deg(frame_time as f32 * body.rotation_omega))
        * Matrix4::from_scale(body.radius)
        * Matrix4::from_angle_x(Deg(-90.));

    body.model.set_transformation(revolution);
    // println!("Applying transform to {}: {origin:?}", body.name);
    for child in &mut body.children {
        apply_transform(child, &rotation, frame_time);
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
        name: "sphere".to_string(),
        indices: Some(Indices::U16(indices)),
        positions: Positions::F32(positions),
        normals: Some(normals),
        uvs: Some(uvs),
        ..Default::default()
    }
}
