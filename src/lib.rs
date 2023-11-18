mod astro_body;
mod orbit_control_ex;
mod parser;
mod run;
// mod web_main;

// Entry point for wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).unwrap();

    use crate::parser::commands;
    use log::info;
    info!("Logging works!");

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    fn map_err(e: impl std::error::Error) -> JsValue {
        JsValue::from_str(&e.to_string())
    }

    let assets = three_d_asset::io::load_async(&["assets/sol.txt"])
        .await
        .map_err(map_err)?;
    let commands = {
        let s =
            std::str::from_utf8(assets.get("").unwrap()).map_err(map_err)?;
        println!("source: {s:?}");
        let (_, commands) = commands(&s).map_err(map_err)?;
        println!("commands: {commands:#?}");
        commands
    };
    run::run(commands).await;
    Ok(())
}
