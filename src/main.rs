mod astro_body;
mod orbit_control_ex;
mod parser;
mod run;

use crate::{parser::commands, run::run};

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
    let s = std::fs::read_to_string("assets/sol.txt")?;
    let commands = {
        println!("source: {s:?}");
        let (_, commands) = commands(&s).map_err(|e| e.to_string())?;
        println!("commands: {commands:#?}");
        commands
    };
    run(commands).await;
    Ok(())
}
