use three_d::*;
use three_d_asset::prelude::*;

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "Texture!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(4.0, 1.5, 4.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let mut loaded = three_d_asset::io::load_async(&[
        "skybox_evening/right.jpg",
        "skybox_evening/left.jpg",
        "skybox_evening/top.jpg",
        "skybox_evening/front.jpg",
        "skybox_evening/back.jpg",
        // "Skybox_example.png",
        // "PenguinBaseMesh.obj",
    ])
    .await
    .unwrap();

    // Skybox
    let top_tex = loaded.deserialize("top").unwrap();
    let skybox = Skybox::new(
        &context,
        &loaded.deserialize("right").unwrap(),
        &loaded.deserialize("left").unwrap(),
        &top_tex,
        &top_tex,
        &loaded.deserialize("front").unwrap(),
        &loaded.deserialize("back").unwrap(),
    );

    // Box
    // let mut cpu_texture: CpuTexture = loaded.deserialize("Skybox_example").unwrap();
    // cpu_texture.data.to_linear_srgb();
    // let mut box_object = Gm::new(
    //     Mesh::new(&context, &CpuMesh::cube()),
    //     ColorMaterial {
    //         texture: Some(Texture2DRef::from_cpu_texture(&context, &cpu_texture)),
    //         ..Default::default()
    //     },
    // );
    // box_object.material.render_states.cull = Cull::Back;

    // Penguin
    // let model = loaded.deserialize("PenguinBaseMesh.obj").unwrap();
    // let mut penguin = Model::<PhysicalMaterial>::new(&context, &model).unwrap();
    // penguin.iter_mut().for_each(|m| {
    //     m.set_transformation(Mat4::from_translation(vec3(0.0, 1.0, 0.5)));
    //     m.material.render_states.cull = Cull::Back;
    // });

    // Lights
    let ambient = AmbientLight::new(&context, 0.4, Srgba::WHITE);
    let directional = DirectionalLight::new(
        &context,
        2.0,
        Srgba::WHITE,
        &vec3(0.0, -1.0, -1.0),
    );

    // main loop
    window.render_loop(move |mut frame_input| {
        let mut redraw = frame_input.first_frame;
        redraw |= camera.set_viewport(frame_input.viewport);
        redraw |= control.handle_events(&mut camera, &mut frame_input.events);

        // draw
        if redraw {
            frame_input.screen().clear(ClearState::default()).render(
                &camera,
                [].into_iter().chain(&skybox),
                &[&ambient, &directional],
            );
        }

        FrameOutput {
            swap_buffers: redraw,
            ..Default::default()
        }
    });
}
