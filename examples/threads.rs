//! Start an agent on a goal, then steer it.
//!
//!     export COMMANDAGI_API_KEY=cagi_...
//!     cargo run --example threads
use commandagi::client::CommandAGI;

fn main() -> Result<(), commandagi::client::Error> {
    let cagi = CommandAGI::from_env()?; // reads COMMANDAGI_API_KEY

    let thread = cagi.threads().create("Design a bracket for a NEMA 17 motor")?;
    cagi.threads().send(&thread.id, "Make it 3 mm thick.")?;
    println!("{}", thread.id);
    Ok(())
}
