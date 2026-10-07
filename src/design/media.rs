//! A VIDEO OR A SONG IN RUST — the media editors' own documents, declared as the nodes the CommandAGI video editor
//! (`.vid.rs`: `video.source`, `video.clip`, `video.track`, `video.composite`) and music studio (`.mus.rs`: `midiClip`,
//! `instrument`, `fx.*`, `track`, `master`) store and edit. The same nodes, node for node, as the TypeScript SDK's
//! `media.ts` and the Python SDK's `commandagi.design.media`:
//!
//! ```
//! use commandagi::design::media::*;
//!
//! fn document() -> El {
//!     video([
//!         track([
//!             clip([]).src("media/sky.png").start(0).duration(3),
//!             title([]).text("Hello").start(0).duration(2).font_size(72),
//!         ])
//!         .name("V1"),
//!         track([clip([]).src("media/tone.wav").start(0).in_(0.5).out(2.5).volume(0.8)]).name("A1").kind("audio"),
//!     ])
//!     .name("Balcony")
//!     .width(1280)
//!     .height(720)
//!     .fps(30)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! ```
//! use commandagi::design::media::*;
//!
//! fn document() -> El {
//!     song([
//!         track([
//!             synth().wave("triangle"),
//!             clip([note().pitch("C4").start(0).duration(1).velocity(0.8)]).name("Keys 1").start(0).length(4),
//!         ])
//!         .name("Keys"),
//!         track([clip([]).src("media/tone.wav").start(4).length(2).in_(0.25).volume(0.8)]).name("Tone"),
//!         track([sampler().src("media/tone.wav").root("A4"), clip([]).name("Bells 1").start(0).length(4)]).name("Bells"),
//!     ])
//!     .name("Loop")
//!     .tempo(120)
//!     .time_signature("4/4")
//!     .bars(8)
//!     .cycle_start(0)
//!     .cycle_end(16)
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! Attributes are snake case of the TypeScript names (`.font_size(…)` is `fontSize`; `.in_(…)` is `in`). An effect's
//! parameters are its attributes (`effect([]).type_("vignette").amount(0.3)`). A tempo map and an effect's colours are
//! JSON text: `.tempo(json(r#"[{"atBeat": 0, "bpm": 120}]"#))`. Media files are named by path relative to the file,
//! never inlined. Times on a video's timeline are seconds; in a song, beats (an audio clip's `in`, where it starts in its
//! file, is seconds, as on a video). Each node carries the call that declared
//! it in `meta.source`; what a node holds that is not a node (a clip's effects, transition, intro, outro and
//! keyframes; a midi clip's notes; a video's markers) carries its call in `meta.sources`, by key. Anything the
//! vocabulary cannot say is refused by name.

use super::element::{attributes, elements, fn_name, rust_name};
use super::ir::{slug, wire};
use super::read::tag;
use super::source::Sources;
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A video: its tracks and markers.
    video: children;
    /// A song: its tracks.
    song: children;
    /// A track: a video's clips, or a song's synth, effects and clips.
    track: children;
    /// A clip: of a media file on a video track (its transition, intro, outro, effects and keyframes), or of notes in a song.
    clip: children;
    /// A title on a video track: its keyframes and effects.
    title: children;
    /// A sticker on a video track.
    shape: children;
    /// An adjustment layer on a video track.
    adjustment: children;
    /// A midi clip on a video's midi track: its notes.
    midi: children;
    /// A clip's effect: its keyframes.
    effect: children;
    /// A video's marker.
    marker: leaf;
    /// A clip's transition into it.
    transition: leaf;
    /// A clip's intro animation.
    intro: leaf;
    /// A clip's outro animation.
    outro: leaf;
    /// A keyframe of a clip's property or an effect's param.
    keyframe: leaf;
    /// A note: `note().pitch("C4").start(0).duration(1)`.
    note: leaf;
    /// A song track's synth.
    synth: leaf;
    /// A song track's sampler: a sound file played at the pitch of each note.
    sampler: leaf;
    /// A song track's gain effect.
    gain: leaf;
    /// A song track's filter effect.
    filter: leaf;
    /// A song track's delay effect.
    delay: leaf;
    /// A song track's reverb effect.
    reverb: leaf;
    /// A song track's equaliser; a video's three-band EQ (a clip's, a track's or the master's).
    eq: leaf;
    /// A video's compressor (a clip's, a track's or the master's audio effect).
    compressor: leaf;
    /// A video's distortion (a clip's, a track's or the master's audio effect).
    distortion: leaf;
}

attributes! {
    /// The media documents' attributes, chained on their elements.
    pub trait MediaAttrs {
        /// A name.
        name;
        /// A video's width in pixels; an effect's width.
        width;
        /// A video's height in pixels; a track's height; an effect's height.
        height;
        /// A video's frames per second.
        fps;
        /// A video's sample rate.
        sample_rate;
        /// A video's background.
        background;
        /// A track's kind ("video", "audio", "midi"), a shape's kind, a transition's kind.
        kind;
        /// A track is muted.
        muted;
        /// A track is hidden.
        hidden;
        /// A track is locked.
        locked;
        /// A track's or a clip's volume.
        volume;
        /// A clip's media file, by its path relative to this file.
        src;
        /// Where a clip or a note starts.
        start;
        /// How long a clip, a note, a transition or an animation lasts.
        duration;
        /// Where a clip starts in its media (`in`).
        in_;
        /// Where a clip ends in its media.
        out;
        /// A clip's speed; an effect's speed.
        speed;
        /// A clip's opacity.
        opacity;
        /// A clip's blend mode.
        blend_mode;
        /// How a clip fits the frame: "fit", "fill", "stretch".
        fit_mode;
        /// A clip's place across.
        x;
        /// A clip's place down.
        y;
        /// A clip's scale across.
        scale_x;
        /// A clip's scale down.
        scale_y;
        /// A clip's rotation.
        rotation;
        /// A clip's anchor across.
        anchor_x;
        /// A clip's anchor down.
        anchor_y;
        /// A clip's exposure.
        exposure;
        /// A clip's contrast; an effect's contrast.
        contrast;
        /// A clip's saturation.
        saturation;
        /// A clip's temperature.
        temperature;
        /// A clip's brightness; an effect's brightness.
        brightness;
        /// A clip's hue.
        hue;
        /// The fraction of a clip's picture hidden at its top.
        crop_top;
        /// The fraction of a clip's picture hidden at its right.
        crop_right;
        /// The fraction of a clip's picture hidden at its bottom.
        crop_bottom;
        /// The fraction of a clip's picture hidden at its left.
        crop_left;
        /// A bezier keyframe's curve: `[x1, y1, x2, y2]`, as CSS's cubic-bezier.
        bezier;
        /// A compressor's ratio.
        ratio;
        /// A title's text.
        text;
        /// A title's font family.
        font_family;
        /// A title's font size.
        font_size;
        /// A title's colour; a marker's colour.
        color;
        /// A title is bold.
        bold;
        /// A title is italic.
        italic;
        /// A title's alignment.
        align;
        /// A title's stroke colour.
        stroke_color;
        /// A title's or a shape's stroke width.
        stroke_width;
        /// A title types itself out over this many seconds.
        typewriter;
        /// A shape's content: an emoji's glyph or SVG path data.
        content;
        /// A shape's fill.
        fill;
        /// A shape's stroke.
        stroke;
        /// A midi clip's instrument.
        instrument;
        /// A midi clip's gain; a synth's or an effect's gain.
        gain;
        /// A keyframe's or a marker's time.
        time;
        /// An intro's or an outro's preset.
        preset;
        /// An effect's type.
        type_;
        /// An effect is on.
        enabled;
        /// An effect's colours: `json(r##"{"key": "#00ff00"}"##)`.
        colors;
        /// The effect param a keyframe animates.
        param;
        /// The clip property a keyframe animates.
        property;
        /// A keyframe's value.
        value;
        /// A keyframe's easing.
        easing;
        /// A note's pitch: a MIDI number or a name ("C4").
        pitch;
        /// A note's velocity, 0 to 1.
        velocity;
        /// A song's tempo: beats per minute, or a tempo map.
        tempo;
        /// A song's time signature: "4/4".
        time_signature;
        /// A song's bars.
        bars;
        /// Where a song's cycle starts, in beats.
        cycle_start;
        /// Where a song's cycle ends, in beats.
        cycle_end;
        /// The pitch a sampler's sound file sounds at.
        root;
        /// A song's or a track's volume in dB.
        volume_db;
        /// A song track's colour.
        color_index;
        /// A song track's pan, -1 to 1.
        pan;
        /// A song track is muted.
        mute;
        /// A song track is soloed.
        solo;
        /// A synth's wave.
        wave;
        /// A synth's detune.
        detune;
        /// A synth's voices.
        voices;
        /// A synth's transpose.
        transpose;
        /// A synth's attack.
        attack;
        /// A synth's decay; a reverb's decay.
        decay;
        /// A synth's sustain.
        sustain;
        /// A synth's release.
        release;
        /// A filter's mode.
        mode;
        /// A filter's frequency; an effect's frequency.
        freq;
        /// A filter's Q.
        q;
        /// A delay's time in beats.
        time_beats;
        /// A delay's feedback.
        feedback;
        /// An effect's mix.
        mix;
        /// An equaliser's low gain.
        low_gain;
        /// An equaliser's mid gain.
        mid_gain;
        /// An equaliser's high gain.
        high_gain;
        /// A song clip's length in beats.
        length;
        /// A song clip loops.
        loop_;
        /// An effect's amount.
        amount;
        /// An effect's amplitude.
        amp;
        /// An effect's angle.
        angle;
        /// An effect's axis.
        axis;
        /// An effect's centre across.
        cx;
        /// An effect's centre down.
        cy;
        /// An effect's degrees.
        degrees;
        /// An effect's feather.
        feather;
        /// An effect's gamma.
        gamma;
        /// An effect inverts.
        invert;
        /// An effect's levels.
        levels;
        /// An effect's lift.
        lift;
        /// An effect's radius.
        radius;
        /// An effect's shape.
        shape;
        /// An effect's similarity.
        similarity;
        /// An effect's size.
        size;
        /// An effect's smoothness.
        smoothness;
        /// An effect's spill.
        spill;
        /// An effect's strength.
        strength;
        /// An effect's threshold.
        threshold;
    }
}

pub(crate) const FAMILY: Family = Family { module: "media", roots: &["video", "song"], declare };

fn declare(root: El) -> Result<Declared, String> {
    Ok(Declared::Graph(if root.tag == "video" { declare_video(&root)? } else { declare_song(&root)? }))
}

/// How a message names a media element: its call, with its name, its file or its text.
fn place(el: &El) -> String {
    match ["name", "src", "text"].iter().find_map(|k| el.attr(k).map(|v| (k, v))) {
        Some((k, v @ Json::Str(_))) => format!("{}().{}({})", fn_name(el.tag), rust_name(k), v.text()),
        _ => format!("{}()", fn_name(el.tag)),
    }
}

fn refuse_unknown(el: &El, allowed: &[&str]) -> Result<(), String> {
    match el.attrs.iter().find(|(k, _)| !allowed.contains(&k.as_str())) {
        Some((k, _)) => Err(format!("{}: {} is not read on a {}", place(el), rust_name(k), tag(el.tag))),
        None => Ok(()),
    }
}

fn number_text(n: f64) -> String {
    Json::Num(n).text()
}

/// A finite number attribute within its bounds, or none.
fn num(el: &El, prop: &str, min: Option<f64>, max: Option<f64>) -> Result<Option<f64>, String> {
    let v = match el.attr(prop) {
        None => return Ok(None),
        Some(Json::Num(n)) if n.is_finite() => *n,
        Some(v) => return Err(format!("{}: {} is a number, not {}", place(el), rust_name(prop), v.text())),
    };
    if let Some(lo) = min.filter(|lo| v < *lo) {
        return Err(format!("{}: {} is at least {}, not {}", place(el), rust_name(prop), number_text(lo), number_text(v)));
    }
    if let Some(hi) = max.filter(|hi| v > *hi) {
        return Err(format!("{}: {} is at most {}, not {}", place(el), rust_name(prop), number_text(hi), number_text(v)));
    }
    Ok(Some(v))
}

fn any(el: &El, prop: &str) -> Result<Option<f64>, String> {
    num(el, prop, None, None)
}

fn at_least(el: &El, prop: &str, min: f64) -> Result<Option<f64>, String> {
    num(el, prop, Some(min), None)
}

/// A text attribute (one of `one_of` when given), or none.
fn txt(el: &El, prop: &str, one_of: Option<&[&str]>) -> Result<Option<String>, String> {
    let v = match el.attr(prop) {
        None => return Ok(None),
        Some(Json::Str(s)) => s.clone(),
        Some(v) => return Err(format!("{}: {} is text, not {}", place(el), rust_name(prop), v.text())),
    };
    if let Some(values) = one_of.filter(|values| !values.contains(&v.as_str())) {
        return Err(format!("{}: {} is one of {}, not \"{v}\"", place(el), rust_name(prop), values.join(", ")));
    }
    Ok(Some(v))
}

fn flag(el: &El, prop: &str) -> Result<Option<bool>, String> {
    match el.attr(prop) {
        None => Ok(None),
        Some(Json::Bool(b)) => Ok(Some(*b)),
        Some(v) => Err(format!("{}: {} is true or false, not {}", place(el), rust_name(prop), v.text())),
    }
}

/// Ids in declaration order, numbered on collision: the same file declares the same ids.
#[derive(Default)]
struct Ids(Vec<String>);

impl Ids {
    fn make(&mut self, base: &str) -> String {
        let b = slug(base);
        let mut id = b.clone();
        let mut n = 2;
        while self.0.contains(&id) {
            id = format!("{b}_{n}");
            n += 1;
        }
        self.0.push(id.clone());
        id
    }
}

/// A node's meta: `extra`, the element it came from, and the elements of what it holds, by key.
fn meta_of(el: &El, sources: Sources, extra: Json) -> Option<Json> {
    let mut meta = extra;
    if let Some(s) = el.source_json() {
        meta.set("source", s);
    }
    if !sources.is_empty() {
        meta.set("sources", sources.json());
    }
    match &meta {
        Json::Obj(e) if e.is_empty() => None,
        _ => Some(meta),
    }
}

fn node(id: &str, kind: &str, inputs: Json, label: Option<&str>, meta: Option<Json>) -> Json {
    let mut n = Json::obj().with("id", id).with("type", kind);
    if let Some(label) = label {
        n.set("label", label);
    }
    n.set("inputs", inputs);
    if let Some(meta) = meta {
        n.set("meta", meta);
    }
    n
}

/// Nodes by id, in the order they were declared.
#[derive(Default)]
struct Nodes(Vec<(String, Json)>);

impl Nodes {
    fn has(&self, id: &str) -> bool {
        self.0.iter().any(|(k, _)| k == id)
    }
    fn put(&mut self, id: &str, n: Json) {
        match self.0.iter_mut().find(|(k, _)| k == id) {
            Some(entry) => entry.1 = n,
            None => self.0.push((id.to_string(), n)),
        }
    }
    fn get_mut(&mut self, id: &str) -> Option<&mut Json> {
        self.0.iter_mut().find(|(k, _)| k == id).map(|(_, v)| v)
    }
}

fn obj(entries: &[(&str, Json)]) -> Json {
    Json::Obj(entries.iter().map(|(k, v)| (k.to_string(), v.clone())).collect())
}

// ── Video ────────────────────────────────────────────────────────────────────────────────────────────────────────

/// What a media file is, by its extension: "video", "audio" or "image".
pub fn media_kind(src: &str) -> Option<&'static str> {
    let (_, ext) = src.rsplit_once('.')?;
    if ext.is_empty() || !ext.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return None;
    }
    Some(match ext.to_lowercase().as_str() {
        "mp4" | "webm" | "mov" | "mkv" | "m4v" | "ogv" => "video",
        "wav" | "mp3" | "ogg" | "oga" | "m4a" | "flac" | "aac" | "opus" => "audio",
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "avif" | "bmp" | "svg" => "image",
        _ => return None,
    })
}

pub const VIDEO_TRACK_KINDS: &[&str] = &["video", "audio", "midi"];
/// A `shape` clip's shape (the video editor's stickers).
pub const SHAPE_KINDS: &[&str] = &["emoji", "rect", "ellipse", "triangle", "star", "arrow", "heart", "speech", "svg"];
/// A `midi` clip's instruments (the video editor's synth voices).
pub const MIDI_INSTRUMENTS: &[&str] = &[
    "grandPiano", "electricPiano", "synthLead", "synthPad", "strings", "organ", "bass", "pluck", "bell", "sawLead", "squareLead", "superSaw",
    "reeseBass", "subBass", "fmBell", "musicBox", "marimba", "vibraphone", "kalimba", "harp", "clav", "brass", "flute", "choir", "glass",
    "sineLead", "pwmPad", "pluckSynth", "eightBit", "drumKit",
];
pub const EASINGS: &[&str] = &["linear", "easeIn", "easeOut", "easeInOut", "hold", "bezier"];
/// A transition's alignment on the cut.
pub const TRANSITION_ALIGN: &[&str] = &["start", "center", "end"];
pub const FILTER_MODES: &[&str] = &["lowpass", "highpass", "bandpass"];
/// A video's audio effects (a clip's, a track's or the master's chain), by tag: each number prop, its default and bounds.
fn video_audio_effect(t: &str) -> Option<&'static [(&'static str, f64, f64, f64)]> {
    Some(match t {
        "gain" => &[("gain", 1.0, 0.0, 16.0)],
        "eq" => &[("lowGain", 0.0, -40.0, 40.0), ("midGain", 0.0, -40.0, 40.0), ("highGain", 0.0, -40.0, 40.0)],
        "filter" => &[("freq", 1200.0, 10.0, 24000.0), ("q", 1.0, 0.0001, 100.0)],
        "delay" => &[("time", 0.3, 0.0, 2.0), ("feedback", 0.4, 0.0, 0.95), ("mix", 0.35, 0.0, 1.0)],
        "reverb" => &[("decay", 2.0, 0.01, 20.0), ("mix", 0.35, 0.0, 1.0)],
        "compressor" => &[("threshold", -24.0, -100.0, 0.0), ("ratio", 4.0, 1.0, 20.0), ("attack", 0.003, 0.0, 1.0), ("release", 0.25, 0.0, 1.0)],
        "distortion" => &[("amount", 0.4, 0.0, 1.0), ("mix", 1.0, 0.0, 1.0)],
        _ => return None,
    })
}

/// One audio effect element as the editor's effect: the tag's engine type (a filter is its mode) and its params.
fn audio_effect(el: &El, ids: &mut Ids) -> Result<Json, String> {
    let spec = video_audio_effect(el.tag).expect("an audio effect");
    let mut allowed: Vec<&str> = spec.iter().map(|(k, ..)| *k).collect();
    allowed.push("enabled");
    if el.tag == "filter" {
        allowed.push("mode");
    }
    refuse_unknown(el, &allowed)?;
    let mut params = Json::obj();
    let kind = if el.tag == "filter" {
        let mode = txt(el, "mode", Some(FILTER_MODES))?.unwrap_or_else(|| "lowpass".into());
        let (f, q) = match mode.as_str() {
            "highpass" => (300.0, 1.0),
            "bandpass" => (1500.0, 2.0),
            _ => (1200.0, 1.0),
        };
        params.set("frequency", num(el, "freq", Some(10.0), Some(24000.0))?.unwrap_or(f));
        params.set("q", num(el, "q", Some(0.0001), Some(100.0))?.unwrap_or(q));
        mode
    } else {
        for (k, d, lo, hi) in spec {
            params.set(k, num(el, k, Some(*lo), Some(*hi))?.unwrap_or(*d));
        }
        el.tag.to_string()
    };
    let id = ids.make(&format!("afx_{kind}"));
    Ok(obj(&[("id", id.as_str().into()), ("type", kind.as_str().into()), ("enabled", flag(el, "enabled")?.unwrap_or(true).into()), ("params", params)]))
}
pub const TRANSITIONS: &[&str] = &[
    "none", "cut", "crossDissolve", "fadeToBlack", "fadeToWhite", "dipToColor", "wipeLeft", "wipeRight", "wipeUp", "wipeDown", "diagonalWipe",
    "barnDoors", "iris", "diamond", "clockWipe", "pixelDissolve", "pushLeft", "pushRight", "slideUp", "slideDown", "zoomIn", "zoomBlur", "glitch",
    "kineticMatte", "shapeWipe",
];
pub const EFFECT_TYPES: &[&str] = &[
    "brightnessContrast", "saturation", "hueRotate", "gaussianBlur", "sharpen", "pixelate", "chromaKey", "twist", "wave", "mirror", "vignette",
    "glow", "grayscale", "sepia", "invert", "posterize", "edges", "chromaticAberration", "bulge", "duotone", "colorWheels", "mask", "lut",
];
/// A clip's intro and outro animations.
pub const ANIM_PRESETS: &[&str] = &["fade", "slideL", "slideR", "slideU", "slideD", "pop", "rise", "spin"];
/// The properties a clip's keyframes animate (an effect's keyframes name a `param` of it).
pub const ANIMATABLE: &[&str] = &["opacity", "volume", "transform.x", "transform.y", "transform.scaleX", "transform.scaleY", "transform.rotation"];

/// A clip's transform and colour: the attributes that set one field of each, and the field's default.
const CLIP_TRANSFORM: &[(&str, f64)] = &[("x", 0.0), ("y", 0.0), ("scaleX", 1.0), ("scaleY", 1.0), ("rotation", 0.0), ("anchorX", 0.5), ("anchorY", 0.5)];
const CLIP_COLOR: &[&str] = &["exposure", "contrast", "saturation", "temperature", "brightness", "hue"];
const CLIP_PROPS: &[&str] = &[
    "name", "start", "duration", "in", "out", "speed", "opacity", "volume", "blendMode", "fitMode", "x", "y", "scaleX", "scaleY", "rotation",
    "anchorX", "anchorY", "exposure", "contrast", "saturation", "temperature", "brightness", "hue", "cropTop", "cropRight", "cropBottom",
    "cropLeft",
];
/// A clip's crop: each edge's attribute and its field.
const CLIP_CROP: &[(&str, &str)] = &[("cropTop", "top"), ("cropRight", "right"), ("cropBottom", "bottom"), ("cropLeft", "left")];
/// A title's text: its fields and their defaults.
fn title_text() -> Vec<(&'static str, Json)> {
    vec![
        ("text", "Title".into()),
        ("fontFamily", "Inter, system-ui, sans-serif".into()),
        ("fontSize", Json::Num(96.0)),
        ("color", "#ffffff".into()),
        ("bold", true.into()),
        ("italic", false.into()),
        ("align", "center".into()),
        ("background", "transparent".into()),
        ("strokeColor", "#000000".into()),
        ("strokeWidth", Json::Num(0.0)),
    ]
}
const TITLE_FIELDS: &[&str] = &["text", "fontFamily", "fontSize", "color", "bold", "italic", "align", "background", "strokeColor", "strokeWidth"];

/// The clip props a clip with no media leaves out (`in`, `out`, `speed`, and `volume` unless it is kept).
fn clip_props_without(left_out: &[&str]) -> Vec<&'static str> {
    CLIP_PROPS.iter().copied().filter(|p| !left_out.contains(p)).collect()
}

/// The props every clip and title shares: its look and level.
fn common(c: &El, inputs: &mut Json) -> Result<(), String> {
    let mut transform = Json::obj();
    for (k, d) in CLIP_TRANSFORM {
        transform.set(k, any(c, k)?.unwrap_or(*d));
    }
    let mut color = Json::obj();
    for k in CLIP_COLOR {
        let bound = if *k == "hue" { 180.0 } else { 1.0 };
        color.set(k, num(c, k, Some(-bound), Some(bound))?.unwrap_or(0.0));
    }
    let fit = txt(c, "fitMode", Some(&["fit", "fill", "stretch"]))?;
    inputs.set("opacity", num(c, "opacity", Some(0.0), Some(1.0))?.unwrap_or(1.0));
    inputs.set("volume", at_least(c, "volume", 0.0)?.unwrap_or(1.0));
    // The blend modes are the compositing set the editor names (its own table); the editor checks the name.
    inputs.set("blendMode", txt(c, "blendMode", None)?.unwrap_or_else(|| "normal".into()));
    if let Some(fit) = fit {
        inputs.set("fitMode", fit);
    }
    inputs.set("transform", transform);
    inputs.set("color", color);
    if CLIP_CROP.iter().any(|(k, _)| c.attr(k).is_some()) {
        let mut crop = Json::obj();
        for (k, edge) in CLIP_CROP {
            crop.set(edge, num(c, k, Some(0.0), Some(0.49))?.unwrap_or(0.0));
        }
        inputs.set("crop", crop);
    }
    Ok(())
}

/// What a clip's children declare: its transition, intro and outro, effects, keyframes and notes.
struct ClipKids {
    transition_in: Json,
    anim_in: Option<Json>,
    anim_out: Option<Json>,
    effects: Vec<Json>,
    audio: Vec<Json>,
    /// Keyframe tracks: the property, and its keys `(time, key)` in time order.
    keyframes: Vec<(String, Vec<(f64, Json)>)>,
    notes: Vec<Json>,
}

/// A clip's children: its transition, its intro and outro, its effects (each with its keyframes) and its keyframes.
fn clip_children(clip: &El, clip_id: &str, sources: &mut Sources) -> Result<ClipKids, String> {
    let mut k = ClipKids {
        transition_in: obj(&[("kind", "none".into()), ("duration", Json::Num(0.0))]),
        anim_in: None,
        anim_out: None,
        effects: Vec::new(),
        audio: Vec::new(),
        keyframes: Vec::new(),
        notes: Vec::new(),
    };
    let (mut note_ids, mut fx_ids, mut kf_ids, mut afx_ids) = (Ids::default(), Ids::default(), Ids::default(), Ids::default());
    let mut add_key = |k: &mut ClipKids, el: &El, property: String, sources: &mut Sources| -> Result<(), String> {
        let time = at_least(el, "time", 0.0)?;
        let value = any(el, "value")?;
        let (Some(time), Some(value)) = (time, value) else { return Err(format!("{}: a keyframe has a time and a value", place(el))) };
        let id = kf_ids.make(&format!("kf_{clip_id}"));
        let easing = txt(el, "easing", Some(EASINGS))?.unwrap_or_else(|| "linear".into());
        let bezier = el.attr("bezier");
        if bezier.is_some() && easing != "bezier" {
            return Err(format!("{}: bezier is read with .easing(\"bezier\")", place(el)));
        }
        let mut key = obj(&[("id", id.as_str().into()), ("time", time.into()), ("value", value.into()), ("easing", easing.as_str().into())]);
        if easing == "bezier" {
            let curve: Option<Vec<f64>> = match bezier {
                Some(Json::Arr(v)) if v.len() == 4 => v.iter().map(|x| match x {
                    Json::Num(n) if n.is_finite() => Some(*n),
                    _ => None,
                }).collect(),
                _ => None,
            };
            match curve {
                Some(c) if (0.0..=1.0).contains(&c[0]) && (0.0..=1.0).contains(&c[2]) => {
                    key.set("bezier", Json::Arr(c.into_iter().map(Json::Num).collect()));
                }
                _ => return Err(format!("{}: an easing \"bezier\" names its curve: .bezier([x1, y1, x2, y2]) (x1 and x2 from 0 to 1)", place(el))),
            }
        }
        let track = match k.keyframes.iter().position(|(p, _)| *p == property) {
            Some(i) => &mut k.keyframes[i].1,
            None => {
                k.keyframes.push((property, Vec::new()));
                &mut k.keyframes.last_mut().expect("a track").1
            }
        };
        track.push((time, key));
        track.sort_by(|a, b| a.0.total_cmp(&b.0));
        sources.put(format!("keyframe:{id}"), el);
        Ok(())
    };
    let mut saw_transition = false;
    for el in clip.child_elements() {
        match el.tag {
            "transition" => {
                refuse_unknown(el, &["kind", "duration", "align", "color", "shape"])?;
                if saw_transition {
                    return Err(format!("{}: a clip has one transition() (into it)", place(clip)));
                }
                saw_transition = true;
                let kind = txt(el, "kind", Some(TRANSITIONS))?.unwrap_or_else(|| "crossDissolve".into());
                k.transition_in = obj(&[("kind", kind.into()), ("duration", at_least(el, "duration", 0.0)?.unwrap_or(0.5).into())]);
                let mut params = Json::obj();
                if let Some(a) = txt(el, "align", Some(TRANSITION_ALIGN))? {
                    params.set("align", a);
                }
                if let Some(c) = txt(el, "color", None)? {
                    params.set("color", c);
                }
                if let Some(n) = at_least(el, "shape", 0.0)? {
                    params.set("shape", n);
                }
                if !matches!(&params, Json::Obj(e) if e.is_empty()) {
                    k.transition_in.set("params", params);
                }
                sources.put("transition", el);
            }
            "note" => {
                if clip.tag != "midi" {
                    return Err(format!("note() is read in a midi() clip, not a {}", tag(clip.tag)));
                }
                refuse_unknown(el, &["pitch", "start", "duration", "velocity"])?;
                let Some(pitch) = pitch_of(el.attr("pitch")) else {
                    let given = el.attr("pitch").map(Json::text).unwrap_or_else(|| "nothing".into());
                    return Err(format!("note(): pitch is a MIDI number 0–127 or a name like \"C4\", not {given}"));
                };
                let (start, duration) = (at_least(el, "start", 0.0)?, at_least(el, "duration", 0.0)?);
                let (Some(start), Some(duration)) = (start, duration) else {
                    return Err("note(): a note has a start and a duration (seconds in the clip)".into());
                };
                let id = note_ids.make(&format!("note_{clip_id}"));
                let velocity = num(el, "velocity", Some(0.0), Some(1.0))?.unwrap_or(0.8);
                k.notes.push(obj(&[("id", id.as_str().into()), ("pitch", pitch.into()), ("start", start.into()), ("duration", duration.into()), ("velocity", velocity.into())]));
                sources.put(format!("note:{id}"), el);
            }
            "intro" | "outro" => {
                refuse_unknown(el, &["preset", "duration"])?;
                let slot = if el.tag == "intro" { &mut k.anim_in } else { &mut k.anim_out };
                if slot.is_some() {
                    return Err(format!("{}: a clip has one {}", place(clip), tag(el.tag)));
                }
                let Some(preset) = txt(el, "preset", Some(ANIM_PRESETS))? else {
                    return Err(format!("{} names its preset ({})", tag(el.tag), ANIM_PRESETS.join(", ")));
                };
                *slot = Some(obj(&[("preset", preset.into()), ("duration", at_least(el, "duration", 0.0)?.unwrap_or(1.0).into())]));
                sources.put(el.tag, el);
            }
            "effect" => {
                let Some(kind) = txt(el, "type", Some(EFFECT_TYPES))? else { return Err(format!("{}: an effect() names its type", place(clip))) };
                let id = fx_ids.make(&format!("fx_{kind}"));
                let src = if kind == "lut" {
                    let Some(src) = txt(el, "src", None)?.filter(|s| !s.is_empty()) else {
                        return Err("effect().type_(\"lut\"): a LUT names its .cube file (.src(\"looks/film.cube\"), relative to this file)".into());
                    };
                    let scheme = src.find(':').is_some_and(|i| i > 0 && src[..i].bytes().all(|b| b.is_ascii_alphabetic()));
                    if scheme || src.starts_with('/') {
                        return Err(format!("effect().type_(\"lut\"): src is a path relative to this file, not {src}"));
                    }
                    Some(src)
                } else if el.attr("src").is_some() {
                    return Err(format!("effect().type_(\"{kind}\"): src is read on a LUT (.type_(\"lut\"))"));
                } else {
                    None
                };
                let mut params = Json::obj();
                for (p, v) in &el.attrs {
                    if ["type", "enabled", "colors", "src"].contains(&p.as_str()) {
                        continue;
                    }
                    match v {
                        Json::Num(n) if n.is_finite() => params.set(p, v.clone()),
                        _ => return Err(format!("effect().type_(\"{kind}\"): {} is a number, not {}", rust_name(p), v.text())),
                    };
                }
                let colors = match el.attr("colors") {
                    None => None,
                    Some(Json::Obj(e)) if e.iter().all(|(_, c)| c.as_str().is_some()) => Some(Json::Obj(e.clone())),
                    // TypeScript copies a list of colours as an object by index.
                    Some(Json::Arr(e)) if e.iter().all(|c| c.as_str().is_some()) => Some(Json::Obj(e.iter().enumerate().map(|(i, c)| (i.to_string(), c.clone())).collect())),
                    Some(_) => return Err(format!("effect().type_(\"{kind}\"): colors is {{ name: \"#rrggbb\" }}")),
                };
                let mut fx = obj(&[("id", id.as_str().into()), ("type", kind.as_str().into()), ("enabled", flag(el, "enabled")?.unwrap_or(true).into()), ("params", params)]);
                if let Some(colors) = colors {
                    fx.set("colors", colors);
                }
                if let Some(src) = src {
                    fx.set("src", src);
                }
                k.effects.push(fx);
                sources.put(format!("effect:{id}"), el);
                for kf in el.child_elements() {
                    if kf.tag != "keyframe" {
                        return Err(format!("{} is not read in an effect() (its keyframes are keyframe().param(…).time(…).value(…))", tag(kf.tag)));
                    }
                    refuse_unknown(kf, &["param", "time", "value", "easing", "bezier"])?;
                    let param = txt(kf, "param", None)?.filter(|p| !p.is_empty());
                    let Some(param) = param else { return Err(format!("{}: an effect's keyframe names its param", place(kf))) };
                    add_key(&mut k, kf, format!("effect.{id}.{param}"), sources)?;
                }
            }
            "keyframe" => {
                refuse_unknown(el, &["property", "time", "value", "easing", "bezier"])?;
                let property = txt(el, "property", Some(ANIMATABLE))?.filter(|p| !p.is_empty());
                let Some(property) = property else {
                    return Err(format!("{}: a keyframe names its property ({})", place(el), ANIMATABLE.join(", ")));
                };
                add_key(&mut k, el, property, sources)?;
            }
            t if video_audio_effect(t).is_some() => {
                let picture = clip.tag == "clip" && clip.attr("src").and_then(Json::as_str).and_then(media_kind) == Some("image");
                if (clip.tag != "clip" && clip.tag != "midi") || picture {
                    return Err(format!("{} is an audio effect: {} has no sound", tag(el.tag), place(clip)));
                }
                let fx = audio_effect(el, &mut afx_ids)?;
                let id = fx.get("id").and_then(Json::as_str).unwrap_or_default().to_string();
                k.audio.push(fx);
                sources.put(format!("audio:{id}"), el);
            }
            _ => {
                return Err(format!(
                    "{} is not read in a {} (a clip holds transition(), intro(), outro(), effect(), keyframe() and audio effects; a midi() clip its note()s)",
                    tag(el.tag),
                    tag(clip.tag)
                ))
            }
        }
    }
    Ok(k)
}

/// Declare a video (`video(…)` and its tracks) as the video editor's own op graph.
fn declare_video(root: &El) -> Result<Json, String> {
    refuse_unknown(root, &["name", "width", "height", "fps", "sampleRate", "background"])?;
    let name = txt(root, "name", None)?.unwrap_or_else(|| "Video".into());
    let settings = obj(&[
        ("width", at_least(root, "width", 1.0)?.unwrap_or(1920.0).into()),
        ("height", at_least(root, "height", 1.0)?.unwrap_or(1080.0).into()),
        ("fps", at_least(root, "fps", 1.0)?.unwrap_or(30.0).into()),
        ("sampleRate", at_least(root, "sampleRate", 1.0)?.unwrap_or(48000.0).into()),
        ("background", txt(root, "background", None)?.unwrap_or_else(|| "#000000".into()).into()),
    ]);
    let mut nodes = Nodes::default();
    let mut ids = Ids::default();
    ids.make("composite");
    let mut track_wires = Vec::new();
    let mut markers = Vec::new();
    let mut composite_sources = Sources::new();
    let mut master = Vec::new();
    let mut master_ids = Ids::default();
    for tr in root.child_elements() {
        if video_audio_effect(tr.tag).is_some() {
            let fx = audio_effect(tr, &mut master_ids)?;
            let id = fx.get("id").and_then(Json::as_str).unwrap_or_default().to_string();
            master.push(fx);
            composite_sources.put(format!("audio:{id}"), tr);
            continue;
        }
        if tr.tag == "marker" {
            refuse_unknown(tr, &["name", "time", "color"])?;
            let Some(time) = at_least(tr, "time", 0.0)? else { return Err(format!("{}: a marker has a time", place(tr))) };
            let mid = ids.make(&format!("marker_{}", markers.len() + 1));
            let marker_name = txt(tr, "name", None)?.unwrap_or_else(|| "Marker".into());
            let color = txt(tr, "color", None)?.unwrap_or_else(|| "#f5c542".into());
            markers.push(obj(&[("id", mid.as_str().into()), ("time", time.into()), ("name", marker_name.into()), ("color", color.into())]));
            composite_sources.put(format!("marker:{mid}"), tr);
            continue;
        }
        if tr.tag != "track" {
            return Err(format!("{} is not read in a video() (it holds track(), marker() and the master's audio effects)", tag(tr.tag)));
        }
        refuse_unknown(tr, &["name", "kind", "muted", "hidden", "locked", "volume", "height"])?;
        let kind = txt(tr, "kind", Some(VIDEO_TRACK_KINDS))?.unwrap_or_else(|| "video".into());
        let track_name = txt(tr, "name", None)?.unwrap_or_else(|| match kind.as_str() {
            "audio" => "A".into(),
            "midi" => "M".into(),
            _ => "V".into(),
        });
        let track_id = ids.make(&format!("track_{track_name}"));
        let mut clip_wires = Vec::new();
        let mut track_fx = Vec::new();
        let mut track_sources = Sources::new();
        let mut track_fx_ids = Ids::default();
        for c in tr.child_elements() {
            if video_audio_effect(c.tag).is_some() {
                let fx = audio_effect(c, &mut track_fx_ids)?;
                let id = fx.get("id").and_then(Json::as_str).unwrap_or_default().to_string();
                track_fx.push(fx);
                track_sources.put(format!("audio:{id}"), c);
                continue;
            }
            let mut sources = Sources::new();
            let mut inputs: Json;
            let clip_name: String;
            let source: Json;
            let start = |c: &El| at_least(c, "start", 0.0).map(|s| s.unwrap_or(0.0));
            let duration_given = |c: &El| -> Result<f64, String> {
                at_least(c, "duration", 0.0)?.ok_or_else(|| format!("{}: give its duration in seconds", place(c)))
            };
            if c.tag == "clip" {
                refuse_unknown(c, &[&["src"][..], CLIP_PROPS].concat())?;
                let src = txt(c, "src", None)?.filter(|s| !s.is_empty());
                let Some(src) = src else { return Err("clip() names its media file (.src(\"media/take-1.mp4\"), relative to this file)".into()) };
                let scheme = src.find(':').is_some_and(|i| i > 0 && src[..i].bytes().all(|b| b.is_ascii_alphabetic()));
                if scheme || src.starts_with('/') {
                    return Err(format!("{}: src is a path relative to this file, not {src}", place(c)));
                }
                let Some(mk) = media_kind(&src) else { return Err(format!("{}: {src} is not a video, audio or image file this editor reads", place(c))) };
                if (mk == "audio") != (kind == "audio") {
                    let (what, k) = if mk == "audio" { ("a sound goes on an audio track", "audio") } else { ("a picture goes on a video track", "video") };
                    return Err(format!("{}: {what} (.kind(\"{k}\"))", place(c)));
                }
                let asset_id = slug(&src);
                let source_id = format!("src_{asset_id}");
                let file_name = src.rsplit('/').next().unwrap_or(&src).to_string();
                if !nodes.has(&source_id) {
                    let asset = obj(&[
                        ("id", asset_id.as_str().into()),
                        ("kind", mk.into()),
                        ("name", file_name.as_str().into()),
                        ("url", "".into()),
                        ("path", src.as_str().into()),
                        ("duration", Json::Num(0.0)),
                        ("width", Json::Num(0.0)),
                        ("height", Json::Num(0.0)),
                        ("hasAudio", (mk != "image").into()),
                    ]);
                    let inputs = obj(&[("asset", asset), ("__asset", obj(&[("kind", "file".into()), ("$file", src.as_str().into())]))]);
                    nodes.put(&source_id, node(&source_id, "video.source", inputs, Some(&file_name), None));
                }
                clip_name = txt(c, "name", None)?.unwrap_or(file_name);
                let speed = at_least(c, "speed", 0.01)?.unwrap_or(1.0);
                let in_point = at_least(c, "in", 0.0)?.unwrap_or(0.0);
                let out = at_least(c, "out", 0.0)?;
                let mut duration = at_least(c, "duration", 0.0)?;
                if out.is_some() && duration.is_some() {
                    return Err(format!("{}: give out or duration, not both", place(c)));
                }
                if let Some(out) = out {
                    if mk == "image" {
                        return Err(format!("{}: a picture has no out (give its duration)", place(c)));
                    }
                    if out <= in_point {
                        return Err(format!("{}: out ({}) is after in ({})", place(c), number_text(out), number_text(in_point)));
                    }
                    duration = Some((out - in_point) / speed);
                }
                let Some(duration) = duration else {
                    let what = if mk == "image" { "duration" } else { "out (or duration)" };
                    return Err(format!("{}: give its {what} in seconds", place(c)));
                };
                inputs = obj(&[
                    ("kind", mk.into()),
                    ("assetId", asset_id.as_str().into()),
                    ("name", clip_name.as_str().into()),
                    ("start", start(c)?.into()),
                    ("duration", duration.into()),
                    ("inPoint", in_point.into()),
                    ("speed", speed.into()),
                ]);
                common(c, &mut inputs)?;
                source = wire(&source_id, "media");
            } else if c.tag == "title" {
                refuse_unknown(c, &[clip_props_without(&["in", "out", "speed", "volume"]).as_slice(), TITLE_FIELDS, &["typewriter"]].concat())?;
                if kind != "video" {
                    return Err(format!("{}: a title goes on a video track", place(c)));
                }
                clip_name = txt(c, "name", None)?.unwrap_or_else(|| "Title".into());
                let duration = duration_given(c)?;
                let mut text = Json::obj();
                for (k, d) in title_text() {
                    let v = match d {
                        Json::Num(_) => at_least(c, k, 0.0)?.map(Json::Num),
                        Json::Bool(_) => flag(c, k)?.map(Json::Bool),
                        _ => txt(c, k, (k == "align").then_some(&["left", "center", "right"][..]))?.map(Json::from),
                    };
                    text.set(k, v.unwrap_or(d));
                }
                if let Some(tw) = at_least(c, "typewriter", 0.0)? {
                    text.set("typewriter", tw);
                }
                inputs = obj(&[
                    ("kind", "text".into()),
                    ("name", clip_name.as_str().into()),
                    ("start", start(c)?.into()),
                    ("duration", duration.into()),
                    ("inPoint", Json::Num(0.0)),
                    ("speed", Json::Num(1.0)),
                ]);
                common(c, &mut inputs)?;
                inputs.set("volume", 1i64);
                inputs.set("text", text);
                source = Json::Null;
            } else if c.tag == "shape" || c.tag == "adjustment" {
                // A shape and an adjustment layer are synthetic: no media, no in/out, speed 1, no sound.
                let own: &[&str] = if c.tag == "shape" { &["kind", "content", "fill", "stroke", "strokeWidth"] } else { &[] };
                refuse_unknown(c, &[clip_props_without(&["in", "out", "speed", "volume"]).as_slice(), own].concat())?;
                if kind != "video" {
                    return Err(format!("{}: a {} goes on a video track", place(c), tag(c.tag)));
                }
                clip_name = txt(c, "name", None)?.unwrap_or_else(|| if c.tag == "shape" { "Sticker" } else { "Adjustment" }.into());
                let duration = duration_given(c)?;
                inputs = obj(&[
                    ("kind", c.tag.into()),
                    ("name", clip_name.as_str().into()),
                    ("start", start(c)?.into()),
                    ("duration", duration.into()),
                    ("inPoint", Json::Num(0.0)),
                    ("speed", Json::Num(1.0)),
                ]);
                common(c, &mut inputs)?;
                inputs.set("volume", 1i64);
                if c.tag == "shape" {
                    let Some(sk) = txt(c, "kind", Some(SHAPE_KINDS))? else {
                        return Err(format!("{}: a shape() names its kind ({})", place(c), SHAPE_KINDS.join(", ")));
                    };
                    let content = txt(c, "content", None)?.unwrap_or_else(|| if sk == "star" { "⭐".into() } else { String::new() });
                    inputs.set(
                        "sticker",
                        obj(&[
                            ("kind", sk.into()),
                            ("content", content.into()),
                            ("fill", txt(c, "fill", None)?.unwrap_or_else(|| "#ffd166".into()).into()),
                            ("stroke", txt(c, "stroke", None)?.unwrap_or_else(|| "#00000000".into()).into()),
                            ("strokeWidth", at_least(c, "strokeWidth", 0.0)?.unwrap_or(0.0).into()),
                        ]),
                    );
                }
                source = Json::Null;
            } else if c.tag == "midi" {
                refuse_unknown(c, &[clip_props_without(&["in", "out", "speed", "cropTop", "cropRight", "cropBottom", "cropLeft"]).as_slice(), &["instrument", "gain"]].concat())?;
                if kind != "midi" {
                    return Err(format!("{}: a midi() clip goes on a midi track (.kind(\"midi\"))", place(c)));
                }
                clip_name = txt(c, "name", None)?.unwrap_or_else(|| "MIDI".into());
                let duration = duration_given(c)?;
                inputs = obj(&[
                    ("kind", "midi".into()),
                    ("name", clip_name.as_str().into()),
                    ("start", start(c)?.into()),
                    ("duration", duration.into()),
                    ("inPoint", Json::Num(0.0)),
                    ("speed", Json::Num(1.0)),
                ]);
                common(c, &mut inputs)?;
                source = Json::Null;
            } else {
                return Err(format!("{} is not read on a track() (it holds clip(), title(), shape(), adjustment(), midi() and audio effects)", tag(c.tag)));
            }
            if kind == "midi" && c.tag != "midi" {
                return Err(format!("{}: a midi track holds midi() clips", place(c)));
            }
            let clip_id = ids.make(&format!("clip_{clip_name}"));
            let kids = clip_children(c, &clip_id, &mut sources)?;
            inputs.set("transitionIn", kids.transition_in);
            if let Some(a) = kids.anim_in {
                inputs.set("animIn", a);
            }
            if let Some(a) = kids.anim_out {
                inputs.set("animOut", a);
            }
            inputs.set("effects", Json::Arr(kids.effects));
            if !kids.audio.is_empty() {
                inputs.set("audioEffects", Json::Arr(kids.audio));
            }
            inputs.set("trackers", Json::Arr(Vec::new()));
            if !kids.keyframes.is_empty() {
                let tracks = kids.keyframes.into_iter().map(|(property, keys)| {
                    obj(&[("property", property.into()), ("keys", Json::Arr(keys.into_iter().map(|(_, k)| k).collect()))])
                });
                inputs.set("keyframes", Json::Arr(tracks.collect()));
            }
            if c.tag == "midi" {
                let instrument = txt(c, "instrument", Some(MIDI_INSTRUMENTS))?.unwrap_or_else(|| "grandPiano".into());
                let gain = at_least(c, "gain", 0.0)?.unwrap_or(0.8);
                inputs.set("midi", obj(&[("notes", Json::Arr(kids.notes)), ("instrument", instrument.into()), ("gain", gain.into())]));
            }
            inputs.set("source", source);
            nodes.put(&clip_id, node(&clip_id, "video.clip", inputs, Some(&clip_name), meta_of(c, sources, Json::obj())));
            clip_wires.push(wire(&clip_id, "frames"));
        }
        let height = match kind.as_str() {
            "audio" => 48.0,
            "midi" => 56.0,
            _ => 64.0,
        };
        let mut track_inputs = obj(&[
            ("kind", kind.as_str().into()),
            ("name", track_name.as_str().into()),
            ("muted", flag(tr, "muted")?.unwrap_or(false).into()),
            ("hidden", flag(tr, "hidden")?.unwrap_or(false).into()),
            ("locked", flag(tr, "locked")?.unwrap_or(false).into()),
            ("height", at_least(tr, "height", 16.0)?.unwrap_or(height).into()),
            ("volume", at_least(tr, "volume", 0.0)?.unwrap_or(1.0).into()),
        ]);
        if !track_fx.is_empty() {
            track_inputs.set("audioEffects", Json::Arr(track_fx));
        }
        for (i, w) in clip_wires.into_iter().enumerate() {
            track_inputs.set(&format!("clips.{}", i + 1), w);
        }
        nodes.put(&track_id, node(&track_id, "video.track", track_inputs, Some(&track_name), meta_of(tr, track_sources, Json::obj())));
        track_wires.push(wire(&track_id, "frames"));
    }
    let project_id = slug(&format!("proj_{name}"));
    let mut composite = obj(&[("id", project_id.as_str().into()), ("name", name.as_str().into()), ("settings", settings)]);
    if !markers.is_empty() {
        composite.set("markers", Json::Arr(markers));
    }
    if !master.is_empty() {
        composite.set("masterEffects", Json::Arr(master));
    }
    composite.set("createdAt", 0i64);
    composite.set("updatedAt", 0i64);
    for (i, w) in track_wires.into_iter().enumerate() {
        composite.set(&format!("tracks.{}", i + 1), w);
    }
    nodes.put("composite", node("composite", "video.composite", composite, Some(&name), meta_of(root, composite_sources, Json::obj())));
    Ok(obj(&[
        ("id", project_id.as_str().into()),
        ("nodes", Json::Obj(nodes.0)),
        ("outputs", Json::Arr(vec!["composite".into()])),
        ("meta", obj(&[("kind", "video".into())])),
    ]))
}

// ── Song ─────────────────────────────────────────────────────────────────────────────────────────────────────────

const NOTE_NAMES: &[&str] = &["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// A pitch as MIDI: a number (60), or a name with its octave ("C4" is 60, "F#3", "Bb2").
pub fn pitch_of(v: Option<&Json>) -> Option<i64> {
    match v? {
        Json::Num(n) => (n.fract() == 0.0 && (0.0..=127.0).contains(n)).then_some(*n as i64),
        Json::Str(s) => {
            let s = s.trim();
            let mut chars = s.chars();
            let letter = match chars.next()?.to_ascii_uppercase() {
                'C' => 0,
                'D' => 2,
                'E' => 4,
                'F' => 5,
                'G' => 7,
                'A' => 9,
                'B' => 11,
                _ => return None,
            };
            let rest: String = chars.collect();
            let (accidental, octave) = match rest.chars().next() {
                Some('#') => (1, &rest[1..]),
                Some('b') => (-1, &rest[1..]),
                _ => (0, rest.as_str()),
            };
            let digits = octave.strip_prefix('-').unwrap_or(octave);
            if digits.len() != 1 || !digits.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let p = letter + accidental + (octave.parse::<i64>().ok()? + 1) * 12;
            (0..=127).contains(&p).then_some(p)
        }
        _ => None,
    }
}

/// A MIDI pitch's name ("C4" for 60; sharps).
pub fn pitch_name(p: i64) -> String {
    format!("{}{}", NOTE_NAMES[p.rem_euclid(12) as usize], p.div_euclid(12) - 1)
}

pub const WAVES: &[&str] = &["sine", "square", "sawtooth", "triangle"];

/// The effects a track's chain can hold: the tag, its node type, its default name and its spec's fields.
fn song_effect(t: &str) -> Option<(&'static str, &'static str, Vec<(&'static str, Json)>)> {
    Some(match t {
        "gain" => ("fx.gain", "Gain", vec![("gain", Json::Num(1.0))]),
        "filter" => ("fx.filter", "Filter", vec![("mode", "lowpass".into()), ("freq", Json::Num(1200.0)), ("q", Json::Num(1.0))]),
        "delay" => ("fx.delay", "Delay", vec![("timeBeats", Json::Num(0.5)), ("feedback", Json::Num(0.35)), ("mix", Json::Num(0.3))]),
        "reverb" => ("fx.reverb", "Reverb", vec![("decay", Json::Num(1.8)), ("mix", Json::Num(0.3))]),
        "eq" => ("fx.eq", "EQ", vec![("lowGain", Json::Num(0.0)), ("midGain", Json::Num(0.0)), ("highGain", Json::Num(0.0))]),
        _ => return None,
    })
}

/// A time signature written "3/4" → `{"num": 3, "den": 4}`.
pub fn time_signature_of(v: Option<&Json>) -> Option<Json> {
    let s = v?.as_str()?;
    let (a, b) = s.trim().split_once('/')?;
    let (a, b) = (a.trim(), b.trim());
    if a.is_empty() || b.is_empty() || !a.bytes().chain(b.bytes()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    let (num, den) = (a.parse::<u64>().ok()?, b.parse::<u64>().ok()?);
    ((1..=32).contains(&num) && [1, 2, 4, 8, 16, 32].contains(&den)).then(|| obj(&[("num", Json::Num(num as f64)), ("den", Json::Num(den as f64))]))
}

/// A tempo: beats per minute from the start (`.tempo(120)`), or a map of changes (`[{"atBeat": 0, "bpm": 120}, …]`).
pub fn tempo_of(v: &Json) -> Option<Json> {
    let change = |at: f64, bpm: f64| obj(&[("atBeat", at.into()), ("bpm", bpm.into())]);
    match v {
        Json::Num(n) if *n > 0.0 && n.is_finite() => Some(Json::Arr(vec![change(0.0, *n)])),
        Json::Arr(list) if !list.is_empty() => {
            let mut out = Vec::new();
            let mut prev = f64::NEG_INFINITY;
            for c in list {
                let (Some(at), Some(bpm)) = (c.get("atBeat").and_then(Json::as_f64), c.get("bpm").and_then(Json::as_f64)) else { return None };
                if !(bpm > 0.0) || at < prev {
                    return None;
                }
                out.push(change(at, bpm));
                prev = at;
            }
            (out[0].get("atBeat").and_then(Json::as_f64)? <= 0.0).then_some(Json::Arr(out))
        }
        _ => None,
    }
}

/// A sound file named by a path relative to the song (`src`), checked to be one.
fn sound_src(el: &El) -> Result<String, String> {
    let Some(src) = txt(el, "src", None)?.filter(|s| !s.is_empty()) else {
        return Err(format!("{} names its sound file (.src(\"media/tone.wav\"), relative to this file)", tag(el.tag)));
    };
    let scheme = src.find(':').is_some_and(|i| i > 0 && src[..i].bytes().all(|b| b.is_ascii_alphabetic()));
    if scheme || src.starts_with('/') {
        return Err(format!("{}: src is a path relative to this file, not {src}", place(el)));
    }
    if media_kind(&src) != Some("audio") {
        return Err(format!("{}: {src} is not a sound file this studio reads", place(el)));
    }
    Ok(src)
}

/// A file's name without its folder and extension: an audio clip's name when it gives none.
fn stem_of(src: &str) -> String {
    let base = src.rsplit_once('/').map_or(src, |(_, b)| b);
    base.rsplit_once('.').map_or(base, |(s, _)| s).to_string()
}

/// Declare a song (`song(…)` and its tracks) as the music studio's own op graph. A track plays a `synth()` or a
/// `sampler()` (an instrument: its clips hold notes), or holds audio clips (`clip([]).src(…)`: a sound file placed on
/// the song, played by the track's player); effects follow the instrument or the player.
fn declare_song(root: &El) -> Result<Json, String> {
    refuse_unknown(root, &["name", "volumeDb", "tempo", "timeSignature", "bars", "cycleStart", "cycleEnd"])?;
    let name = txt(root, "name", None)?.unwrap_or_else(|| "Song".into());
    let tempo = match root.attr("tempo") {
        None => tempo_of(&Json::Num(120.0)),
        Some(v) => tempo_of(v),
    };
    let Some(tempo) = tempo else {
        return Err("song(): tempo is beats per minute (120), or [{ atBeat: 0, bpm: 120 }, …] sorted by beat".into());
    };
    let default_signature = Json::from("4/4");
    let Some(time_sig) = time_signature_of(Some(root.attr("timeSignature").unwrap_or(&default_signature))) else {
        return Err("song(): time_signature is \"beats/unit\" (\"4/4\", \"6/8\")".into());
    };
    let mut nodes = Nodes::default();
    let mut ids = Ids::default();
    ids.make("master");
    let mut master = obj(&[
        ("name", "Master".into()),
        ("volumeDb", any(root, "volumeDb")?.unwrap_or(0.0).into()),
        ("tempo", tempo),
        ("timeSig", time_sig),
        ("bars", at_least(root, "bars", 1.0)?.unwrap_or(8.0).into()),
    ]);
    match (at_least(root, "cycleStart", 0.0)?, at_least(root, "cycleEnd", 0.0)?) {
        (None, None) => {}
        (Some(start), Some(end)) => {
            if !(end > start) {
                return Err(format!("song(): cycle_end ({}) comes after cycle_start ({})", number_text(end), number_text(start)));
            }
            master.set("cycle", obj(&[("start", start.into()), ("end", end.into())]));
        }
        _ => return Err("song(): a cycle names both its cycle_start and its cycle_end (beats)".into()),
    }
    let mut order = 0usize;
    for tr in root.child_elements() {
        if tr.tag != "track" {
            return Err(format!("{} is not read in a song() (it holds track())", tag(tr.tag)));
        }
        refuse_unknown(tr, &["name", "colorIndex", "volumeDb", "pan", "mute", "solo"])?;
        let track_name = txt(tr, "name", None)?.unwrap_or_else(|| format!("Track {}", order + 1));
        let track_id = ids.make(&format!("track_{track_name}"));
        let children: Vec<&El> = tr.child_elements().collect();
        let audio_track = !children.is_empty() && !children.iter().any(|el| el.tag == "synth" || el.tag == "sampler");
        let mut upstream: Option<String> = None;
        let mut clips: Vec<String> = Vec::new();
        let mut instrument: Option<String> = None;
        let mut player: Option<String> = None;
        let mut note_clips = false;
        if audio_track {
            let id = ids.make(&format!("player_{track_name}"));
            nodes.put(&id, node(&id, "player", obj(&[("name", track_name.as_str().into())]), None, None));
            player = Some(id.clone());
            upstream = Some(id);
        }
        for el in children {
            if el.tag == "synth" || el.tag == "sampler" {
                if instrument.is_some() {
                    return Err(format!("{}: a track plays one instrument (a synth() or a sampler())", place(tr)));
                }
                let mut spec = if el.tag == "synth" {
                    refuse_unknown(el, &["name", "wave", "gain", "detune", "voices", "transpose", "attack", "decay", "sustain", "release"])?;
                    let mut spec = obj(&[("kind", "synth".into()), ("wave", txt(el, "wave", Some(WAVES))?.unwrap_or_else(|| "sawtooth".into()).into())]);
                    for (k, d) in [("gain", 0.7), ("detune", 8.0), ("voices", 1.0), ("transpose", 0.0)] {
                        spec.set(k, any(el, k)?.unwrap_or(d));
                    }
                    spec
                } else {
                    refuse_unknown(el, &["name", "src", "root", "gain", "attack", "decay", "sustain", "release"])?;
                    let base = match el.attr("root") {
                        None => Some(69),
                        Some(v) => pitch_of(Some(v)),
                    };
                    let Some(base) = base else {
                        let given = el.attr("root").map(Json::text).unwrap_or_default();
                        return Err(format!("sampler(): root is the pitch the file sounds at, a MIDI number (0–127) or a name (\"A4\"), not {given}"));
                    };
                    obj(&[
                        ("kind", "sampler".into()),
                        ("src", sound_src(el)?.into()),
                        ("baseNote", base.into()),
                        ("gain", at_least(el, "gain", 0.0)?.unwrap_or(1.0).into()),
                    ])
                };
                let mut env = Json::obj();
                for (k, d) in [("attack", 0.01), ("decay", 0.15), ("sustain", 0.6), ("release", 0.25)] {
                    env.set(k, at_least(el, k, 0.0)?.unwrap_or(d));
                }
                spec.set("env", env);
                let id = ids.make(&format!("inst_{track_name}"));
                let inputs = obj(&[("name", txt(el, "name", None)?.unwrap_or_else(|| track_name.clone()).into()), ("spec", spec)]);
                nodes.put(&id, node(&id, "instrument", inputs, None, meta_of(el, Sources::new(), Json::obj())));
                instrument = Some(id.clone());
                upstream = Some(id);
            } else if let Some((fx_type, fx_name, fields)) = song_effect(el.tag) {
                let Some(up) = upstream.clone() else {
                    return Err(format!("{} comes after the track's instrument (the chain runs instrument → effects → track)", tag(el.tag)));
                };
                let allowed: Vec<&str> = std::iter::once("name").chain(fields.iter().map(|(k, _)| *k)).collect();
                refuse_unknown(el, &allowed)?;
                let mut spec = obj(&[("type", el.tag.into())]);
                for (k, d) in fields {
                    let v = match d {
                        Json::Num(_) => any(el, k)?.map(Json::Num),
                        _ => txt(el, k, Some(&["lowpass", "highpass", "bandpass"]))?.map(Json::from),
                    };
                    spec.set(k, v.unwrap_or(d));
                }
                let fid = ids.make(&format!("fx_{track_name}_{}", el.tag));
                let inputs = obj(&[("name", txt(el, "name", None)?.unwrap_or_else(|| fx_name.into()).into()), ("spec", spec), ("audio", wire(&up, "audio"))]);
                nodes.put(&fid, node(&fid, fx_type, inputs, None, meta_of(el, Sources::new(), Json::obj())));
                upstream = Some(fid);
            } else if el.tag == "clip" && player.is_some() && el.attr("src").is_some() {
                refuse_unknown(el, &["name", "src", "start", "length", "in", "volume", "loop"])?;
                if el.child_elements().next().is_some() {
                    return Err(format!("{}: an audio clip holds no notes (notes go in a clip on a track with a synth() or a sampler())", place(el)));
                }
                let src = sound_src(el)?;
                let clip_name = txt(el, "name", None)?.unwrap_or_else(|| stem_of(&src));
                let cid = ids.make(&format!("clip_{clip_name}"));
                let inputs = obj(&[
                    ("name", clip_name.as_str().into()),
                    ("src", src.into()),
                    ("start", at_least(el, "start", 0.0)?.unwrap_or(0.0).into()),
                    ("length", at_least(el, "length", 0.0)?.unwrap_or(4.0).into()),
                    ("offsetSeconds", at_least(el, "in", 0.0)?.unwrap_or(0.0).into()),
                    ("gain", at_least(el, "volume", 0.0)?.unwrap_or(1.0).into()),
                    ("loop", flag(el, "loop")?.unwrap_or(false).into()),
                ]);
                nodes.put(&cid, node(&cid, "sample", inputs, None, meta_of(el, Sources::new(), Json::obj())));
                clips.push(cid);
            } else if el.tag == "clip" {
                if el.attr("src").is_some() {
                    return Err(format!("{}: a track that plays an instrument holds clips of notes; an audio clip (src) goes on a track without one", place(tr)));
                }
                refuse_unknown(el, &["name", "start", "length", "loop"])?;
                let clip_name = txt(el, "name", None)?.unwrap_or_else(|| format!("{track_name} {}", clips.len() + 1));
                let cid = ids.make(&format!("clip_{clip_name}"));
                let mut notes = Vec::new();
                let mut sources = Sources::new();
                for n in el.child_elements() {
                    if n.tag != "note" {
                        return Err(format!("{} is not read in a clip() (it holds note())", tag(n.tag)));
                    }
                    refuse_unknown(n, &["pitch", "start", "duration", "velocity"])?;
                    let Some(pitch) = pitch_of(n.attr("pitch")) else {
                        let given = n.attr("pitch").map(Json::text).unwrap_or_else(|| "nothing".into());
                        return Err(format!("note(): pitch is a MIDI number (0–127) or a name (\"C4\", \"F#3\"), not {given}"));
                    };
                    let nid = format!("{cid}_n{}", notes.len() + 1);
                    notes.push(obj(&[
                        ("id", nid.as_str().into()),
                        ("pitch", pitch.into()),
                        ("start", at_least(n, "start", 0.0)?.unwrap_or(0.0).into()),
                        ("dur", at_least(n, "duration", 0.0)?.unwrap_or(1.0).into()),
                        ("vel", num(n, "velocity", Some(0.0), Some(1.0))?.unwrap_or(0.8).into()),
                    ]));
                    sources.put(format!("note:{nid}"), n);
                }
                let inputs = obj(&[
                    ("name", clip_name.as_str().into()),
                    ("start", at_least(el, "start", 0.0)?.unwrap_or(0.0).into()),
                    ("length", at_least(el, "length", 0.0)?.unwrap_or(4.0).into()),
                    ("notes", Json::Arr(notes)),
                    ("loop", flag(el, "loop")?.unwrap_or(false).into()),
                ]);
                nodes.put(&cid, node(&cid, "midiClip", inputs, None, meta_of(el, sources, Json::obj())));
                clips.push(cid);
                note_clips = true;
            } else {
                return Err(format!("{} is not read on a track() (it holds a synth() or a sampler(), effects (gain, filter, delay, reverb, eq) and clip())", tag(el.tag)));
            }
        }
        let (holder, port) = match (&instrument, &player) {
            (Some(inst), _) => (inst.clone(), "midi"),
            (None, Some(p)) => {
                if note_clips {
                    return Err(format!("{}: its clips need a synth() or a sampler() to play them", place(tr)));
                }
                (p.clone(), "audio")
            }
            (None, None) => (String::new(), ""),
        };
        if !holder.is_empty() {
            if let Some(Json::Obj(fields)) = nodes.get_mut(&holder) {
                if let Some((_, inputs)) = fields.iter_mut().find(|(k, _)| k == "inputs") {
                    for (i, c) in clips.iter().enumerate() {
                        inputs.set(&format!("{port}.{}", i + 1), wire(c, port));
                    }
                }
            }
        }
        let mut track_inputs = obj(&[
            ("name", track_name.as_str().into()),
            ("volumeDb", any(tr, "volumeDb")?.unwrap_or(0.0).into()),
            ("pan", num(tr, "pan", Some(-1.0), Some(1.0))?.unwrap_or(0.0).into()),
            ("mute", flag(tr, "mute")?.unwrap_or(false).into()),
            ("solo", flag(tr, "solo")?.unwrap_or(false).into()),
            ("colorIndex", at_least(tr, "colorIndex", 0.0)?.unwrap_or(order as f64).into()),
        ]);
        if let Some(up) = &upstream {
            track_inputs.set("audio", wire(up, "audio"));
        }
        let extra = obj(&[("order", Json::Num(order as f64))]);
        nodes.put(&track_id, node(&track_id, "track", track_inputs, None, meta_of(tr, Sources::new(), extra)));
        master.set(&format!("audio.{}", order + 1), wire(&track_id, "audio"));
        order += 1;
    }
    nodes.put("master", node("master", "master", master, None, meta_of(root, Sources::new(), Json::obj())));
    Ok(obj(&[
        ("id", slug(&format!("music_{name}")).into()),
        ("nodes", Json::Obj(nodes.0)),
        ("outputs", Json::Arr(vec!["master".into()])),
        ("meta", obj(&[("domain", "music".into()), ("name", name.into())])),
    ]))
}
