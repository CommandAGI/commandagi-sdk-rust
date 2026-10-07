//! 2D DOCUMENTS IN RUST — a drawing, a paint document, a photo and a nest, declared as the nodes the CommandAGI 2D
//! editors draw and edit. The same nodes, node for node, as the TypeScript SDK's JSX (`twod.ts`) and the Python SDK's
//! `commandagi.design.twod`:
//!
//! ```
//! use commandagi::design::twod::*;
//!
//! fn document() -> El {
//!     drawing([layer([
//!         rect().x(40).y(40).w(200).h(120).fill("#3b82f6"),
//!         group([
//!             ellipse().cx(500).cy(300).rx(60).ry(60).fill("#f59e0b"),
//!             path().d("M 440 300 L 560 300").stroke_color("#111111").stroke_width(2),
//!         ])
//!         .name("Badge"),
//!     ])
//!     .name("Layer 1")])
//!     .name("Poster")
//!     .width(800)
//!     .height(600)
//!     .background("#ffffff")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The root names the document: `drawing` (layers of shapes, `.draw.rs`), `painting` (a layer stack of brush strokes,
//! `.paint.rs`), `photo` (pixel layers, adjustments, filters, `.img.rs`) and `nest` (flat parts nested on a sheet,
//! `.nest.rs`).
//!
//! ONE RULE FOR EVERY TAG: an element is one node, its attributes are the node's inputs (`.stroke_width(2)` is
//! `strokeWidth`), and its children are the nodes it takes, in order. `.id(…)`, `.label(…)` and `.disabled(true)` set
//! the node's own fields. The encodings are the TypeScript SDK's: a top-level group of a drawing is a `layer`; a path's
//! `d` (absolute M L H V C Q Z); a drawn brush stroke's `[x, y]` points; a paint stroke's `[x, y, pressure, t]` points;
//! a modifier (`blur`, `transform`, `fill` …) wraps the one node it takes; `clip` takes its content, then its mask; in a
//! painting or a photo a `mask` child of a layer, a stroke or a filter holds the one layer that masks it; a painting's
//! `text_layer` is a type layer; a painting layer's `fx` holds its styles (`drop_shadow`, `inner_shadow`, `outer_glow`,
//! `stroke`, `color_overlay`, `gradient_overlay`). A value that
//! is an object is JSON text: `.brush(json(r#"{"kind": "round", "size": 4}"#))`. PIXELS ARE NOT CODE: a pixel layer
//! names its image file by relative path (`.src("scan.png")`). Each node carries its element's call in `meta.source`; a
//! `mask` is not a node, so a masked node carries the mask's call in `meta.sources.mask`. A tag the vocabulary does not
//! have is refused by name.
//!
//! A root without `.name(…)` declares the document named after its file, as TypeScript does ("Drawing" with no file).

use super::element::{attributes, elements, rust_name};
use super::ir::{channels, slug, wire, Scope};
use super::read::{call, tag};
use super::source::meta;
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A drawing: its layers.
    drawing: children;
    /// A paint document: its layer stack, bottom first.
    painting: children;
    /// A photo: its layer stack, bottom first.
    photo: children;
    /// A nest: its sheet, stock, options and parts.
    nest: children;
    /// A drawing's layer (its shapes), or a paint document's pixel layer (its strokes and mask).
    layer: children;
    /// A group: of shapes in a drawing, of layers in a paint document or a photo.
    group: children;
    /// A rectangle: `rect().x(…).y(…).w(…).h(…)`.
    rect: leaf;
    /// An ellipse: `ellipse().cx(…).cy(…).rx(…).ry(…)`.
    ellipse: leaf;
    /// A regular polygon.
    polygon: leaf;
    /// A path: `path().d("M 0 0 L 10 0")`.
    path: leaf;
    /// A text: `text().text("Open studio").x(…).y(…).size(…)`.
    text: leaf;
    /// A drawn brush stroke: `[x, y]` points.
    brush_stroke = "brush-stroke": leaf;
    /// A placed image: `image().src("photo.png")`.
    image: leaf;
    /// A raster layer of a drawing: `raster_layer().src("scan.png")`.
    raster_layer = "raster-layer": leaf;
    /// A drawing's modifier: the one node it changes.
    transform: children;
    /// A drawing's modifier: the one node it changes.
    offset: children;
    /// A drawing's modifier: the one node it changes.
    array: children;
    /// A drawing's modifier: the one node it changes.
    mirror: children;
    /// A drawing's modifier (the one node it changes), or a paint stroke (its mask).
    stroke: children;
    /// A drawing's modifier (the one node it changes), or a stack's colour layer.
    fill: children;
    /// A drawing's modifier: the one node it changes.
    blur: children;
    /// A drawing's modifier (the one node it changes), or a photo's adjustment.
    levels: children;
    /// A drawing's modifier (the one node it changes), or a photo's adjustment.
    threshold: children;
    /// A drawing's modifier: the one node it changes.
    adjust: children;
    /// A drawing's modifier: the one node it changes.
    crop: children;
    /// A drawing's modifier: the one node it changes.
    bucket_fill = "bucket-fill": children;
    /// A boolean of a drawing's shapes.
    boolean: children;
    /// A drawing's clip: its content, then its mask.
    clip: children;
    /// A photo's pixel layer: `raster([filters]).src("photo.png")`.
    raster: children;
    /// A photo's gradient layer.
    gradient: children;
    /// The one layer that masks a layer, a stroke or a filter (`.enabled(false)`: turned off).
    mask: children;
    /// A painting's line layer: `line().from(json("[0, 0]")).to(json("[40, 30]"))`.
    line: leaf;
    /// A painting's Paint Bucket on a layer: `bucket().x(…).y(…).tolerance(32).color(…)`.
    bucket: leaf;
    /// A painting's Gradient on a layer.
    gradient_fill = "gradientFill": leaf;
    /// A painting's Move of a layer's pixels (or its selected pixels).
    move_ = "move": leaf;
    /// A painting's type layer: `text_layer([]).text("Title").x(…).y(…).size(96)`.
    text_layer = "textLayer": children;
    /// A painting layer's styles: `fx([drop_shadow().distance(8), stroke([]).size(3)])`.
    fx: children;
    /// A layer style: a shadow under the layer.
    drop_shadow = "dropShadow": leaf;
    /// A layer style: a shadow inside the layer's edge.
    inner_shadow = "innerShadow": leaf;
    /// A layer style: a glow around the layer.
    outer_glow = "outerGlow": leaf;
    /// A layer style: the layer's colour replaced.
    color_overlay = "colorOverlay": leaf;
    /// A layer style: a gradient over the layer.
    gradient_overlay = "gradientOverlay": leaf;
    /// A photo's adjustment.
    exposure: children;
    /// A photo's adjustment.
    curves: children;
    /// A photo's adjustment.
    hsl: children;
    /// A photo's adjustment.
    vibrance: children;
    /// A photo's adjustment.
    color_balance = "colorBalance": children;
    /// A photo's adjustment.
    black_white = "blackWhite": children;
    /// A photo's adjustment.
    invert: children;
    /// A photo's adjustment.
    posterize: children;
    /// A photo's adjustment: the develop (light, colour, effects) in one tag.
    develop: children;
    /// A photo's filter on a raster's own pixels.
    gaussian_blur = "gaussianBlur": children;
    /// A photo's filter on a raster's own pixels.
    unsharp_mask = "unsharpMask": children;
    /// A photo's filter on a raster's own pixels.
    sharpen: children;
    /// A photo's filter on a raster's own pixels.
    noise: children;
    /// A nest's sheet.
    sheet: leaf;
    /// A nest's stock.
    stock: leaf;
    /// A nest's options.
    options: leaf;
    /// A nest's part: `part().id("tab").quantity(2).outline([[0, 0], [10, 0], [10, 5]])`.
    part: leaf;
}

attributes! {
    /// The 2D documents' attributes: each is the node input of its TypeScript name.
    pub trait TwodAttrs {
        /// The node's id (a nest's part: the name its placements use).
        id;
        /// The node's label.
        label;
        /// The node is disabled.
        disabled;
        /// A layer's or a document's name.
        name;
        /// Shown.
        visible;
        /// Opacity, 0 to 1.
        opacity;
        /// A layer's blend mode.
        blend;
        /// A layer clips to the layer below.
        clip;
        /// A layer is locked.
        locked;
        /// A group is collapsed.
        collapsed;
        /// Wide (a document, a layer).
        width;
        /// High (a document, a layer).
        height;
        /// A document's background.
        background;
        /// A document's dots per inch.
        dpi;
        /// Across.
        x;
        /// Down.
        y;
        /// Wide (a shape).
        w;
        /// High (a shape).
        h;
        /// A corner radius, or a radius across.
        rx;
        /// A radius down.
        ry;
        /// A centre across.
        cx;
        /// A centre down.
        cy;
        /// Rotation, in degrees.
        rotation;
        /// A polygon's sides.
        sides;
        /// A radius.
        radius;
        /// A path's SVG data (absolute M L H V C Q Z).
        d;
        /// A path's subpaths (written as `d`).
        subpaths;
        /// A stroke's points.
        points;
        /// A text's text.
        text;
        /// A text's font family.
        family;
        /// A type layer's face.
        font;
        /// A text's size.
        size;
        /// A text's weight.
        weight;
        /// A text's alignment.
        align;
        /// A text's letter spacing.
        letter_spacing;
        /// A text's line height.
        line_height;
        /// A fill.
        fill;
        /// A stroke.
        stroke;
        /// A stroke's colour.
        stroke_color;
        /// A stroke's width.
        stroke_width;
        /// A stroke's start cap.
        start_cap;
        /// A stroke's end cap.
        end_cap;
        /// The image file, by its path relative to this file.
        src;
        /// A transform.
        transform;
        /// A scale across.
        scale_x;
        /// A scale down.
        scale_y;
        /// A move across.
        dx;
        /// A move down.
        dy;
        /// An offset's distance.
        delta;
        /// An array's count.
        count;
        /// An array's count across.
        count_x;
        /// An array's count down.
        count_y;
        /// An array's sweep.
        sweep;
        /// A mirror's axis.
        axis;
        /// A boolean's operation.
        op;
        /// A kind.
        kind;
        /// A brush stroke erases.
        erase;
        /// A brush's texture.
        texture;
        /// A bucket fill's tolerance.
        tolerance;
        /// A bucket fill or a magic wand reaches only the like pixels joined to the seed.
        contiguous;
        /// The region an operation of a painting changed: `json(r#"[{"rect": [10, 10, 40, 30], "feather": 4}]"#)`.
        selection;
        /// A colour.
        color;
        /// Brightness.
        brightness;
        /// Contrast.
        contrast;
        /// Exposure.
        exposure;
        /// Gamma.
        gamma;
        /// Hue.
        hue;
        /// Saturation.
        saturation;
        /// Lightness.
        lightness;
        /// Temperature.
        temperature;
        /// Vibrance.
        vibrance;
        /// Levels: the input black.
        in_black;
        /// Levels: the input white.
        in_white;
        /// Levels: the output black.
        out_black;
        /// Levels: the output white.
        out_white;
        /// A threshold's level.
        level;
        /// A posterize's levels.
        levels;
        /// Spacing.
        spacing;
        /// An asset.
        asset;
        /// Annotations.
        annotations;
        /// Where.
        at;
        /// A drawing's scale.
        drawing_scale;
        /// A connector's end.
        from_end;
        /// A connector's end.
        to_end;
        /// A connector's routing.
        routing;
        /// A reference.
        ref_;
        /// A sketch.
        sketch;
        /// A unit.
        unit;
        /// A paint stroke's brush: `json(r#"{"kind": "round", "size": 4}"#)`.
        brush;
        /// A paint stroke's symmetry.
        mirror;
        /// A gradient's shape: "linear" or "radial".
        shape;
        /// A gradient's start.
        from;
        /// A gradient's end.
        to;
        /// A gradient's stops.
        stops;
        /// Exposure in stops.
        ev;
        /// An exposure's offset.
        offset;
        /// A levels' channel.
        channel;
        /// A curve of all channels.
        rgb;
        /// A curve of red.
        r;
        /// A curve of green.
        g;
        /// A curve of blue.
        b;
        /// A colour balance's shadows.
        shadows;
        /// A colour balance's midtones.
        midtones;
        /// A colour balance's highlights.
        highlights;
        /// A colour balance keeps the luminosity.
        preserve_luminosity;
        /// Black and white: red.
        red;
        /// Black and white: yellow.
        yellow;
        /// Black and white: green.
        green;
        /// Black and white: cyan.
        cyan;
        /// Black and white: blue.
        blue;
        /// Black and white: magenta.
        magenta;
        /// A filter's amount.
        amount;
        /// A filter's threshold.
        threshold;
        /// Noise is monochrome.
        monochrome;
        /// Noise's seed.
        seed;
        /// A develop's tint.
        tint;
        /// A develop's whites.
        whites;
        /// A develop's blacks.
        blacks;
        /// A develop's dehaze.
        dehaze;
        /// A develop's vignette.
        vignette;
        /// A develop's grain.
        grain;
        /// A develop's clarity (local contrast).
        clarity;
        /// A develop's tone curve.
        curve;
        /// A develop's sections turned off: `["light", "color", "effects"]`.
        off;
        /// A mask is on.
        enabled;
        /// A style's angle, in degrees.
        angle;
        /// A shadow's distance.
        distance;
        /// A shadow's or a glow's spread.
        spread;
        /// An inner shadow's choke.
        choke;
        /// A stroke style's position: "outside", "inside" or "center".
        position;
        /// A nest's safe height (mm).
        safe_z_mm;
        /// A nest's overrides.
        overrides;
        /// A sheet's width (mm).
        width_mm;
        /// A sheet's height (mm).
        height_mm;
        /// A sheet's margin (mm).
        margin_mm;
        /// A stock's material.
        material_id;
        /// A stock's thickness (mm).
        thickness_mm;
        /// A stock's machine: `json(r#"{"powerW": 40}"#)`.
        machine;
        /// The nest's resolution (mm).
        resolution_mm;
        /// The kerf (mm).
        kerf_mm;
        /// The spacing between parts (mm).
        spacing_mm;
        /// The rotations a part may take (degrees).
        rotations;
        /// The most sheets.
        max_sheets;
        /// How many of a part.
        quantity;
        /// A part's outline: `[x, y]` points.
        outline;
        /// A part's holes: lists of `[x, y]` points.
        holes;
        /// The file a nest's part was imported from: `json(r#"{"file": "disc.svg"}"#)`.
        source;
    }
}

pub(crate) const FAMILY: Family = Family { module: "twod", roots: &["drawing", "painting", "photo", "nest"], declare };

/// A painting layer's styles: the tags of its `fx`.
pub const LAYER_STYLES: &[&str] = &["dropShadow", "innerShadow", "outerGlow", "stroke", "colorOverlay", "gradientOverlay"];

/// A photo's adjustments: each a layer whose attributes are the adjustment.
pub const PHOTO_ADJUSTMENTS: &[&str] = &["exposure", "levels", "curves", "hsl", "vibrance", "colorBalance", "blackWhite", "invert", "threshold", "posterize", "develop"];
/// A photo's filters: each on a raster's own pixels.
pub const PHOTO_FILTERS: &[&str] = &["gaussianBlur", "unsharpMask", "sharpen", "noise"];

/// The name of a document whose root names none: the file's, as TypeScript names it ("Drawing" with no file).
fn unnamed() -> String {
    super::source::stem().unwrap_or_else(|| "Drawing".into())
}

fn declare(root: El) -> Result<Declared, String> {
    let mut d = Doc::default();
    let graph = match root.tag {
        "drawing" => d.drawing(&root)?,
        "painting" => d.stack_document(&PAINT, &root)?,
        "photo" => d.stack_document(&PHOTO, &root)?,
        _ => d.nest(&root)?,
    };
    Ok(Declared::Graph(graph))
}

fn place(el: &El) -> String {
    call(el, &["name"])
}

/// A value is plain data with finite numbers.
fn plain(v: &Json, what: &str) -> Result<(), String> {
    match v {
        Json::Num(n) if !n.is_finite() => Err(format!("{what}: {n} is not a finite number")),
        Json::Arr(items) => items.iter().enumerate().try_for_each(|(i, x)| plain(x, &format!("{what}[{i}]"))),
        Json::Obj(entries) => entries.iter().try_for_each(|(k, x)| plain(x, &format!("{what}.{k}"))),
        _ => Ok(()),
    }
}

/// An element's attributes as the node's inputs, without the node's own fields (`id`, `label`, `disabled`).
fn attrs(el: &El, skip: &[&str]) -> Result<Json, String> {
    let mut out = Json::obj();
    for (k, v) in &el.attrs {
        if ["id", "label", "disabled"].contains(&k.as_str()) || skip.contains(&k.as_str()) {
            continue;
        }
        plain(v, &format!("{} {}", place(el), rust_name(k)))?;
        out.set(k, v.clone());
    }
    Ok(out)
}

fn entries(j: &Json) -> &[(String, Json)] {
    match j {
        Json::Obj(e) => e,
        _ => &[],
    }
}

/// `inputs` with `more` written over it, as JavaScript's spread writes one object over another.
fn spread(mut inputs: Json, more: impl IntoIterator<Item = (String, Json)>) -> Json {
    for (k, v) in more {
        inputs.set(&k, v);
    }
    inputs
}

fn wires(set: &str, ids: &[String]) -> Vec<(String, Json)> {
    channels(set, ids.iter().map(|id| wire(id, "out")).collect())
}

// ── Drawing ─────────────────────────────────────────────────────────────────────────────────────────────────────

const DRAW_SOURCES: &[&str] = &["rect", "ellipse", "polygon", "path", "text", "brush-stroke"];
/// Modifiers: the one node each takes is its child, on port `in`.
const DRAW_MODIFIERS: &[&str] = &["transform", "offset", "array", "mirror", "stroke", "fill", "blur", "levels", "threshold", "adjust", "crop", "bucket-fill"];
/// Pixels a drawing places: each names its image file (`src`).
const DRAW_PIXELS: &[&str] = &["image", "raster-layer"];
const DRAW_LATER: &[&str] = &["sketch", "connector", "draw.instance"];

/// The tokens of an SVG path's `d`: its command letters and its numbers (`-?(\d+\.?\d*|\.\d+)(e[-+]?\d+)?`).
fn path_tokens(d: &str) -> Vec<String> {
    let s: Vec<char> = d.chars().collect();
    let digit = |i: usize| s.get(i).is_some_and(|c| c.is_ascii_digit());
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if "MLHVCQZmlhvcqz".contains(s[i]) {
            out.push(s[i].to_string());
            i += 1;
            continue;
        }
        let start = i;
        let mut j = if s[i] == '-' { i + 1 } else { i };
        let number = if digit(j) {
            while digit(j) {
                j += 1;
            }
            if s.get(j) == Some(&'.') {
                j += 1;
                while digit(j) {
                    j += 1;
                }
            }
            true
        } else if s.get(j) == Some(&'.') && digit(j + 1) {
            j += 1;
            while digit(j) {
                j += 1;
            }
            true
        } else {
            false
        };
        if !number {
            i += 1;
            continue;
        }
        if s.get(j) == Some(&'e') {
            let k = if matches!(s.get(j + 1), Some('-' | '+')) { j + 2 } else { j + 1 };
            if digit(k) {
                j = k;
                while digit(j) {
                    j += 1;
                }
            }
        }
        out.push(s[start..j].iter().collect());
        i = j;
    }
    out
}

/// An SVG path's `d` (absolute M L H V C Q Z) as the editor's subpaths.
pub fn subpaths_of(d: &str, what: &str) -> Result<Json, String> {
    let tokens = path_tokens(d);
    let is_cmd = |t: &str| t.chars().all(|c| c.is_ascii_alphabetic());
    let pt = |x: f64, y: f64| Json::obj().with("x", x).with("y", y);
    let mut out: Vec<Json> = Vec::new();
    let mut i = 0;
    let mut cmd = String::new();
    let (mut cx, mut cy) = (0.0, 0.0);
    let n = |i: &mut usize| -> Result<f64, String> {
        match tokens.get(*i) {
            Some(t) if !is_cmd(t) => {
                *i += 1;
                t.parse::<f64>().map_err(|_| format!("{what}: {t} is not a number"))
            }
            _ => Err(format!("{what}: d ends where a number is needed")),
        }
    };
    let segs = |out: &mut Vec<Json>| -> Result<(), String> {
        if out.is_empty() {
            Err(format!("{what}: d starts with M"))
        } else {
            Ok(())
        }
    };
    let push = |out: &mut Vec<Json>, seg: Json| {
        if let Some(Json::Arr(s)) = out.last_mut().and_then(|sp| match sp {
            Json::Obj(e) => e.iter_mut().find(|(k, _)| k == "segs").map(|(_, v)| v),
            _ => None,
        }) {
            s.push(seg);
        }
    };
    while i < tokens.len() {
        if is_cmd(&tokens[i]) {
            cmd = tokens[i].clone();
            i += 1;
        }
        if cmd != cmd.to_uppercase() {
            return Err(format!("{what}: d is written in absolute commands (M L H V C Q Z), not {cmd}"));
        }
        match cmd.as_str() {
            "M" => {
                (cx, cy) = (n(&mut i)?, n(&mut i)?);
                out.push(Json::obj().with("start", pt(cx, cy)).with("segs", Json::Arr(Vec::new())));
                cmd = "L".into();
            }
            "L" => {
                (cx, cy) = (n(&mut i)?, n(&mut i)?);
                segs(&mut out)?;
                push(&mut out, Json::obj().with("to", pt(cx, cy)));
            }
            "H" => {
                cx = n(&mut i)?;
                segs(&mut out)?;
                push(&mut out, Json::obj().with("to", pt(cx, cy)));
            }
            "V" => {
                cy = n(&mut i)?;
                segs(&mut out)?;
                push(&mut out, Json::obj().with("to", pt(cx, cy)));
            }
            "C" => {
                let c1 = pt(n(&mut i)?, n(&mut i)?);
                let c2 = pt(n(&mut i)?, n(&mut i)?);
                (cx, cy) = (n(&mut i)?, n(&mut i)?);
                segs(&mut out)?;
                push(&mut out, Json::obj().with("c1", c1).with("c2", c2).with("to", pt(cx, cy)));
            }
            "Q" => {
                let c1 = pt(n(&mut i)?, n(&mut i)?);
                (cx, cy) = (n(&mut i)?, n(&mut i)?);
                segs(&mut out)?;
                push(&mut out, Json::obj().with("c1", c1).with("to", pt(cx, cy)));
            }
            // A Z closes the subpath once: a number after it names no command (TypeScript would loop on it).
            "Z" if i >= tokens.len() || is_cmd(&tokens[i]) => {
                segs(&mut out)?;
                let last = out.last_mut().expect("a subpath");
                last.set("closed", true);
                let start = last.get("start").cloned().unwrap_or(Json::Null);
                cx = start.get("x").and_then(Json::as_f64).unwrap_or(0.0);
                cy = start.get("y").and_then(Json::as_f64).unwrap_or(0.0);
            }
            _ => return Err(format!("{what}: d has no command before {}", tokens[i])),
        }
    }
    Ok(Json::Arr(out))
}

fn pairs(v: &Json, what: &str) -> Result<Json, String> {
    let Json::Arr(points) = v else { return Err(format!("{what} is a list of [x, y] points")) };
    let out = points.iter().enumerate().map(|(i, p)| match p {
        Json::Arr(c) if c.len() == 2 && c.iter().all(|x| matches!(x, Json::Num(_))) => Ok(Json::obj().with("x", c[0].clone()).with("y", c[1].clone())),
        _ => Err(format!("{what}[{i}] is [x, y]")),
    });
    Ok(Json::Arr(out.collect::<Result<_, _>>()?))
}

const MIME: &[(&str, &str)] = &[("png", "image/png"), ("jpg", "image/jpeg"), ("jpeg", "image/jpeg"), ("webp", "image/webp"), ("gif", "image/gif"), ("bmp", "image/bmp")];

/// A pixel layer's `src`: the image file it names, as the document's own file names it.
fn asset(src: Option<&Json>, el: &El) -> Result<Json, String> {
    let src = match src {
        Some(Json::Str(s)) if !s.is_empty() && !s.starts_with("data:") => s,
        _ => return Err(format!("{}: src names an image file by relative path (\"scan.png\"); pixels are not written in code", place(el))),
    };
    let scheme = src.find(':').is_some_and(|i| i > 0 && src[..i].bytes().all(|b| b.is_ascii_alphabetic()));
    if src.starts_with('/') || scheme {
        return Err(format!("{}: src is a path relative to this file, not {src}", place(el)));
    }
    let ext = src.rsplit('.').next().unwrap_or("").to_lowercase();
    let mut a = Json::obj().with("kind", "file").with("$file", src.as_str());
    if let Some((_, mime)) = MIME.iter().find(|(e, _)| *e == ext) {
        a.set("mime", *mime);
    }
    Ok(a)
}

/// A paint stroke's point tuples as the editor's points.
fn stroke_points(v: &Json, what: &str) -> Result<Json, String> {
    let Json::Arr(points) = v else { return Err(format!("{what} is a list of [x, y, pressure, t] points")) };
    let out = points.iter().enumerate().map(|(i, p)| match p {
        Json::Arr(c) if (c.len() == 4 || c.len() == 6) && c.iter().all(|x| matches!(x, Json::Num(_))) => {
            let mut q = Json::obj().with("x", c[0].clone()).with("y", c[1].clone()).with("pressure", c[2].clone()).with("t", c[3].clone());
            if c.len() == 6 {
                q = q.with("tiltX", c[4].clone()).with("tiltY", c[5].clone());
            }
            Ok(q)
        }
        _ => Err(format!("{what}[{i}] is [x, y, pressure, t] or [x, y, pressure, t, tiltX, tiltY]")),
    });
    Ok(Json::Arr(out.collect::<Result<_, _>>()?))
}

// ── Paint and photo: a layer stack ─────────────────────────────────────────────────────────────────────────────

/// The fields every layer of a stack carries (a chain's top stands in for its layer in the stack).
const COMMON: &[&str] = &["name", "visible", "opacity", "blend", "clip", "locked"];

struct StackKind {
    root: &'static str,
    prefix: &'static str,
    /// A chain member's node type.
    chain_type: fn(&El) -> String,
    /// A layer's tag → its node type and inputs; none when the tag is not a layer.
    layer: fn(&El) -> Result<Option<(String, Json)>, String>,
    /// A chain member's tag (a stroke, a filter) → its own inputs; none when the tag is not one.
    chain: fn(&El) -> Result<Option<Json>, String>,
}

fn pixel_layer(el: &El, kind: &str) -> Result<Option<(String, Json)>, String> {
    let mut a = attrs(el, &["src"])?;
    if el.attr("src").is_some() {
        a.set("__asset", asset(el.attr("src"), el)?);
    }
    Ok(Some((kind.to_string(), a)))
}

fn common_refused(el: &El, a: &Json, why: &str) -> Result<(), String> {
    match COMMON.iter().find(|k| a.get(k).is_some()) {
        Some(k) => Err(format!("{}: {k} is the layer's ({why})", place(el))),
        None => Ok(()),
    }
}

/// An adjustment tag as a layer: the layer's own fields, and the rest as its `adjustment`.
fn adjustment_layer(el: &El, node_type: &str) -> Result<Option<(String, Json)>, String> {
    let mut common = Json::obj();
    let mut adjustment = Json::obj().with("type", el.tag);
    for (k, v) in entries(&attrs(el, &[])?) {
        if COMMON.contains(&k.as_str()) { &mut common } else { &mut adjustment }.set(k, v.clone());
    }
    Ok(Some((node_type.into(), common.with("adjustment", adjustment))))
}

/// What a painting chains on a pixel layer besides its strokes: the Paint Bucket, the Gradient, the Move tool.
pub const PAINT_CHAIN: &[&str] = &["stroke", "bucket", "gradientFill", "move"];
/// A painting's shape layers.
pub const PAINT_SHAPES: &[&str] = &["rect", "ellipse", "polygon", "line"];

const PAINT: StackKind = StackKind {
    root: "painting",
    prefix: "paint",
    chain_type: |el| format!("paint.{}", el.tag),
    layer: |el| match el.tag {
        "layer" => pixel_layer(el, "paint.layer"),
        "fill" | "group" => Ok(Some((format!("paint.{}", el.tag), attrs(el, &[])?))),
        "textLayer" => {
            let a = attrs(el, &[])?;
            if a.get("text").and_then(Json::as_str).is_none() {
                return Err(format!("{}: text is the layer's words (.text(\"Title\"))", place(el)));
            }
            Ok(Some(("paint.text".into(), a)))
        }
        t if PHOTO_ADJUSTMENTS.contains(&t) => adjustment_layer(el, "paint.adjust"),
        t if PAINT_SHAPES.contains(&t) => Ok(Some(("paint.shape".into(), attrs(el, &[])?.with("shape", el.tag)))),
        _ => Ok(None),
    },
    chain: |el| {
        if !PAINT_CHAIN.contains(&el.tag) {
            return Ok(None);
        }
        let mut a = attrs(el, &[])?;
        common_refused(el, &a, "write it on the layer it is painted on")?;
        if let (true, Some(points)) = (el.tag == "stroke", a.get("points")) {
            let points = stroke_points(points, &format!("{} points", place(el)))?;
            a.set("points", points);
        }
        Ok(Some(a))
    },
};

const PHOTO: StackKind = StackKind {
    root: "photo",
    prefix: "photo",
    chain_type: |_| "photo.filter".into(),
    layer: |el| match el.tag {
        "raster" => pixel_layer(el, "photo.raster"),
        "fill" | "gradient" | "group" => Ok(Some((format!("photo.{}", el.tag), attrs(el, &[])?))),
        t if PHOTO_ADJUSTMENTS.contains(&t) => adjustment_layer(el, "photo.adjust"),
        _ => Ok(None),
    },
    chain: |el| {
        if !PHOTO_FILTERS.contains(&el.tag) {
            return Ok(None);
        }
        let a = attrs(el, &[])?;
        common_refused(el, &a, "write it on the layer it filters")?;
        Ok(Some(Json::obj().with("filter", spread(Json::obj().with("type", el.tag), entries(&a).to_vec()))))
    },
};

// ── The declaration ────────────────────────────────────────────────────────────────────────────────────────────

/// One 2D graph being declared: the scope, and what is written on its nodes after they are added.
#[derive(Default)]
struct Doc {
    s: Option<Scope>,
    /// Nodes that are disabled.
    disabled: Vec<String>,
    /// What a node holds that is not a node (its mask element, its styles): its `meta.sources`.
    sources: Vec<(String, Json)>,
}

impl Doc {
    fn scope(&mut self) -> &mut Scope {
        self.s.as_mut().expect("a scope")
    }

    /// Declare `el` as one node of `kind`.
    fn add(&mut self, el: &El, kind: &str, inputs: Json) -> Result<String, String> {
        let id = match el.attr("id") {
            None => None,
            Some(Json::Str(s)) => Some(s.as_str()).filter(|s| !s.is_empty()),
            Some(_) => return Err(format!("{}: id is a string", place(el))),
        };
        let label = el.attr("label").and_then(Json::as_str);
        let Json::Obj(inputs) = inputs else { unreachable!("inputs are an object") };
        let nid = self.scope().add(kind, inputs, id, label, meta(el))?;
        if el.attr("disabled") == Some(&Json::Bool(true)) {
            self.disabled.push(nid.clone());
        }
        Ok(nid)
    }

    /// The graph: `{id, nodes, outputs, meta}`, with the fields written after each node was added.
    fn build(&mut self, output: &str) -> Json {
        let g = self.s.take().expect("a scope").build();
        let mut nodes = g.get("nodes").cloned().unwrap_or(Json::obj());
        if let Json::Obj(entries) = &mut nodes {
            for (id, node) in entries.iter_mut() {
                if self.disabled.contains(id) {
                    node.set("disabled", true);
                }
                if let Some((_, at)) = self.sources.iter().find(|(m, _)| m == id) {
                    let m = node.remove("meta").unwrap_or(Json::obj()).with("sources", at.clone());
                    node.set("meta", m);
                }
            }
        }
        Json::obj()
            .with("id", g.get("id").cloned().unwrap_or(Json::Null))
            .with("nodes", nodes)
            .with("outputs", Json::Arr(vec![output.into()]))
            .with("meta", g.get("meta").cloned().unwrap_or(Json::obj()))
    }

    fn drawn(&mut self, el: &El, top: bool) -> Result<String, String> {
        let t = el.tag;
        let kids: Vec<&El> = el.child_elements().collect();
        if t == "layer" && !top {
            return Err(format!("{}: a layer() is a child of the drawing(); inside it, group with group()", place(el)));
        }
        if t != "layer" && top {
            return Err(format!("{} is inside a layer() (a drawing's children are its layers)", tag(t)));
        }
        if t == "layer" || t == "group" {
            let children = kids.iter().map(|c| self.drawn(c, false)).collect::<Result<Vec<_>, _>>()?;
            return self.add(el, "group", spread(attrs(el, &[])?, wires("children", &children)));
        }
        if DRAW_SOURCES.contains(&t) {
            if !kids.is_empty() {
                return Err(format!("{} takes no children", place(el)));
            }
            let mut a = attrs(el, &[])?;
            if t == "path" {
                if a.get("subpaths").is_some() {
                    return Err(format!("{}: a path is written with d (an SVG path), not subpaths", place(el)));
                }
                if let Some(d) = a.remove("d") {
                    let Json::Str(d) = d else { return Err(format!("{}: d is an SVG path string", place(el))) };
                    a.set("subpaths", subpaths_of(&d, &place(el))?);
                }
            }
            if t == "brush-stroke" {
                if let Some(points) = a.get("points") {
                    let points = pairs(points, &format!("{} points", place(el)))?;
                    a.set("points", points);
                }
            }
            return self.add(el, t, a);
        }
        if DRAW_PIXELS.contains(&t) {
            if !kids.is_empty() {
                return Err(format!("{} takes no children", place(el)));
            }
            if el.attr("src").is_none() {
                return Err(format!("{}: src names its image file by relative path (\"photo.png\")", place(el)));
            }
            let a = attrs(el, &["src"])?.with("__asset", asset(el.attr("src"), el)?);
            return self.add(el, t, a);
        }
        if DRAW_MODIFIERS.contains(&t) {
            if kids.len() != 1 {
                return Err(format!("{} wraps the one node it changes", place(el)));
            }
            let a = attrs(el, &[])?;
            let input = self.drawn(kids[0], false)?;
            return self.add(el, t, a.with("in", wire(&input, "out")));
        }
        if t == "boolean" {
            let a = attrs(el, &[])?;
            let shapes = kids.iter().map(|c| self.drawn(c, false)).collect::<Result<Vec<_>, _>>()?;
            return self.add(el, t, spread(a, wires("shapes", &shapes)));
        }
        if t == "clip" {
            if kids.len() != 2 {
                return Err(format!("{} takes its content, then its mask", place(el)));
            }
            let a = attrs(el, &[])?;
            let content = self.drawn(kids[0], false)?;
            let mask = self.drawn(kids[1], false)?;
            return self.add(el, t, a.with("content", wire(&content, "out")).with("mask", wire(&mask, "out")));
        }
        if DRAW_LATER.contains(&t) {
            return Err(format!("{} is not declared in code yet (a drawing in code holds shapes, images, groups and modifiers)", tag(t)));
        }
        Err(format!("{} is not read in a drawing (see commandagi::design::twod)", tag(t)))
    }

    fn drawing(&mut self, root: &El) -> Result<Json, String> {
        let a = attrs(root, &["name", "width", "height", "background"])?;
        if let Some((k, _)) = entries(&a).first() {
            return Err(format!("drawing(): {} is not read (a drawing has name, width, height, background)", rust_name(k)));
        }
        // A drawing with no name of its own declares none; the file's name only names the graph.
        let own = root.attr("name").and_then(Json::as_str);
        let mut m = Json::obj();
        if let Some(own) = own {
            m.set("name", own);
        }
        for k in ["width", "height", "background"] {
            if let Some(v) = root.attr(k) {
                plain(v, &format!("drawing() {k}"))?;
                m.set(k, v.clone());
            }
        }
        self.s = Some(Scope::new(&format!("draw:{}", slug(&own.map(str::to_string).unwrap_or_else(unnamed))), m.clone()));
        let layers = root.child_elements().map(|c| self.drawn(c, true)).collect::<Result<Vec<_>, _>>()?;
        let mut inputs = Vec::new();
        if let Some(bg) = m.get("background") {
            inputs.push(("background".to_string(), bg.clone()));
        }
        inputs.extend(wires("layers", &layers));
        let comp = self.scope().add("composite", inputs, Some("composite"), Some("Output"), meta(root))?;
        Ok(self.build(&comp))
    }

    /// A layer's (or a stroke's, or a filter's) `mask`: the one stack layer it holds, declared outside the stack and
    /// wired to the node's `mask` port. The `mask` element is not a node: its call is the masked node's
    /// `meta.sources.mask`.
    fn mask_of<'a>(&mut self, kind: &StackKind, el: &'a El) -> Result<Option<(String, &'a El)>, String> {
        let masks: Vec<&El> = el.child_elements().filter(|c| c.tag == "mask").collect();
        let Some(m) = masks.first() else { return Ok(None) };
        if masks.len() > 1 {
            return Err(format!("{} has one mask()", place(el)));
        }
        for (k, v) in entries(&attrs(m, &[])?) {
            if k != "enabled" || !matches!(v, Json::Bool(_)) {
                return Err(format!("mask() in {} has one attribute, enabled (write the rest on the layer it holds)", place(el)));
            }
        }
        let held: Vec<&El> = m.child_elements().collect();
        if held.len() != 1 {
            return Err(format!("mask() in {} holds one layer", place(el)));
        }
        let ids = self.stack(kind, &held)?;
        Ok(Some((ids[0].clone(), m)))
    }

    /// A painting layer's `fx`: its styles in order (the layer's `fx`), and where each was written.
    fn styles_of(kind: &StackKind, el: &El) -> Result<Option<(Json, Json)>, String> {
        let blocks: Vec<&El> = el.child_elements().filter(|c| c.tag == "fx").collect();
        let Some(block) = blocks.first() else { return Ok(None) };
        if kind.prefix != "paint" {
            return Err(format!("{}: fx() (layer styles) is a painting's", place(el)));
        }
        if blocks.len() > 1 {
            return Err(format!("{} has one fx()", place(el)));
        }
        if !entries(&attrs(block, &[])?).is_empty() {
            return Err(format!("fx() in {} has no attributes (write them on its styles)", place(el)));
        }
        let mut sources = Json::obj();
        if let Some(at) = block.source_json() {
            sources.set("fx", at);
        }
        let mut fx = Vec::new();
        for (i, c) in block.child_elements().enumerate() {
            if !LAYER_STYLES.contains(&c.tag) {
                return Err(format!("{} is not a layer style ({})", tag(c.tag), LAYER_STYLES.join(", ")));
            }
            if c.child_elements().next().is_some() {
                return Err(format!("{} in fx() takes no children", place(c)));
            }
            if let Some(at) = c.source_json() {
                sources.set(&format!("fx.{i}"), at);
            }
            fx.push(spread(Json::obj().with("type", c.tag), entries(&attrs(c, &[])?).to_vec()));
        }
        Ok(Some((Json::Arr(fx), sources)))
    }

    /// Declare `el` as one node of `kind`, masked by its `mask` when it has one, with its `fx` when it has them.
    fn add_masked(&mut self, kind: &StackKind, el: &El, node_type: &str, inputs: Json) -> Result<String, String> {
        let mask = self.mask_of(kind, el)?;
        let styles = Self::styles_of(kind, el)?;
        let mut inputs = match &mask {
            Some((id, m)) => {
                let i = inputs.with("mask", wire(id, "out"));
                if m.attr("enabled") == Some(&Json::Bool(false)) { i.with("maskEnabled", false) } else { i }
            }
            None => inputs,
        };
        let mut sources = Json::obj();
        if let Some((fx, at)) = styles {
            inputs.set("fx", fx);
            sources = at;
        }
        let id = self.add(el, node_type, inputs)?;
        if let Some(at) = mask.and_then(|(_, m)| m.source_json()) {
            sources.set("mask", at);
        }
        if !entries(&sources).is_empty() {
            self.sources.push((id.clone(), sources));
        }
        Ok(id)
    }

    fn stack(&mut self, kind: &StackKind, children: &[&El]) -> Result<Vec<String>, String> {
        let mut slots = Vec::new();
        for &el in children {
            let Some((layer_type, inputs)) = (kind.layer)(el)? else {
                if (kind.chain)(el)?.is_some() {
                    return Err(format!("{} is painted on a layer: write it inside one", place(el)));
                }
                return Err(format!("{} is not read in a {} (see commandagi::design::twod)", tag(el.tag), tag(kind.root)));
            };
            let (mut stack, mut chain) = (Vec::new(), Vec::new());
            for c in el.child_elements().filter(|c| c.tag != "mask" && c.tag != "fx") {
                if (kind.chain)(c)?.is_some() { &mut chain } else { &mut stack }.push(c);
            }
            let group = format!("{}.group", kind.prefix);
            if !stack.is_empty() && layer_type != group {
                return Err(format!("{}: only a group() holds layers", place(el)));
            }
            let inner = if layer_type == group { self.stack(kind, &stack)? } else { Vec::new() };
            let mut top = self.add_masked(kind, el, &layer_type, spread(inputs.clone(), wires("layers", &inner)))?;
            // Each chain member carries the layer's fields: the stack reads them off whichever member is on top.
            let mut carried = Json::obj().with("name", "Layer").with("visible", true).with("opacity", 1i64).with("blend", "normal");
            for k in COMMON {
                if let Some(v) = inputs.get(k) {
                    carried.set(k, v.clone());
                }
            }
            for c in chain {
                let own = (kind.chain)(c)?.expect("a chain member");
                let inputs = spread(carried.clone(), entries(&own).to_vec()).with("src", wire(&top, "out"));
                top = self.add_masked(kind, c, &(kind.chain_type)(c), inputs)?;
            }
            slots.push(top);
        }
        Ok(slots)
    }

    fn stack_document(&mut self, kind: &StackKind, root: &El) -> Result<Json, String> {
        let a = attrs(root, &[])?;
        if let Some((k, _)) = entries(&a).iter().find(|(k, _)| !["name", "width", "height", "background", "dpi"].contains(&k.as_str())) {
            return Err(format!("{}: {} is not read (it has name, width, height, background, dpi)", tag(kind.root), rust_name(k)));
        }
        let title = a.get("name").and_then(Json::as_str).map(str::to_string).unwrap_or_else(unnamed);
        self.s = Some(Scope::new(&format!("{}:{}", kind.prefix, slug(&title)), Json::obj().with("domain", kind.prefix).with("name", title)));
        let kids: Vec<&El> = root.child_elements().collect();
        let layers = self.stack(kind, &kids)?;
        let Json::Obj(inputs) = spread(a, wires("layers", &layers)) else { unreachable!() };
        let doc = self.scope().add(&format!("{}.doc", kind.prefix), inputs, Some("doc"), None, meta(root))?;
        Ok(self.build(&doc))
    }

    // ── Nest ───────────────────────────────────────────────────────────────────────────────────────────────────

    fn nest(&mut self, root: &El) -> Result<Json, String> {
        const ONE: &[&str] = &["sheet", "stock", "options"];
        let a = attrs(root, &["name"])?;
        if let Some((k, _)) = entries(&a).iter().find(|(k, _)| !["safeZMm", "overrides"].contains(&k.as_str())) {
            return Err(format!("nest(): {} is not read (it has safe_z_mm, overrides)", rust_name(k)));
        }
        let title = root.attr("name").and_then(Json::as_str).map(str::to_string).unwrap_or_else(unnamed);
        self.s = Some(Scope::new(&format!("nest:{}", slug(&title)), Json::obj().with("domain", "nest").with("name", title)));
        let mut one: Vec<(String, Json)> = Vec::new();
        let mut parts = Vec::new();
        for el in root.child_elements() {
            if ONE.contains(&el.tag) {
                if one.iter().any(|(k, _)| k == el.tag) {
                    return Err(format!("a nest has one {}", tag(el.tag)));
                }
                let id = self.add(el, &format!("nest.{}", el.tag), attrs(el, &[])?)?;
                one.push((el.tag.to_string(), wire(&id, "out")));
            } else if el.tag == "part" {
                if el.attr("id").and_then(Json::as_str).is_none() {
                    return Err(format!("{}: a part has an id (the name its placements and overrides use)", place(el)));
                }
                parts.push(self.add(el, "nest.part", attrs(el, &[])?)?);
            } else {
                return Err(format!("{} is not read in a nest (it has sheet(), stock(), options(), part())", tag(el.tag)));
            }
        }
        if let Some(k) = ONE.iter().find(|k| !one.iter().any(|(t, _)| t == *k)) {
            return Err(format!("a nest needs its {}", tag(k)));
        }
        let Json::Obj(inputs) = spread(spread(a, one), wires("parts", &parts)) else { unreachable!() };
        let doc = self.scope().add("nest.doc", inputs, Some("nest"), None, meta(root))?;
        Ok(self.build(&doc))
    }
}
