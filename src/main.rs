mod astro_body;
mod parser;

use crate::{
    astro_body::{load_astro_body, uv_sphere, BodyContext},
    parser::{commands, Command},
};

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    let s = std::fs::read_to_string("sol.txt")?;
    let commands = {
        println!("source: {s:?}");
        let (_, commands) = commands(&s).map_err(|e| e.to_string())?;
        println!("commands: {commands:#?}");
        commands
    };
    run(commands).await;
    Ok(())
}

use astro_body::apply_transform;
use three_d::*;

pub async fn run<'src>(commands: Vec<Command<'src>>) {
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
    let mut bodies = vec![];
    let mut body_context = BodyContext {
        context: &context,
        loaded: &mut loaded,
        mesh: &mesh,
    };
    for command in commands {
        if let Some(body) = load_astro_body(command, &mut body_context) {
            bodies.push(body);
        }
    }

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

        for body in &mut bodies {
            apply_transform(
                body,
                &Matrix4::identity(),
                frame_input.accumulated_time,
            );
        }

        let render_models: Vec<&dyn three_d::Object> =
            bodies.iter().map(|body| &body.model as _).collect();

        frame_input
            .screen()
            .clear(ClearState::default())
            .render(&camera, &[&skybox], &[&light, &directional])
            .render(&camera, &render_models[..], &[&light, &directional]);

        FrameOutput::default()
    });
}
