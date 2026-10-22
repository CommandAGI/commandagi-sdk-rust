//! GIT ON A FOLDER: the folder door's operations (status, log, diff, commit, checkpoint, branch, switch, checkout, merge,
//! remotes, fetch, pull, push, fork), the same on a folder of this computer and on a folder of a cloud home.
//!
//! [`Call`] is one operation as the door takes it: its method, its operation and its JSON body. [`Host`] sends a call to
//! this computer's CommandAGI host (its MCP endpoint on the loopback, from `~/.commandagi/mcp.json`), with no dependency:
//! the git tools there (`git_status`, `git_commit` …) run real git in the repository the folder is in. A call that
//! changes a repository needs a principal credential (`COMMANDAGI_CREDENTIAL`) whose files grant covers the
//! repository's folder.
//!
//! A folder of a cloud home is reached over HTTPS, which this crate does not carry (it has no dependency, so a document's
//! `.rs` file compiles offline): [`Call::platform`] gives the request (method, path, body) to send with your own HTTP
//! client and a `cagi_` API key as its bearer, to `https://api.commandagi.com`.
//!
//! ```no_run
//! use commandagi::git::{self, Host};
//! let host = Host::from_env().unwrap();
//! let status = host.send(&git::status(), "/home/me/projects/bracket").unwrap();
//! println!("{}", status.text());
//! host.send(&git::commit("tighten the bracket").paths(["bracket.3d.tsx"]), "/home/me/projects/bracket").unwrap();
//! ```

use crate::design::Json;
use std::io::{Read, Write};
use std::net::TcpStream;

/// One operation of the folder door.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    /// "GET" (reads) or "POST" (changes).
    pub method: &'static str,
    /// The door's operation: `status`, `remote/add` …
    pub op: &'static str,
    /// The tool both MCP servers name it by.
    pub tool: &'static str,
    /// The arguments: a GET's query, a POST's body.
    pub args: Json,
}

impl Call {
    fn new(method: &'static str, op: &'static str, tool: &'static str) -> Call {
        Call { method, op, tool, args: Json::obj() }
    }
    fn arg(mut self, key: &str, value: impl Into<Json>) -> Call {
        self.args.set(key, value);
        self
    }
    /// Only these paths (a commit).
    pub fn paths<I: IntoIterator<Item = S>, S: Into<String>>(self, paths: I) -> Call {
        let list: Vec<Json> = paths.into_iter().map(|p| Json::Str(p.into())).collect();
        self.arg("paths", Json::Arr(list))
    }
    /// The remote (fetch, pull, push; default origin).
    pub fn remote(self, name: &str) -> Call {
        self.arg("remote", name)
    }
    /// The branch (log, pull, push).
    pub fn branch(self, name: &str) -> Call {
        self.arg("branch", name)
    }
    /// A remote's account: on this computer `<service>/<account>` of the host's secret store, in the cloud a connected account.
    pub fn account(self, name: &str) -> Call {
        self.arg("account", name)
    }
    /// One path (a diff).
    pub fn path(self, path: &str) -> Call {
        self.arg("path", path)
    }
    /// One commit (a diff).
    pub fn commit_id(self, oid: &str) -> Call {
        self.arg("commit", oid)
    }
    /// Replace the remote's branch even when it has commits this one lacks (push).
    pub fn force(self) -> Call {
        self.arg("force", true)
    }
    /// The platform's request for a folder of a cloud home: (method, path with its query, JSON body or none). `home` is
    /// `me` or an account id (`acct:user:<id>`); `folder` a path of that home.
    pub fn platform(&self, home: &str, folder: &str) -> (&'static str, String, Option<String>) {
        let mut query = format!("folder={}", encode(folder));
        if self.method == "GET" {
            if let Json::Obj(entries) = &self.args {
                for (k, v) in entries {
                    let value = v.as_str().map(str::to_string).unwrap_or_else(|| v.text());
                    query.push_str(&format!("&{}={}", encode(k), encode(&value)));
                }
            }
        }
        let path = format!("/accounts/{}/git/{}?{}", encode(home), self.op, query);
        (self.method, path, if self.method == "POST" { Some(self.args.text()) } else { None })
    }
}

fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => (b as char).to_string(),
            _ => format!("%{:02X}", b),
        })
        .collect()
}

pub fn status() -> Call {
    Call::new("GET", "status", "git_status")
}
pub fn log() -> Call {
    Call::new("GET", "log", "git_log")
}
pub fn diff() -> Call {
    Call::new("GET", "diff", "git_diff")
}
pub fn remotes() -> Call {
    Call::new("GET", "remotes", "git_remotes")
}
pub fn commit(message: &str) -> Call {
    Call::new("POST", "commit", "git_commit").arg("message", message)
}
pub fn checkpoint() -> Call {
    Call::new("POST", "checkpoint", "git_checkpoint")
}
/// A branch at the commit checked out, switched to.
pub fn branch(name: &str) -> Call {
    Call::new("POST", "branch", "git_create_branch").arg("name", name)
}
pub fn switch(name: &str) -> Call {
    Call::new("POST", "switch", "git_switch").arg("name", name)
}
/// A branch, or a commit as a detached HEAD.
pub fn checkout(rev: &str) -> Call {
    Call::new("POST", "checkout", "git_checkout").arg("rev", rev)
}
pub fn merge(from: &str) -> Call {
    Call::new("POST", "merge", "git_merge").arg("from", from)
}
pub fn remote_add(name: &str, url: &str) -> Call {
    Call::new("POST", "remote/add", "git_remote_add").arg("name", name).arg("url", url)
}
pub fn remote_remove(name: &str) -> Call {
    Call::new("POST", "remote/remove", "git_remote_remove").arg("name", name)
}
pub fn fetch() -> Call {
    Call::new("POST", "fetch", "git_fetch")
}
pub fn pull() -> Call {
    Call::new("POST", "pull", "git_pull")
}
pub fn push() -> Call {
    Call::new("POST", "push", "git_push")
}
/// A cloud folder's: a new repository in folder `to` of the same home whose remote origin is this one.
pub fn fork(to: &str) -> Call {
    Call::new("POST", "fork", "git_fork").arg("to", to)
}

/// This computer's host: its MCP endpoint on the loopback, the launch token, and a principal credential when there is one.
#[derive(Clone, Debug)]
pub struct Host {
    /// `127.0.0.1:<port>`
    pub addr: String,
    /// The path of the endpoint (`/mcp`).
    pub path: String,
    pub token: String,
    pub credential: Option<String>,
}

impl Host {
    /// From `~/.commandagi/mcp.json` (or `$COMMANDAGI_USER_DIR/mcp.json`), and `COMMANDAGI_CREDENTIAL`.
    pub fn from_env() -> Result<Host, String> {
        let dir = std::env::var("COMMANDAGI_USER_DIR")
            .ok()
            .or_else(|| std::env::var("HOME").ok().map(|h| format!("{h}/.commandagi")))
            .ok_or("no home folder: set COMMANDAGI_USER_DIR")?;
        let file = format!("{dir}/mcp.json");
        let text = std::fs::read_to_string(&file).map_err(|e| format!("{file}: {e} (is the CommandAGI host running?)"))?;
        let info = Json::parse(&text)?;
        let url = info.get("url").and_then(Json::as_str).ok_or(format!("{file} names no url"))?;
        let token = info.get("token").and_then(Json::as_str).ok_or(format!("{file} names no token"))?;
        let mut host = Host::at(url, token)?;
        host.credential = std::env::var("COMMANDAGI_CREDENTIAL").ok().filter(|c| !c.is_empty());
        Ok(host)
    }

    /// An endpoint `http://127.0.0.1:<port>/mcp` and its token.
    pub fn at(url: &str, token: &str) -> Result<Host, String> {
        let rest = url.strip_prefix("http://").ok_or(format!("{url}: the host's endpoint is http on the loopback"))?;
        let (addr, path) = match rest.find('/') {
            Some(i) => (&rest[..i], &rest[i..]),
            None => (rest, "/"),
        };
        Ok(Host { addr: addr.to_string(), path: path.to_string(), token: token.to_string(), credential: None })
    }

    /// Run `call` in the repository `folder` (an absolute path or an address of this host) is in. → the answer as JSON
    /// (a diff is a JSON string); a refusal is the door's words.
    pub fn send(&self, call: &Call, folder: &str) -> Result<Json, String> {
        let mut args = call.args.clone();
        args.set("folder", folder);
        let body = Json::obj()
            .with("jsonrpc", "2.0")
            .with("id", 1i64)
            .with("method", "tools/call")
            .with("params", Json::obj().with("name", call.tool).with("arguments", args))
            .text();
        let answer = Json::parse(&self.post(&body)?)?;
        if let Some(e) = answer.get("error") {
            return Err(e.get("message").and_then(Json::as_str).unwrap_or("the host refused the call").to_string());
        }
        let result = answer.get("result").ok_or("the host answered no result")?;
        let text = match result.get("content") {
            Some(Json::Arr(parts)) => parts.iter().filter_map(|p| p.get("text").and_then(Json::as_str)).collect::<Vec<_>>().join("\n"),
            _ => String::new(),
        };
        if result.get("isError") == Some(&Json::Bool(true)) {
            return Err(text);
        }
        Ok(Json::parse(&text).unwrap_or(Json::Str(text)))
    }

    fn post(&self, body: &str) -> Result<String, String> {
        let mut stream = TcpStream::connect(&self.addr).map_err(|e| format!("{}: {e} (is the CommandAGI host running?)", self.addr))?;
        let credential = self.credential.as_ref().map(|c| format!("x-commandagi-credential: {c}\r\n")).unwrap_or_default();
        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.path,
            self.addr,
            self.token,
            credential,
            body.len(),
            body
        );
        stream.write_all(request.as_bytes()).map_err(|e| e.to_string())?;
        let mut raw = Vec::new();
        stream.read_to_end(&mut raw).map_err(|e| e.to_string())?;
        let split = raw.windows(4).position(|w| w == b"\r\n\r\n").ok_or("the host's answer has no head")?;
        let head = String::from_utf8_lossy(&raw[..split]).to_string();
        let mut payload = raw[split + 4..].to_vec();
        if head.to_ascii_lowercase().contains("transfer-encoding: chunked") {
            payload = unchunk(&payload)?;
        }
        let status = head.split_whitespace().nth(1).unwrap_or("");
        let text = String::from_utf8(payload).map_err(|e| e.to_string())?;
        if status != "200" {
            return Err(format!("the host answered {status}: {text}"));
        }
        Ok(text)
    }
}

fn unchunk(mut data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    loop {
        let line = data.windows(2).position(|w| w == b"\r\n").ok_or("a chunk without its size")?;
        let size = usize::from_str_radix(String::from_utf8_lossy(&data[..line]).split(';').next().unwrap_or("").trim(), 16).map_err(|e| e.to_string())?;
        data = &data[line + 2..];
        if size == 0 {
            return Ok(out);
        }
        if data.len() < size + 2 {
            return Err("a chunk cut short".into());
        }
        out.extend_from_slice(&data[..size]);
        data = &data[size + 2..];
    }
}
