//! Launch a simulated world, look at what it accepts, drive it, and save what it sees.
//!
//!     export COMMANDAGI_API_KEY=cagi_...
//!     cargo run --example drive_a_sim
use commandagi::client::CommandAGI;
use commandagi::design::Json;
use std::{thread::sleep, time::Duration};

fn main() -> Result<(), commandagi::client::Error> {
    let cagi = CommandAGI::from_env()?;

    // A world YOU drive (no agent in it). Dropping `world` stops it and releases the machine.
    let world = cagi.launch("simulation/arm-cell")?;
    for channel in world.controls()? {
        println!("{}", channel.get("channelId").and_then(Json::as_str).unwrap_or("?"));
    }

    world.sim().run()?; // a launched simulation starts paused
    world.sim().ik([0.08, -0.12, 0.86])?; // the arm's tool point, in metres
    sleep(Duration::from_secs(2)); // an answer is not a state: let it move, then read

    println!("{}", world.describe()?.text().len());
    std::fs::write("cell.jpg", world.observe()?.bytes).expect("write cell.jpg");
    println!("saved cell.jpg");
    // The depth channel: a 16-bit PNG, millimetres, 0 where nothing is.
    std::fs::write("depth.png", world.observe_channel("depth")?.bytes).expect("write depth.png");
    println!("saved depth.png");
    world.stop()
}
