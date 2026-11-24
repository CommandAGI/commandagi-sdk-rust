//! The README's first example, as it runs: `cargo run --example readme` (a key in COMMANDAGI_API_KEY).
use commandagi::client::CommandAGI;
use std::{thread::sleep, time::Duration};

fn main() -> Result<(), commandagi::client::Error> {
    let cagi = CommandAGI::from_env()?; // reads COMMANDAGI_API_KEY

    // Agents: start one on a goal, then steer it.
    let thread = cagi.threads().create("Design a bracket for a NEMA 17 motor")?;
    cagi.threads().send(&thread.id, "Make it 3 mm thick.")?;

    // Embodiments: launch a world YOU drive (no agent in it), watch it, control it.
    let world = cagi.launch("simulation/arm-cell")?; // waits until it declares its controls
    world.sim().run()?;                              // a launched simulation starts paused
    world.sim().ik([0.08, -0.12, 0.86])?;            // the arm's tool point, in metres
    sleep(Duration::from_secs(2));                   // an answer is not a state: let it move, then read
    println!("{}", world.describe()?.text().len());  // where everything is now
    std::fs::write("cell.jpg", world.observe()?.bytes).unwrap(); // what its camera sees: a 320 x 240 JPEG
    world.stop()?;                                   // (dropping the world stops it too)
    Ok(())
}
