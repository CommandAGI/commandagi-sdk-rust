//! `cargo run --example git -- <folder> <op> [arg]`: one git operation on a folder of this computer, through the running
//! CommandAGI host (`commandagi::git::Host`). Prints the answer as JSON; a refusal goes to standard error, exit 1.
//!
//!   status | log | remotes | diff [path] | commit <message> | checkpoint | branch <name> | switch <name>
//!   checkout <rev> | merge <branch> | remote-add <name>=<url> | fetch | pull | push

use commandagi::git::{self, Call, Host};

fn call(op: &str, arg: Option<&str>) -> Result<Call, String> {
    let need = || arg.ok_or(format!("{op} needs an argument"));
    Ok(match op {
        "status" => git::status(),
        "log" => git::log(),
        "remotes" => git::remotes(),
        "diff" => match arg {
            Some(p) => git::diff().path(p),
            None => git::diff(),
        },
        "commit" => git::commit(need()?),
        "checkpoint" => git::checkpoint(),
        "branch" => git::branch(need()?),
        "switch" => git::switch(need()?),
        "checkout" => git::checkout(need()?),
        "merge" => git::merge(need()?),
        "remote-add" => {
            let (name, url) = need()?.split_once('=').ok_or("remote-add <name>=<url>")?;
            git::remote_add(name, url)
        }
        "fetch" => git::fetch(),
        "pull" => git::pull(),
        "push" => git::push(),
        _ => return Err(format!("no git operation {op}")),
    })
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let run = || -> Result<String, String> {
        let folder = args.first().ok_or("usage: git <folder> <op> [arg]")?;
        let op = args.get(1).ok_or("usage: git <folder> <op> [arg]")?;
        let answer = Host::from_env()?.send(&call(op, args.get(2).map(String::as_str))?, folder)?;
        Ok(answer.text())
    };
    match run() {
        Ok(text) => println!("{text}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
