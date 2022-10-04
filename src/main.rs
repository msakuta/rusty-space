mod parser;

use crate::parser::expr;

// fn main() {
//     // for line in std::io::stdin().lines().flatten() {
//     //     match expr(&line) {
//     //         Ok(ast) => println!("Parsed AST: {ast:#?}"),
//     //         Err(err) => println!("Parse error: {err:?}"),
//     //     }
//     // }
//     run();
// }

#[tokio::main]
async fn main() {
    run().await;
}

use three_d::*;

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Environment!".to_string(),
        min_size: (512, 512),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(-3.0, 1.0, 2.5),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let mut loaded = three_d_asset::io::load_async(&[
        "hipparcossq.jpg",
        "land_ocean_ice_cloud_2048.jpg",
    ])
    .await
    .unwrap();

    let top_tex = loaded.deserialize("hipparcossq").unwrap();
    let skybox = Skybox::new(
        &context, &top_tex, &top_tex, &top_tex, &top_tex, &top_tex, &top_tex,
    );

    let light = AmbientLight::new(&context, 1.0, Color::WHITE);
    let directional = DirectionalLight::new(
        &context,
        2.0,
        Color::WHITE,
        &vec3(0.0, -1.0, -1.0),
    );

    let mesh = uv_sphere(32);
    let mut model = Gm::new(
        Mesh::new(&context, &mesh),
        ColorMaterial {
            texture: Some(std::sync::Arc::new(Texture2D::new(
                &context,
                &loaded.deserialize("land_ocean_ice_cloud_2048").unwrap(),
            ))),
            ..Default::default()
        },
    );
    model.material.render_states.cull = Cull::Back;

    let mut mesh_sun = uv_sphere(32);
    mesh_sun.transform(&Matrix4::from_scale(0.3)).unwrap();
    let mut model_sun = Gm::new(
        Mesh::new(&context, &mesh_sun),
        ColorMaterial {
            color: Color::WHITE,
            ..Default::default()
        },
    );
    model_sun.material.render_states.cull = Cull::Back;

    // main loop
    window.render_loop(move |mut frame_input| {
        let viewport = Viewport {
            x: 0,
            y: 0,
            width: frame_input.viewport.width,
            height: frame_input.viewport.height,
        };
        camera.set_viewport(viewport);
        control.handle_events(&mut camera, &mut frame_input.events);

        model.set_transformation(
            Matrix4::from_angle_y(Deg(
                frame_input.accumulated_time as f32 * 0.02
            )) * Matrix4::from_translation(Vec3::new(2., 0., 0.))
                * Matrix4::from_angle_y(Deg(frame_input.accumulated_time
                    as f32
                    * 0.1))
                * Matrix4::from_scale(0.1)
                * Matrix4::from_angle_x(Deg(-90.)),
        );

        frame_input.screen().clear(ClearState::default()).render(
            &camera,
            &[&skybox, &model, &model_sun],
            &[&light, &directional],
        );

        FrameOutput::default()
    });
}

///
/// Returns a sphere mesh with radius 1 and center in `(0, 0, 0)` with UV mapping as longitude and latitude.
///
pub fn uv_sphere(angle_subdivisions: u32) -> CpuMesh {
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
