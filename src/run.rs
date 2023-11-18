use three_d::*;

use crate::{
    astro_body::{
        apply_transform, load_astro_bodies, scan_textures, uv_sphere,
        AstroBody, BodyContext,
    },
    orbit_control_ex::OrbitControlEx,
    parser::Command,
};

pub async fn run<'src>(commands: Vec<Command<'src>>) {
    let window = Window::new(WindowSettings {
        title: "Rusty-space".to_string(),
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
    let mut control = OrbitControlEx::builder()
        .target(*camera.target())
        .min_distance(0.10)
        .max_distance(100.0)
        .pan_speed(0.02)
        .zoom_speed(0.01)
        .build();

    let mut textures = vec!["hipparcossq.jpg".to_owned()];
    for command in &commands {
        scan_textures(command, &mut textures);
    }

    for texture in &mut textures {
        *texture = format!("assets/{}", texture);
    }

    let mut loaded = three_d_asset::io::load_async(&textures).await.unwrap();

    let top_tex = loaded.deserialize("hipparcossq").unwrap();
    let skybox = Skybox::new(
        &context, &top_tex, &top_tex, &top_tex, &top_tex, &top_tex, &top_tex,
    );

    let light = AmbientLight::new(&context, 0.1, Color::WHITE);
    let point = PointLight::new(
        &context,
        10.,
        Color::WHITE,
        &Vec3::zero(),
        Attenuation {
            constant: 0.,
            linear: 0.,
            quadratic: 0.,
        },
    );

    let mesh = uv_sphere(32);
    let mut body_context = BodyContext::new(&context, &mut loaded, &mesh);
    let mut bodies = load_astro_bodies(&commands, &mut body_context);

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
                frame_input.accumulated_time * 1e-3,
            );
        }

        fn get_render_models<'a, 'b>(
            body: &'a AstroBody,
        ) -> Vec<&'b dyn three_d::Object>
        where
            'a: 'b,
        {
            let mut models = vec![body.model.as_ref()];
            if let Some(ref cylinder) = body.orbit_model {
                models.push(cylinder as &dyn three_d::Object);
            }
            for body in body.children.iter() {
                models.extend(get_render_models(&body));
            }
            models
        }

        let mut render_models: Vec<&dyn three_d::Object> = vec![];
        for body in &bodies {
            render_models.extend(get_render_models(body));
        }

        frame_input
            .screen()
            .clear(ClearState::default())
            .render(&camera, &[&skybox], &[])
            .render(&camera, &render_models[..], &[&light, &point]);

        FrameOutput::default()
    });
}
