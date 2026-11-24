//! THE PLATFORM CLIENT: agent threads, and live sessions on a simulated world (or a computer, or a robot) that you
//! launch, drive and watch. The same calls as the TypeScript and Python clients, over the platform's one door
//! (`POST /mcp`, JSON-RPC), with no dependency.
//!
//! The crate has no dependency, so a document's `.rs` file compiles against it offline. The client sends its requests
//! with the `curl` program that every computer with a shell has (a request's key and body go to curl on its standard
//! input, never in its arguments, so they are not in a process list). To use another HTTP client, implement
//! [`Transport`] and start with [`CommandAGI::with_transport`].
//!
//! ```no_run
//! use commandagi::client::CommandAGI;
//!
//! let cagi = CommandAGI::from_env().unwrap(); // reads COMMANDAGI_API_KEY
//!
//! // Start an agent on a goal, then steer it.
//! let thread = cagi.threads().create("Design a bracket for a NEMA 17 motor").unwrap();
//! cagi.threads().send(&thread.id, "Make it 3 mm thick.").unwrap();
//! ```
//!
//! A world you drive, with no agent in it:
//!
//! ```no_run
//! # use commandagi::client::CommandAGI;
//! # let cagi = CommandAGI::from_env().unwrap();
//! let world = cagi.launch("simulation/arm-cell").unwrap(); // waits until the world declares its controls
//! world.sim().run().unwrap();                              // a launched simulation starts paused
//! world.sim().ik([0.08, -0.12, 0.86]).unwrap();            // the arm's tool point, in metres
//! std::thread::sleep(std::time::Duration::from_secs(2));   // an answer is not a state: let it move, then read
//! let seen = world.describe().unwrap();
//! std::fs::write("cell.jpg", world.observe().unwrap().bytes).unwrap(); // what its camera sees: a JPEG
//! world.stop().unwrap();                                   // (dropping the world stops it too)
//! ```

use crate::design::Json;
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const ENV_API_KEY: &str = "COMMANDAGI_API_KEY";
pub const ENV_BASE_URL: &str = "COMMANDAGI_BASE_URL";
pub const DEFAULT_BASE_URL: &str = "https://api.commandagi.com";

/// What a call that failed carries: the tool, the platform's words and, when it gave one, its detail.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    pub tool: String,
    pub message: String,
    pub detail: Option<Json>,
}

impl Error {
    fn new(tool: &str, message: impl Into<String>) -> Error {
        Error { tool: tool.to_string(), message: message.into(), detail: None }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

/// One HTTP request.
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub method: &'static str,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timeout: Option<Duration>,
}

/// One HTTP answer.
#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

/// How a request reaches the network.
pub trait Transport {
    fn send(&self, request: &Request) -> Result<Response, String>;
}

/// The default transport: the `curl` program (`CURL` names another one).
#[derive(Clone, Debug, Default)]
pub struct Curl {
    program: Option<String>,
}

impl Curl {
    pub fn new() -> Curl {
        Curl::default()
    }

    pub fn program(mut self, path: &str) -> Curl {
        self.program = Some(path.to_string());
        self
    }
}

/// A value in curl's config file syntax: quoted, with the escapes it reads.
fn quoted(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0b}' => out.push_str("\\v"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl Transport for Curl {
    fn send(&self, request: &Request) -> Result<Response, String> {
        let program = self.program.clone().or_else(|| std::env::var("CURL").ok()).unwrap_or_else(|| "curl".to_string());
        let mut config = String::from("silent\nshow-error\nlocation\n");
        if let Some(t) = request.timeout {
            config.push_str(&format!("max-time = {}\n", t.as_secs_f64().max(1.0)));
        }
        config.push_str(&format!("request = {}\nurl = {}\nwrite-out = {}\n", quoted(request.method), quoted(&request.url), quoted("\n%{http_code}")));
        for (name, value) in &request.headers {
            config.push_str(&format!("header = {}\n", quoted(&format!("{name}: {value}"))));
        }
        if let Some(body) = &request.body {
            // data-raw: a body that starts with "@" is text, never a file to read.
            config.push_str(&format!("data-raw = {}\n", quoted(body)));
        }
        // The config comes on standard input: the key is in no argument.
        let mut child = Command::new(&program)
            .args(["--config", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("could not start `{program}`: {e} (the client sends its requests with curl; install it, or give the client a Transport of your own)"))?;
        child.stdin.take().ok_or("curl has no standard input")?.write_all(config.as_bytes()).map_err(|e| e.to_string())?;
        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        if !out.status.success() {
            let said = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(if out.status.code() == Some(28) { "no answer in time".to_string() } else { said });
        }
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        let cut = text.rfind('\n').ok_or("curl wrote no status")?;
        let status = text[cut + 1..].trim().parse::<u16>().map_err(|e| format!("curl's status: {e}"))?;
        Ok(Response { status, body: text[..cut].to_string() })
    }
}

/// An image a tool answered with.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub mime: String,
    pub bytes: Vec<u8>,
}

/// A tool's answer: its text (parsed as JSON when it is JSON) and the images beside it.
#[derive(Clone, Debug, PartialEq)]
pub struct Reply {
    pub value: Json,
    pub images: Vec<Image>,
}

/// The client. One surface for developers and agents alike: an API key's scopes are the only difference.
pub struct CommandAGI {
    api_key: String,
    base_url: String,
    timeout: Duration,
    transport: Box<dyn Transport>,
    ids: std::cell::Cell<i64>,
}

impl CommandAGI {
    /// A client with the key in `COMMANDAGI_API_KEY`, the address in `COMMANDAGI_BASE_URL` (default
    /// `https://api.commandagi.com`).
    pub fn from_env() -> Result<CommandAGI, Error> {
        let key = std::env::var(ENV_API_KEY).ok().filter(|k| !k.is_empty()).ok_or_else(|| Error::new("", format!("set {ENV_API_KEY}, or pass the key to CommandAGI::new")))?;
        let mut client = CommandAGI::new(&key);
        if let Some(url) = std::env::var(ENV_BASE_URL).ok().filter(|u| !u.is_empty()) {
            client = client.base_url(&url);
        }
        Ok(client)
    }

    pub fn new(api_key: &str) -> CommandAGI {
        CommandAGI::with_transport(api_key, Curl::new())
    }

    pub fn with_transport(api_key: &str, transport: impl Transport + 'static) -> CommandAGI {
        CommandAGI {
            api_key: api_key.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(120),
            transport: Box::new(transport),
            ids: std::cell::Cell::new(0),
        }
    }

    pub fn base_url(mut self, url: &str) -> CommandAGI {
        self.base_url = url.trim_end_matches('/').to_string();
        self
    }

    /// How long a call waits for its answer (default 120 s). The platform may still finish an operation that timed out.
    pub fn timeout(mut self, timeout: Duration) -> CommandAGI {
        self.timeout = timeout;
        self
    }

    /// Run any platform tool by name and read its answer: the universal call.
    pub fn call(&self, tool: &str, args: Json) -> Result<Json, Error> {
        Ok(self.call_reply(tool, args)?.value)
    }

    /// Like [`call`](Self::call), with the images the tool answered beside its text.
    pub fn call_reply(&self, tool: &str, args: Json) -> Result<Reply, Error> {
        let id = self.ids.get() + 1;
        self.ids.set(id);
        let body = Json::obj()
            .with("jsonrpc", "2.0")
            .with("id", id)
            .with("method", "tools/call")
            .with("params", Json::obj().with("name", tool).with("arguments", args))
            .text();
        let request = Request {
            method: "POST",
            url: format!("{}/mcp", self.base_url),
            headers: vec![
                ("authorization".into(), format!("Bearer {}", self.api_key)),
                ("content-type".into(), "application/json".into()),
                ("mcp-protocol-version".into(), "2025-06-18".into()),
                ("user-agent".into(), format!("commandagi-rust/{}", env!("CARGO_PKG_VERSION"))),
            ],
            body: Some(body),
            timeout: Some(self.timeout),
        };
        let response = self.transport.send(&request).map_err(|e| Error::new(tool, format!("{tool}: {e}")))?;
        let answer = Json::parse(&response.body).map_err(|_| Error::new(tool, format!("{tool}: non-JSON response (HTTP {})", response.status)))?;
        if let Some(e) = answer.get("error") {
            let said = e.get("message").and_then(Json::as_str).unwrap_or("tool error");
            return Err(Error { tool: tool.into(), message: format!("{tool}: {said}"), detail: Some(e.clone()) });
        }
        let result = answer.get("result").cloned().unwrap_or(Json::Null);
        let mut texts = Vec::new();
        let mut images = Vec::new();
        if let Some(Json::Arr(parts)) = result.get("content") {
            for part in parts {
                match part.get("type").and_then(Json::as_str) {
                    Some("image") => {
                        let data = part.get("data").and_then(Json::as_str).unwrap_or("");
                        let bytes = base64_decode(data).map_err(|e| Error::new(tool, format!("{tool}: an image that is not base64 ({e})")))?;
                        images.push(Image { mime: part.get("mimeType").and_then(Json::as_str).unwrap_or("image/jpeg").to_string(), bytes });
                    }
                    _ => {
                        if let Some(t) = part.get("text").and_then(Json::as_str) {
                            texts.push(t.to_string());
                        }
                    }
                }
            }
        }
        let text = texts.join("\n");
        if result.get("isError") == Some(&Json::Bool(true)) {
            return Err(Error { tool: tool.into(), message: format!("{tool}: {text}"), detail: Some(Json::Str(text)) });
        }
        let value = Json::parse(&text).unwrap_or(Json::Str(text));
        Ok(Reply { value, images })
    }

    /// One GET of the platform's REST door, answered as JSON.
    fn get(&self, path: &str) -> Result<Json, Error> {
        let request = Request {
            method: "GET",
            url: format!("{}{}", self.base_url, path),
            headers: vec![("authorization".into(), format!("Bearer {}", self.api_key)), ("user-agent".into(), format!("commandagi-rust/{}", env!("CARGO_PKG_VERSION")))],
            body: None,
            timeout: Some(self.timeout),
        };
        let response = self.transport.send(&request).map_err(|e| Error::new(path, format!("{path}: {e}")))?;
        let answer = Json::parse(&response.body).map_err(|_| Error::new(path, format!("{path}: non-JSON response (HTTP {})", response.status)))?;
        if response.status >= 400 {
            let said = answer.get("error").and_then(|e| e.as_str().or_else(|| e.get("message").and_then(Json::as_str))).unwrap_or("refused").to_string();
            return Err(Error { tool: path.into(), message: format!("{path}: {said} (HTTP {})", response.status), detail: Some(answer) });
        }
        Ok(answer)
    }

    /// Agent threads: the unit of work.
    pub fn threads(&self) -> Threads<'_> {
        Threads { client: self }
    }

    /// Computers, cameras, robots and sims attached to a thread.
    pub fn embodiments(&self) -> Embodiments<'_> {
        Embodiments { client: self }
    }

    /// Open a live session on an embodiment that already exists: its frames, and typed control.
    pub fn session(&self, thread_id: &str, embodiment_id: &str) -> Session<'_> {
        Session { client: self, thread_id: thread_id.into(), embodiment_id: embodiment_id.into(), owns_thread: false, stopped: false }
    }

    /// Launch a world YOU drive (a computer, `computer/…`, or a simulation or robot, `simulation/…`) in a new agentless
    /// thread and return its live session, once the world has declared its controls (booted and connected; waits
    /// up to ten minutes). Dropping the session stops the world and releases the machine.
    pub fn launch(&self, snapshot_id: &str) -> Result<Session<'_>, Error> {
        let session = self.launch_without_waiting(snapshot_id)?;
        session.ready(Duration::from_secs(600), Duration::from_secs(3))?;
        Ok(session)
    }

    /// [`launch`](Self::launch), returning at once.
    pub fn launch_without_waiting(&self, snapshot_id: &str) -> Result<Session<'_>, Error> {
        let created = self.call("create_thread", Json::obj().with("agentless", true).with("snapshotId", snapshot_id).with("title", snapshot_id))?;
        let thread_id = created.get("threadId").and_then(Json::as_str).ok_or_else(|| Error::new("create_thread", "create_thread answered no threadId"))?.to_string();
        let embodiment = created.get("launched").and_then(|l| l.get("embodimentId")).and_then(Json::as_str).map(str::to_string);
        let mut session = Session { client: self, thread_id, embodiment_id: String::new(), owns_thread: true, stopped: false };
        match embodiment {
            Some(id) => {
                session.embodiment_id = id;
                Ok(session)
            }
            None => {
                let mut error = Error::new("create_thread", format!("launch of {snapshot_id} started no embodiment"));
                error.detail = Some(created);
                Err(error)
            }
        }
    }
}

/// A thread an agent works in.
#[derive(Clone, Debug, PartialEq)]
pub struct Thread {
    pub id: String,
    /// The platform's whole answer.
    pub raw: Json,
}

pub struct Threads<'a> {
    client: &'a CommandAGI,
}

impl Threads<'_> {
    pub fn list(&self) -> Result<Json, Error> {
        self.client.call("list_threads", Json::obj())
    }

    pub fn get(&self, thread_id: &str) -> Result<Json, Error> {
        self.client.call("get_thread", Json::obj().with("threadId", thread_id))
    }

    /// Start an agent on a goal.
    pub fn create(&self, intent: &str) -> Result<Thread, Error> {
        self.create_with(Json::obj().with("intent", intent))
    }

    /// Start a thread with every argument `create_thread` takes (`about`, `snapshotId`, `agentless` …).
    pub fn create_with(&self, args: Json) -> Result<Thread, Error> {
        let raw = self.client.call("create_thread", args)?;
        let id = raw.get("threadId").and_then(Json::as_str).ok_or_else(|| Error::new("create_thread", "create_thread answered no threadId"))?.to_string();
        Ok(Thread { id, raw })
    }

    /// Message a thread.
    pub fn send(&self, thread_id: &str, text: &str) -> Result<Json, Error> {
        self.client.call("send_message", Json::obj().with("threadId", thread_id).with("text", text))
    }

    /// A thread's event log.
    pub fn events(&self, thread_id: &str) -> Result<Json, Error> {
        self.client.call("thread_events", Json::obj().with("threadId", thread_id))
    }

    /// Stop a thread's run (the thread itself stays).
    pub fn kill(&self, thread_id: &str) -> Result<Json, Error> {
        self.client.call("kill_process", Json::obj().with("pid", format!("thread:{thread_id}")))
    }

    /// Delete a thread (a key of the manager scope).
    pub fn delete(&self, thread_id: &str) -> Result<Json, Error> {
        self.client.call("delete_thread", Json::obj().with("threadId", thread_id))
    }
}

pub struct Embodiments<'a> {
    client: &'a CommandAGI,
}

impl Embodiments<'_> {
    pub fn list(&self) -> Result<Json, Error> {
        self.client.call("list_embodiments", Json::obj())
    }

    /// The thread's observation plane: its channels and the latest frame per embodiment.
    pub fn observe(&self, thread_id: &str) -> Result<Reply, Error> {
        self.client.call_reply("observe", Json::obj().with("threadId", thread_id))
    }

    /// What an embodiment can be driven with right now: channels, actions, payload schemas.
    pub fn controls(&self, embodiment_id: &str, thread_id: &str) -> Result<Json, Error> {
        self.client.call("list_controls", Json::obj().with("embodimentId", embodiment_id).with("threadId", thread_id))
    }

    /// Drive an embodiment with one declared action.
    pub fn act(&self, embodiment_id: &str, action: &str, payload: Json, thread_id: &str) -> Result<Json, Error> {
        let args = Json::obj().with("embodimentId", embodiment_id).with("action", action).with("payload", payload).with("threadId", thread_id);
        self.client.call("act", args)
    }

    /// The live sim or robot world: robots, objects and poses the runtime reports.
    pub fn describe(&self, thread_id: &str) -> Result<Json, Error> {
        self.client.call("describe_world", Json::obj().with("threadId", thread_id))
    }
}

/// One frame of a camera channel: a JPEG or PNG.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub embodiment_id: String,
    pub channel_id: String,
    pub width: u32,
    pub height: u32,
    pub mime: String,
    pub bytes: Vec<u8>,
}

/// A live session on one embodiment: a computer, a robot or a simulated world.
///
/// Control is a call: every action is `act(action, payload)`, resolved by the thread against what the embodiment
/// declares. [`sim`](Self::sim) is the typed vocabulary of a simulated world over that one method.
pub struct Session<'a> {
    client: &'a CommandAGI,
    pub thread_id: String,
    pub embodiment_id: String,
    owns_thread: bool,
    stopped: bool,
}

fn nums(values: &[f64]) -> Json {
    Json::Arr(values.iter().map(|v| Json::Num(*v)).collect())
}

impl Session<'_> {
    /// Send one declared action to this embodiment. A refusal (undeclared, an invalid payload) is an error.
    pub fn act(&self, action: &str, payload: Json) -> Result<Json, Error> {
        self.client.embodiments().act(&self.embodiment_id, action, payload, &self.thread_id)
    }

    /// The controls this embodiment declares right now (empty until its runtime has connected).
    pub fn controls(&self) -> Result<Vec<Json>, Error> {
        match self.client.embodiments().controls(&self.embodiment_id, &self.thread_id)?.get("controls") {
            Some(Json::Arr(controls)) => Ok(controls.clone()),
            _ => Ok(Vec::new()),
        }
    }

    /// The live world a sim or robot runtime reports: robots, objects, poses.
    pub fn describe(&self) -> Result<Json, Error> {
        self.client.embodiments().describe(&self.thread_id)
    }

    /// Wait until the embodiment is connected and declares its controls (a booting machine does not).
    pub fn ready(&self, timeout: Duration, poll: Duration) -> Result<Vec<Json>, Error> {
        let deadline = Instant::now() + timeout;
        loop {
            let controls = self.controls()?;
            if !controls.is_empty() {
                return Ok(controls);
            }
            if Instant::now() > deadline {
                return Err(Error::new("list_controls", format!("{} declared no controls within {} s: is it running?", self.embodiment_id, timeout.as_secs())));
            }
            std::thread::sleep(poll);
        }
    }

    /// The typed vocabulary of a simulated world.
    pub fn sim(&self) -> Sim<'_> {
        Sim { session: self }
    }

    /// The frames the thread's observation carries for this embodiment: its latest picture (a simulated world's `cam-head`,
    /// a 320 x 240 JPEG; a computer's screen). For the others (`depth`) see [`observe_channel`](Self::observe_channel).
    pub fn frames(&self) -> Result<Vec<Frame>, Error> {
        let reply = self.client.embodiments().observe(&self.thread_id)?;
        let entries = match reply.value.get("frames") {
            Some(Json::Arr(entries)) => entries.clone(),
            _ => Vec::new(),
        };
        let mut frames = Vec::new();
        for entry in entries {
            if entry.get("embodimentId").and_then(Json::as_str) != Some(self.embodiment_id.as_str()) {
                continue;
            }
            let frame = entry.get("frame");
            // `frame.image` is the number (from 1) of the image part that shows it.
            let number = frame.and_then(|f| f.get("image")).and_then(Json::as_f64).unwrap_or(1.0) as usize;
            if let Some(image) = reply.images.get(number.saturating_sub(1)) {
                let size = |k: &str| frame.and_then(|f| f.get(k)).and_then(Json::as_f64).unwrap_or(0.0) as u32;
                frames.push(Frame {
                    embodiment_id: self.embodiment_id.clone(),
                    channel_id: entry.get("channelId").and_then(Json::as_str).unwrap_or("").to_string(),
                    width: size("width"),
                    height: size("height"),
                    mime: image.mime.clone(),
                    bytes: image.bytes.clone(),
                });
            }
        }
        Ok(frames)
    }

    /// The latest frame of one channel: `cam-head` (the JPEG), `depth` (the 16-bit PNG), or the id of another camera of
    /// the scene. (The thread's one sample route: the channel's newest frame, whatever it is.)
    pub fn observe_channel(&self, channel: &str) -> Result<Frame, Error> {
        let path = format!("/threads/{}/streams/sample?embodimentId={}&channel={}", escape(&self.thread_id), escape(&self.embodiment_id), escape(channel));
        let sample = self.client.get(&path)?;
        let url = sample.get("url").and_then(Json::as_str).ok_or_else(|| Error::new("sample", format!("{} has no frame on {channel} yet", self.embodiment_id)))?;
        let data = url.strip_prefix("data:").and_then(|r| r.split_once(";base64,")).ok_or_else(|| Error::new("sample", format!("the frame of {channel} is not inline data")))?;
        let bytes = base64_decode(data.1).map_err(|e| Error::new("sample", e))?;
        let (width, height) = image_size(&bytes);
        Ok(Frame { embodiment_id: self.embodiment_id.clone(), channel_id: channel.to_string(), width, height, mime: data.0.to_string(), bytes })
    }

    /// The latest picture: the camera's frame (`cam-head`), else the first channel that has one.
    pub fn observe(&self) -> Result<Frame, Error> {
        let mut frames = self.frames()?;
        match frames.iter().position(|f| f.channel_id == "cam-head") {
            Some(i) => Ok(frames.swap_remove(i)),
            None if frames.is_empty() => Err(Error::new("observe", format!("{} has no frame yet", self.embodiment_id))),
            None => Ok(frames.swap_remove(0)),
        }
    }

    /// The next frame whose image differs from `previous`: what you want right after acting. A world that shows no
    /// change gives none in `timeout`, and that is an error.
    pub fn observe_changed(&self, previous: &Frame, timeout: Duration) -> Result<Frame, Error> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(frame) = self.observe() {
                if frame.bytes != previous.bytes {
                    return Ok(frame);
                }
            }
            if Instant::now() > deadline {
                return Err(Error::new("observe", format!("no new frame from {} within {} s", self.embodiment_id, timeout.as_secs())));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    /// Stop the world's run and release its machine, if this session launched it.
    pub fn stop(mut self) -> Result<(), Error> {
        self.stop_once()
    }

    fn stop_once(&mut self) -> Result<(), Error> {
        if self.owns_thread && !self.stopped {
            self.stopped = true;
            self.client.threads().kill(&self.thread_id)?;
        }
        Ok(())
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        let _ = self.stop_once();
    }
}

/// A simulated world's actions: lifecycle, interaction, and the robots inside it.
pub struct Sim<'a> {
    session: &'a Session<'a>,
}

impl Sim<'_> {
    /// Start the world (a launched simulation starts paused).
    pub fn run(&self) -> Result<Json, Error> {
        self.session.act("run", Json::obj())
    }

    pub fn pause(&self) -> Result<Json, Error> {
        self.session.act("pause", Json::obj())
    }

    /// Advance `n` steps while paused.
    pub fn step(&self, n: u32) -> Result<Json, Error> {
        self.session.act("step", Json::obj().with("n", n))
    }

    /// Run at `x` times real time.
    pub fn speed(&self, x: f64) -> Result<Json, Error> {
        self.session.act("speed", Json::obj().with("x", x))
    }

    pub fn reset(&self) -> Result<Json, Error> {
        self.session.act("reset", Json::obj())
    }

    /// Move the arm's tool point to `target`, in metres.
    pub fn ik(&self, target: [f64; 3]) -> Result<Json, Error> {
        self.session.act("ik", Json::obj().with("target", nums(&target)))
    }

    /// [`ik`](Self::ik) for the robot `robot_id`, with the arguments of the action (`axis`, `along`, `tolerance`).
    pub fn ik_with(&self, robot_id: &str, target: [f64; 3], extra: Json) -> Result<Json, Error> {
        let mut payload = Json::obj().with("robotId", robot_id).with("target", nums(&target));
        if let Json::Obj(entries) = extra {
            for (k, v) in entries {
                payload.set(&k, v);
            }
        }
        self.session.act("ik", payload)
    }

    /// Set joints by name, in radians.
    pub fn pose(&self, joints: &[(&str, f64)]) -> Result<Json, Error> {
        let joints = Json::Obj(joints.iter().map(|(k, v)| (k.to_string(), Json::Num(*v))).collect());
        self.session.act("pose", Json::obj().with("joints", joints))
    }

    /// Close or open the gripper.
    pub fn grip(&self, close: bool) -> Result<Json, Error> {
        self.session.act("grip", Json::obj().with("close", close))
    }

    /// Stop the robot where it is.
    pub fn stop(&self) -> Result<Json, Error> {
        self.session.act("stop", Json::obj())
    }
}

fn escape(text: &str) -> String {
    let mut out = String::new();
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// The pixel size of a PNG or a JPEG (0 x 0 when it is neither).
fn image_size(b: &[u8]) -> (u32, u32) {
    let be32 = |i: usize| u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    if b.len() > 24 && b.starts_with(&[0x89, b'P', b'N', b'G']) {
        return (be32(16), be32(20));
    }
    if b.starts_with(&[0xff, 0xd8]) {
        let mut i = 2;
        while i + 9 < b.len() && b[i] == 0xff {
            let marker = b[i + 1];
            let length = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
            if (0xc0..=0xcf).contains(&marker) && !matches!(marker, 0xc4 | 0xc8 | 0xcc) {
                return (u16::from_be_bytes([b[i + 7], b[i + 8]]) as u32, u16::from_be_bytes([b[i + 5], b[i + 6]]) as u32);
            }
            i += 2 + length;
        }
    }
    (0, 0)
}

/// Standard base64 (RFC 4648), padding optional, line breaks skipped.
pub fn base64_decode(text: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let (mut bits, mut have) = (0u32, 0u32);
    for c in text.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\n' | b'\r' | b' ' => continue,
            _ => return Err(format!("byte {c} is not base64")),
        };
        bits = (bits << 6) | v as u32;
        have += 6;
        if have >= 8 {
            have -= 8;
            out.push((bits >> have) as u8);
            bits &= (1 << have) - 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::rc::Rc;

    /// A PNG header of 320 x 240, base64.
    fn base64_png_320x240() -> &'static str {
        "iVBORw0KGgoAAAANSUhEUgAAAUAAAADwEAAAAA"
    }

    #[test]
    fn base64_decodes_what_the_platform_sends() {
        assert_eq!(base64_decode("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64_decode("aGVsbG8").unwrap(), b"hello");
        assert_eq!(base64_decode("/9j/").unwrap(), vec![0xff, 0xd8, 0xff]);
        assert!(base64_decode("a$b").is_err());
    }

    #[test]
    fn curl_config_quotes_what_it_reads_as_escapes() {
        assert_eq!(quoted("a\\n\"b\"\n"), "\"a\\\\n\\\"b\\\"\\n\"");
    }

    /// A transport that answers from a script and keeps what it was asked.
    struct Script {
        seen: Rc<RefCell<Vec<Request>>>,
        answers: RefCell<Vec<String>>,
    }

    impl Transport for Script {
        fn send(&self, request: &Request) -> Result<Response, String> {
            self.seen.borrow_mut().push(request.clone());
            Ok(Response { status: 200, body: self.answers.borrow_mut().remove(0) })
        }
    }

    fn tool_answer(text: &str) -> String {
        Json::obj()
            .with("jsonrpc", "2.0")
            .with("id", 1i64)
            .with("result", Json::obj().with("content", Json::Arr(vec![Json::obj().with("type", "text").with("text", text)])))
            .text()
    }

    fn scripted(answers: Vec<String>) -> (CommandAGI, Rc<RefCell<Vec<Request>>>) {
        let seen = Rc::new(RefCell::new(Vec::new()));
        let client = CommandAGI::with_transport("cagi_test", Script { seen: seen.clone(), answers: RefCell::new(answers) });
        (client, seen)
    }

    #[test]
    fn threads_create_and_send_make_the_calls_the_other_sdks_make() {
        let (client, seen) = scripted(vec![tool_answer(r#"{"threadId":"t1"}"#), tool_answer(r#"{"ok":true}"#)]);
        let thread = client.threads().create("Design a bracket for a NEMA 17 motor").unwrap();
        assert_eq!(thread.id, "t1");
        client.threads().send(&thread.id, "Make it 3 mm thick.").unwrap();
        let seen = seen.borrow();
        assert_eq!(seen[0].url, "https://api.commandagi.com/mcp");
        assert!(seen[0].headers.contains(&("authorization".into(), "Bearer cagi_test".into())));
        let body = Json::parse(seen[0].body.as_ref().unwrap()).unwrap();
        assert_eq!(body.get("method").and_then(Json::as_str), Some("tools/call"));
        let params = body.get("params").unwrap();
        assert_eq!(params.get("name").and_then(Json::as_str), Some("create_thread"));
        assert_eq!(params.get("arguments").unwrap().get("intent").and_then(Json::as_str), Some("Design a bracket for a NEMA 17 motor"));
        let sent = Json::parse(seen[1].body.as_ref().unwrap()).unwrap();
        let arguments = sent.get("params").unwrap().get("arguments").unwrap();
        assert_eq!(sent.get("params").unwrap().get("name").and_then(Json::as_str), Some("send_message"));
        assert_eq!(arguments.get("threadId").and_then(Json::as_str), Some("t1"));
        assert_eq!(arguments.get("text").and_then(Json::as_str), Some("Make it 3 mm thick."));
    }

    #[test]
    fn a_refusal_is_an_error_that_names_the_tool() {
        let refused = Json::obj().with("jsonrpc", "2.0").with("id", 1i64).with("error", Json::obj().with("code", -32001i64).with("message", "Forbidden: delete_thread needs the manager scope")).text();
        let (client, _) = scripted(vec![refused]);
        let e = client.threads().delete("t1").unwrap_err();
        assert_eq!(e.tool, "delete_thread");
        assert!(e.message.contains("manager scope"));
        let failed = Json::obj()
            .with("result", Json::obj().with("isError", true).with("content", Json::Arr(vec![Json::obj().with("type", "text").with("text", "no such thread")])))
            .text();
        let (client, _) = scripted(vec![failed]);
        assert_eq!(client.threads().get("x").unwrap_err().message, "get_thread: no such thread");
    }

    #[test]
    fn a_launched_world_is_driven_watched_and_stopped_when_dropped() {
        let image = Json::obj().with("type", "image").with("mimeType", "image/jpeg").with("data", "/9j/");
        let observed = Json::obj()
            .with("result", Json::obj().with("content", Json::Arr(vec![
                Json::obj().with("type", "text").with("text", r#"{"frames":[{"embodimentId":"e1","channelId":"cam-head","frame":{"image":1,"width":320,"height":240}}]}"#),
                image,
            ])))
            .text();
        let (client, seen) = scripted(vec![
            tool_answer(r#"{"threadId":"t1","launched":{"embodimentId":"e1"}}"#),
            tool_answer(r#"{"controls":[{"channelId":"ctrl","actions":["run","ik"]}]}"#),
            tool_answer(r#"{"ok":true}"#),
            tool_answer(r#"{"ok":true}"#),
            observed,
            tool_answer(r#"{"ok":true}"#),
        ]);
        {
            let world = client.launch("simulation/arm-cell").unwrap();
            world.sim().run().unwrap();
            world.sim().ik([0.08, -0.12, 0.86]).unwrap();
            let frame = world.observe().unwrap();
            assert_eq!((frame.channel_id.as_str(), frame.width, frame.height, frame.bytes.as_slice()), ("cam-head", 320, 240, &[0xff, 0xd8, 0xff][..]));
        }
        let seen = seen.borrow();
        let call = |i: usize| {
            let body = Json::parse(seen[i].body.as_ref().unwrap()).unwrap();
            let params = body.get("params").unwrap().clone();
            (params.get("name").and_then(Json::as_str).unwrap().to_string(), params.get("arguments").unwrap().clone())
        };
        assert_eq!(call(0).1.get("agentless"), Some(&Json::Bool(true)));
        assert_eq!(call(0).1.get("snapshotId").and_then(Json::as_str), Some("simulation/arm-cell"));
        assert_eq!(call(1).0, "list_controls");
        let (tool, ik) = call(3);
        assert_eq!(tool, "act");
        assert_eq!(ik.get("action").and_then(Json::as_str), Some("ik"));
        assert_eq!(ik.get("payload").unwrap().text(), r#"{"target":[0.08,-0.12,0.86]}"#);
        assert_eq!(ik.get("embodimentId").and_then(Json::as_str), Some("e1"));
        let (tool, kill) = call(5);
        assert_eq!(tool, "kill_process");
        assert_eq!(kill.get("pid").and_then(Json::as_str), Some("thread:t1"));
    }

    #[test]
    fn a_camera_and_its_depth_are_two_channels_of_one_observation() {
        let part = |mime: &str, data: &str| Json::obj().with("type", "image").with("mimeType", mime).with("data", data);
        let text = r#"{"frames":[
            {"embodimentId":"other","channelId":"cam-head","frame":{"image":1}},
            {"embodimentId":"e1","channelId":"depth","frame":{"image":2,"width":320,"height":240}},
            {"embodimentId":"e1","channelId":"cam-head","frame":{"image":3,"width":320,"height":240}}]}"#;
        let observed = Json::obj()
            .with("result", Json::obj().with("content", Json::Arr(vec![Json::obj().with("type", "text").with("text", text), part("image/jpeg", "AAAA"), part("image/png", "iVBO"), part("image/jpeg", "/9j/")])))
            .text();
        let png = format!("{{\"url\":\"data:image/png;base64,{}\",\"kind\":\"depth\"}}", base64_png_320x240());
        let (client, seen) = scripted(vec![observed.clone(), png, observed]);
        let world = client.session("t1", "e1");
        let frames = world.frames().unwrap();
        assert_eq!(frames.iter().map(|f| f.channel_id.as_str()).collect::<Vec<_>>(), ["depth", "cam-head"]);
        let depth = world.observe_channel("depth").unwrap();
        assert_eq!((depth.mime.as_str(), depth.width, depth.height), ("image/png", 320, 240));
        assert_eq!(seen.borrow()[1].url, "https://api.commandagi.com/threads/t1/streams/sample?embodimentId=e1&channel=depth");
        // observe is the camera, not the first frame listed, and not another embodiment's.
        assert_eq!(world.observe().unwrap().bytes, vec![0xff, 0xd8, 0xff]);
    }

    #[test]
    fn a_world_with_no_embodiment_is_refused() {
        let (client, _) = scripted(vec![tool_answer(r#"{"threadId":"t1","launched":{"status":"denied"}}"#), tool_answer("{}")]);
        // The thread it made is stopped, not left running.
        let e = client.launch("simulation/nowhere").err().unwrap();
        assert!(e.message.contains("started no embodiment"));
    }

    /// One real request through the real curl, to a server on the loopback: the key and body arrive whole (a body with
    /// quotes, backslashes, newlines and non-ASCII text, and one that starts with `@`), and neither is in curl's arguments.
    #[test]
    fn curl_carries_the_request_whole() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut raw = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                let n = stream.read(&mut buf).unwrap();
                raw.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&raw).to_string();
                if let Some(split) = text.find("\r\n\r\n") {
                    let length = text.to_ascii_lowercase().split("content-length: ").nth(1).and_then(|r| r.split("\r\n").next()).and_then(|n| n.parse::<usize>().ok()).unwrap_or(0);
                    if raw.len() >= split + 4 + length {
                        break;
                    }
                }
            }
            stream.write_all(b"HTTP/1.1 201 Created\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok").unwrap();
            String::from_utf8(raw).unwrap()
        });
        let body = "@\"quoted\" \\n\\ line\nbreak é ✓ 100%";
        let request = Request {
            method: "POST",
            url: format!("http://127.0.0.1:{port}/mcp"),
            headers: vec![("authorization".into(), "Bearer cagi_secret".into()), ("content-type".into(), "application/json".into())],
            body: Some(body.to_string()),
            timeout: Some(Duration::from_secs(20)),
        };
        let response = Curl::new().send(&request).unwrap();
        assert_eq!((response.status, response.body.as_str()), (201, "ok"));
        let raw = server.join().unwrap();
        assert!(raw.starts_with("POST /mcp HTTP/1.1"));
        assert!(raw.to_ascii_lowercase().contains("authorization: bearer cagi_secret\r\n"));
        assert!(raw.ends_with(body), "the body arrived as {raw:?}");
    }

    #[test]
    fn a_missing_curl_says_how_to_go_on() {
        let request = Request { method: "GET", url: "http://127.0.0.1:9/".into(), headers: vec![], body: None, timeout: None };
        let e = Curl::new().program("/nonexistent/curl").send(&request).unwrap_err();
        assert!(e.contains("Transport of your own"));
    }
}
