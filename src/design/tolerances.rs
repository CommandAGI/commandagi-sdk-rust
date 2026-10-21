//! TOLERANCES — what a toleranced size allows, and the fit of a hole and a shaft. The same rules and messages as the
//! TypeScript SDK's `tolerances.ts`, the Python SDK's `tolerances.py` and the 3D engine's
//! `packages/domain/3d-core/tolerances.ts`:
//!
//! ```
//! use commandagi::design::{tolerances::*, Json};
//!
//! let hole = tolerance_limits(25.0, &Json::parse(r#"{"class": "H7"}"#).unwrap()).unwrap();
//! let shaft = tolerance_limits(25.0, &Json::parse(r#"{"class": "g6"}"#).unwrap()).unwrap();
//! assert_eq!(fit_of(&hole, &shaft).kind, "clearance");
//! ```
//!
//! A class is any ISO 286 class (`H7`, `e8`, `E10`, `H13`, `zc11`), computed from its fundamental deviation and grade
//! over 0 up to 3150 mm (`cloud://global/standards/iso-286-1-grades.csv`, `iso-286-1-deviations.csv`), a general
//! tolerance of ISO 2768-1 (`ISO 2768-m`), or a bearing ring's ISO 492 Normal class (`ISO 492 bore`, `ISO 492 outside`,
//! `ISO 492 width`); `standards_data.json` `tolerances` holds the tables. A class or a size the standard does not give
//! is refused, with the reason.

use super::json::Json;
use super::standards::standards;

/// What a tolerance allows at a nominal size, in millimetres.
#[derive(Clone, Debug, PartialEq)]
pub struct Limits {
    pub nominal: f64,
    pub upper: f64,
    pub lower: f64,
    pub min: f64,
    pub max: f64,
    pub class: Option<String>,
    pub source: Option<String>,
    pub basic: bool,
}

/// The fit of a hole and a shaft.
#[derive(Clone, Debug, PartialEq)]
pub struct Fit {
    /// `clearance`, `transition` or `interference`.
    pub kind: &'static str,
    pub min_clearance: f64,
    pub max_clearance: f64,
}

fn round6(n: f64) -> f64 {
    (n * 1e6).round() / 1e6 + 0.0
}

fn rows(table: &str) -> &'static [Json] {
    match standards().get("tolerances").and_then(|t| t.get(table)) {
        Some(Json::Arr(a)) => a,
        _ => &[],
    }
}

fn text(row: &Json, key: &str) -> String {
    match row.get(key) {
        Some(Json::Str(s)) => s.clone(),
        Some(Json::Num(n)) => format!("{n}"),
        _ => String::new(),
    }
}

fn num(row: &Json, key: &str) -> f64 {
    row.get(key).and_then(Json::as_f64).unwrap_or(f64::NAN)
}

/// The classes an editor suggests: the usual ISO 286-2 classes, ISO 2768-1 and the ISO 492 rings. Every ISO 286 class
/// is read, not only these.
pub fn tolerance_classes() -> Vec<String> {
    let holes = ["H6", "H7", "H8", "H9", "H10", "H11", "H12", "H13", "G6", "G7", "F7", "F8", "E8", "E9", "E10", "D9", "D10", "C11", "JS6", "JS7", "JS8", "K6", "K7", "M6", "M7", "N6", "N7", "P6", "P7", "R7", "S7", "U7"];
    let shafts = ["h5", "h6", "h7", "h8", "h9", "h10", "h11", "h12", "h13", "g5", "g6", "f6", "f7", "e7", "e8", "e9", "d9", "d10", "c11", "js5", "js6", "js7", "j5", "j6", "k5", "k6", "m5", "m6", "n5", "n6", "p5", "p6", "r6", "s6", "u6"];
    let mut out: Vec<String> = holes.iter().chain(shafts.iter()).map(|c| c.to_string()).collect();
    for r in rows("iso-2768-1") {
        let c = format!("ISO 2768-{}", text(r, "class"));
        if !out.contains(&c) {
            out.push(c);
        }
    }
    for r in rows("iso-492-normal") {
        let c = format!("ISO 492 {}", text(r, "ring"));
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

const ISO286_SOURCE: &str = "ISO 286-1: the fundamental deviation (Tables 2 and 3) and the standard tolerance grade (Table 1)";
const KNOWN: &str = "an ISO 286 class (a hole's letters A to ZC or a shaft's a to zc, then a grade 01, 0 or 1 to 18: H7, e8, JS9), ISO 2768-f, -m, -c or -v, or ISO 492 bore, outside or width";
/// The fundamental deviations' letters, the longer of two that share a start first (CD before C).
const LETTERS: [&str; 28] = ["A", "B", "CD", "C", "D", "EF", "E", "FG", "F", "G", "H", "JS", "J", "K", "M", "N", "P", "R", "S", "T", "U", "V", "X", "Y", "ZA", "ZB", "ZC", "Z"];
/// The shaft letters whose fundamental deviation is the upper one (es); a hole's A to H mirror them (EI = -es).
const UPPER_LETTERS: [&str; 11] = ["a", "b", "c", "cd", "d", "e", "ef", "f", "fg", "g", "h"];

fn row_at(table: &str, size: f64, wanted: impl Fn(&Json) -> bool) -> Option<&'static Json> {
    rows(table).iter().find(|r| wanted(r) && size > num(r, "overMm") && size <= num(r, "uptoMm"))
}

fn has(row: &Json, key: &str) -> bool {
    !num(row, key).is_nan()
}

/// An ISO 286 class's letters and grade, as written ("H7" -> ("H", "7"), "zc11" -> ("zc", "11")).
fn iso286_name(text: &str) -> Option<(String, String)> {
    let upper = text.to_uppercase();
    let hole = text == upper;
    if !hole && text != text.to_lowercase() {
        return None;
    }
    for l in LETTERS {
        if let Some(grade) = upper.strip_prefix(l) {
            let ok = matches!(grade, "01" | "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "12" | "13" | "14" | "15" | "16" | "17" | "18");
            if ok {
                return Some((if hole { l.to_string() } else { l.to_lowercase() }, grade.to_string()));
            }
        }
    }
    None
}

/// An ISO 286 class at a size, in micrometres (the 3D engine's `iso286`, line for line).
fn iso286(letters: &str, grade_text: &str, size: f64) -> Result<(f64, f64), String> {
    let name = format!("{letters}{grade_text}");
    let hole = letters != letters.to_lowercase();
    let l = letters.to_lowercase();
    let grade: i32 = if grade_text == "01" { -1 } else { grade_text.parse().unwrap_or(0) };
    if !(size > 0.0 && size <= 3150.0) {
        return Err(format!("{name}: ISO 286 gives sizes over 0 up to 3150 mm, not {} mm", fmt(size)));
    }
    let key = if grade_text == "01" { "it01".to_string() } else { format!("it{grade_text}") };
    let it = row_at("iso-286-1-grades", size, |_| true).map(|g| num(g, &key)).unwrap_or(f64::NAN);
    if it.is_nan() {
        return Err(format!("{name}: the grade IT{grade_text} is not defined at {} mm (IT01 and IT0 go up to 500 mm)", fmt(size)));
    }
    if grade >= 14 && size <= 1.0 {
        return Err(format!("{name}: the grades IT14 to IT18 are not used at 1 mm and below (ISO 286-1)"));
    }
    if (l == "a" || l == "b") && size <= 1.0 {
        return Err(format!("{name}: the deviations a and b are not used at 1 mm and below (ISO 286-1)"));
    }
    if l == "js" {
        return Ok((it / 2.0, -it / 2.0));
    }
    let d = row_at("iso-286-1-deviations", size, |_| true).ok_or_else(|| format!("{name}: no deviations at {} mm", fmt(size)))?;
    let val = |k: &str| num(d, k);
    let none = || format!("{name}: ISO 286-1 gives no fundamental deviation {letters} at {} mm", fmt(size));
    if UPPER_LETTERS.contains(&l.as_str()) {
        if !has(d, &l) {
            return Err(none());
        }
        return Ok(if hole { (-val(&l) + it, -val(&l)) } else { (val(&l), val(&l) - it) });
    }
    if l == "j" {
        let k = if hole { format!("holeJ{grade}") } else if grade == 5 || grade == 6 { "j5".to_string() } else { format!("j{grade}") };
        if !has(d, &k) {
            let gives = if hole { "J6, J7 and J8" } else { "j5, j6 and j7 (j8 to 3 mm)" };
            return Err(format!("{name}: ISO 286-2 gives {gives} up to 500 mm, not {name} at {} mm", fmt(size)));
        }
        return Ok(if hole { (val(&k), val(&k) - it) } else { (val(&k) + it, val(&k)) });
    }
    if !hole {
        let ei = if l == "k" {
            if (4..=7).contains(&grade) { val("k") } else { 0.0 }
        } else if has(d, &l) {
            val(&l)
        } else {
            return Err(none());
        };
        return Ok((ei + it, ei));
    }
    let delta = if size <= 500.0 && (3..=8).contains(&grade) { val(&format!("delta{grade}")) } else { 0.0 };
    let es = if l == "k" {
        if size <= 500.0 && grade <= 8 { -val("k") + delta } else { 0.0 }
    } else if l == "m" {
        if size > 250.0 && size <= 315.0 && grade == 6 { -9.0 } else { -val("m") + if grade <= 8 { delta } else { 0.0 } }
    } else if l == "n" {
        if size > 3.0 && size <= 500.0 && grade > 8 { 0.0 } else { -val("n") + if grade <= 8 { delta } else { 0.0 } }
    } else {
        if !has(d, &l) {
            return Err(none());
        }
        -val(&l) + if grade <= 7 { delta } else { 0.0 }
    };
    Ok((es, es - it))
}

/// A general tolerance's class letter ("ISO 2768-m", "ISO 2768-1 f"), lower-case.
fn general_class(name: &str) -> Option<String> {
    let rest = name.strip_prefix("ISO").or_else(|| name.strip_prefix("iso"))?.trim_start();
    let rest = rest.strip_prefix("2768")?;
    let rest = rest.strip_prefix("-1").unwrap_or(rest);
    let c = rest.trim_start_matches(|ch: char| ch == '-' || ch.is_whitespace()).to_lowercase();
    ["f", "m", "c", "v"].contains(&c.as_str()).then_some(c)
}

/// A class's deviations at a size, in millimetres, with the source they were read from.
fn class_deviations(name: &str, size: f64) -> Result<(f64, f64, Option<String>), String> {
    if let Some(ring) = bearing_ring(name) {
        let candidates: Vec<&Json> = rows("iso-492-normal").iter().filter(|r| text(r, "ring") == ring).collect();
        let Some(row) = candidates.iter().find(|r| size > num(r, "overMm") && size <= num(r, "uptoMm")) else {
            let lo = candidates.iter().map(|r| num(r, "overMm")).fold(f64::INFINITY, f64::min);
            let hi = candidates.iter().map(|r| num(r, "uptoMm")).fold(f64::NEG_INFINITY, f64::max);
            return Err(format!("{name} is tabled over {} up to {} mm, not at {} mm", fmt(lo), fmt(hi), fmt(size)));
        };
        let source = match row.get("source") {
            Some(Json::Str(s)) => Some(s.clone()),
            _ => None,
        };
        return Ok((num(row, "upperUm") / 1000.0, num(row, "lowerUm") / 1000.0, source));
    }
    if let Some(c) = general_class(name) {
        let row = row_at("iso-2768-1", size, |r| text(r, "class") == c).ok_or_else(|| format!("ISO 2768-{c} gives no deviation at {} mm", fmt(size)))?;
        let dev = num(row, "deviationMm");
        return Ok((dev, -dev, Some(text(row, "source"))));
    }
    let Some((letters, grade)) = iso286_name(name) else {
        return Err(format!("there is no tolerance class \"{name}\": a class is {KNOWN}"));
    };
    let (upper, lower) = iso286(&letters, &grade, size)?;
    Ok((round6(upper / 1000.0), round6(lower / 1000.0), Some(ISO286_SOURCE.to_string())))
}

/// The bearing ring an "ISO 492 bore|outside|width" name says, lower-case.
fn bearing_ring(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.as_slice() {
        [iso, n, ring] if iso.eq_ignore_ascii_case("ISO") && *n == "492" => {
            let r = ring.to_lowercase();
            ["bore", "outside", "width"].contains(&r.as_str()).then_some(r)
        }
        _ => None,
    }
}

/// What is wrong with a tolerance as written, or None.
pub fn tolerance_problem(tol: &Json) -> Option<String> {
    let Json::Obj(entries) = tol else {
        return Some("a tolerance is { upper, lower }, { class } or { basic: true }".into());
    };
    for (k, _) in entries {
        if !["upper", "lower", "class", "basic"].contains(&k.as_str()) {
            return Some(format!("a tolerance has no field {k} (it has upper, lower, class, basic)"));
        }
    }
    let (up, lo, cls, basic) = (tol.get("upper"), tol.get("lower"), tol.get("class"), tol.get("basic"));
    if up.is_some() && !matches!(up, Some(Json::Num(_))) {
        return Some("upper is a number of millimetres".into());
    }
    if lo.is_some() && !matches!(lo, Some(Json::Num(_))) {
        return Some("lower is a number of millimetres".into());
    }
    if cls.is_some() && !matches!(cls, Some(Json::Str(s)) if !s.trim().is_empty()) {
        return Some("class is a name (H7, g6, ISO 492 bore)".into());
    }
    if basic.is_some() && !matches!(basic, Some(Json::Bool(_))) {
        return Some("basic is true or false".into());
    }
    let devs = up.is_some() || lo.is_some();
    let basic = matches!(basic, Some(Json::Bool(true)));
    if basic && (devs || cls.is_some()) {
        return Some("a basic dimension has no tolerance of its own: clear basic, or clear the tolerance".into());
    }
    if devs {
        if let Some(Json::Str(c)) = cls {
            return Some(format!("give the class ({c}) or the deviations, not both"));
        }
        let (Some(Json::Num(u)), Some(Json::Num(l))) = (up, lo) else {
            return Some("give both deviations, upper and lower (0 for none)".into());
        };
        if !(u >= l) {
            return Some(format!("the upper deviation ({u}) is below the lower ({l})"));
        }
    }
    if !devs && cls.is_none() && !basic {
        return Some("an empty tolerance: give upper and lower, or a class, or basic".into());
    }
    None
}

fn fmt(n: f64) -> String {
    format!("{}", (n * 1e4).round() / 1e4 + 0.0)
}

/// The sizes a tolerance allows at a nominal size, or why it cannot say.
pub fn tolerance_limits(nominal: f64, tol: &Json) -> Result<Limits, String> {
    if let Some(p) = tolerance_problem(tol) {
        return Err(p);
    }
    if matches!(tol.get("basic"), Some(Json::Bool(true))) {
        return Ok(Limits { nominal, upper: 0.0, lower: 0.0, min: nominal, max: nominal, class: None, source: None, basic: true });
    }
    if let Some(Json::Str(cls)) = tol.get("class") {
        let name = cls.trim();
        let (upper, lower, source) = class_deviations(name, nominal)?;
        return Ok(Limits { nominal, upper, lower, min: round6(nominal + lower), max: round6(nominal + upper), class: Some(name.to_string()), source, basic: false });
    }
    let u = tol.get("upper").and_then(Json::as_f64).unwrap_or_default();
    let l = tol.get("lower").and_then(Json::as_f64).unwrap_or_default();
    Ok(Limits { nominal, upper: u, lower: l, min: round6(nominal + l), max: round6(nominal + u), class: None, source: None, basic: false })
}

/// The fit of a hole and a shaft: clearance when the smallest hole takes the largest shaft (a minimum clearance of 0
/// counts), interference when the largest hole is smaller than the smallest shaft, else transition.
pub fn fit_of(hole: &Limits, shaft: &Limits) -> Fit {
    let min_c = hole.min - shaft.max;
    let max_c = hole.max - shaft.min;
    let kind = if min_c >= -1e-9 {
        "clearance"
    } else if max_c <= 1e-9 {
        "interference"
    } else {
        "transition"
    };
    Fit { kind, min_clearance: round6(min_c), max_clearance: round6(max_c) }
}
