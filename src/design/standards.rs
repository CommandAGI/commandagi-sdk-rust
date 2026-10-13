//! ENGINEERING TABLES — material grades, standard holes by name and the dimension tables of standard parts. The same
//! data and the same calls as the TypeScript SDK's `standards.ts` and the Python SDK's `standards.py`:
//!
//! ```
//! use commandagi::design::standards::*;
//!
//! assert_eq!(hole_size("M4 close fit").unwrap().get("diameter").and_then(|d| d.as_f64()), Some(4.3)); // ISO 273 fine
//! assert_eq!(hole_size("M4 tap").unwrap().get("diameter").and_then(|d| d.as_f64()), Some(3.3)); // ISO 2306
//! assert_eq!(standard_row("iso-4762", "M4").unwrap().get("dkMax").and_then(|d| d.as_f64()), Some(7.0));
//! assert!(material("6061-T6", &[]).unwrap().get("source").is_some()); // where its numbers were read
//! ```
//!
//! The numbers are data (`standards_data.json`), generated from the shipped tables
//! (`cloud://global/standards/<table>.csv`, `cloud://global/materials/engineering.csv`). Each row of a table names its
//! source, how it was read, by whom and when; `sources` in the data and a material's `source` carry the source's name
//! here.
//!
//! A name the tables do not have is refused with the names they do have. Nothing is computed in its place.
//!
//! A 3D document uses them by name: `hole().size("M4 close fit")` has that diameter, and
//! `body().engineering(json(r#"{"material": "6061-T6"}"#))` has that density, so an observation of the document says
//! the body's mass. A materials table of your own (`materials/<name>.csv` beside the document or in a folder above it)
//! is read before the shipped one: pass its rows as `tables`.

use super::json::Json;
use std::sync::OnceLock;

/// What a hole's `size` says after its thread.
pub const HOLE_FITS: &[&str] = &["close fit", "normal fit", "loose fit", "tap"];

/// The data: `materials` (a row per grade), `holeSizes` (the standard holes of each thread, in millimetres), `tables`
/// (each dimension table by name) and `sources` (where each table's rows were read).
pub fn standards() -> &'static Json {
    static DATA: OnceLock<Json> = OnceLock::new();
    DATA.get_or_init(|| Json::parse(include_str!("standards_data.json")).expect("standards_data.json is JSON"))
}

fn entries(v: Option<&Json>) -> &[(String, Json)] {
    match v {
        Some(Json::Obj(e)) => e,
        _ => &[],
    }
}

fn items(v: Option<&Json>) -> &[Json] {
    match v {
        Some(Json::Arr(a)) => a,
        _ => &[],
    }
}

fn norm(id: &str) -> String {
    id.trim().to_lowercase().split(|c: char| c.is_whitespace() || c == '_').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-")
}

fn id_of(row: &Json) -> &str {
    row.get("id").and_then(Json::as_str).unwrap_or_default()
}

/// The shipped materials table: a row per grade.
pub fn materials() -> &'static [Json] {
    items(standards().get("materials"))
}

/// The row a material id names: the nearest of `tables` (your own tables, nearest first), then the shipped table.
pub fn material(id: &str, tables: &[Json]) -> Result<Json, String> {
    let want = norm(id);
    tables.iter().chain(materials()).find(|m| norm(id_of(m)) == want).cloned().ok_or_else(|| {
        format!("no materials table has a row \"{id}\" (the shipped table has {})", materials().iter().map(id_of).collect::<Vec<_>>().join(", "))
    })
}

/// The mass in grams of a solid of `volume_mm3` made of a material: volume × density (1 g/cm³ is 1 g per 1000 mm³).
pub fn mass_grams(volume_mm3: f64, material_id: &str, tables: &[Json]) -> Result<f64, String> {
    let row = material(material_id, tables)?;
    Ok(volume_mm3 / 1000.0 * row.get("densityGCm3").and_then(Json::as_f64).unwrap_or_default())
}

/// The hole a name says: a thread and a fit, "M4 close fit", "M4 normal fit", "M4 loose fit" (the clearance holes of
/// ISO 273 fine, medium and coarse; ASME B18.2.8 for an inch thread), or "M4 tap" (the drill before tapping):
/// `{thread, fit, diameter, row, source}`, millimetres.
pub fn hole_size(name: &str) -> Result<Json, String> {
    let text = name.split_whitespace().collect::<Vec<_>>().join(" ");
    let (thread, rest) = text.split_once(' ').unwrap_or((text.as_str(), ""));
    let fit = rest.to_lowercase();
    let sizes = entries(standards().get("holeSizes"));
    let Some((_, row)) = sizes.iter().find(|(k, _)| k.to_lowercase() == thread.to_lowercase()) else {
        let known = sizes.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(", ");
        return Err(format!("\"{name}\": no hole table has the thread {} (it has {known})", if thread.is_empty() { "(none)" } else { thread }));
    };
    let row_thread = row.get("thread").and_then(Json::as_str).unwrap_or_default();
    if !HOLE_FITS.contains(&fit.as_str()) {
        return Err(format!("\"{name}\": a hole size is a thread and one of {} (\"{row_thread} close fit\")", HOLE_FITS.join(", ")));
    }
    let source = |what: &str| row.get("sources").and_then(|s| s.get(what)).and_then(Json::as_str).unwrap_or_default().to_string();
    let (diameter, source) = if fit == "tap" {
        match row.get("tap") {
            Some(d) => (d.clone(), source("tap")),
            None => return Err(format!("\"{name}\": the tap drill table has no {row_thread}")),
        }
    } else {
        (row.get(&fit[..fit.len() - 4]).cloned().unwrap_or(Json::Null), source("clearance"))
    };
    Ok(Json::obj().with("thread", row_thread).with("fit", fit.as_str()).with("diameter", diameter).with("row", row.clone()).with("source", source))
}

/// A hole's fields with what its `size` says: the diameter, and for a counterbore or a countersink the head the
/// tables give for that thread (a counterbore for a hexagon socket head cap screw, DIN 974-1; a countersink for a
/// countersunk screw, DIN 74 form F, its depth from its diameter and angle). The size wins: a `diameter`,
/// `headDiameter` or `headDepth` that differs from the size's is refused. Leave it out, or clear `size` to give your
/// own.
pub fn sized_hole(hole: &Json) -> Result<Json, String> {
    let Some(size) = hole.get("size").and_then(Json::as_str) else { return Ok(hole.clone()) };
    let named = hole_size(size)?;
    let diameter = named.get("diameter").and_then(Json::as_f64).unwrap_or_default();
    let mut says: Vec<(&str, f64)> = vec![("diameter", diameter)];
    let tap = named.get("fit").and_then(Json::as_str) == Some("tap");
    let row = named.get("row");
    let style = hole.get("style").and_then(Json::as_str);
    let num = |v: Option<&Json>, k: &str| v.and_then(|j| j.get(k)).and_then(Json::as_f64);
    if let (false, Some("counterbore"), Some(bore)) = (tap, style, row.and_then(|r| r.get("counterbore"))) {
        says.push(("headDiameter", num(Some(bore), "diameter").unwrap_or_default()));
        says.push(("headDepth", num(Some(bore), "depth").unwrap_or_default()));
    } else if let (false, Some("countersink"), Some(sink)) = (tap, style, row.and_then(|r| r.get("countersink"))) {
        let head = num(Some(sink), "diameter").unwrap_or_default();
        let angle = num(Some(sink), "angleDeg").unwrap_or(90.0);
        let depth = (head - diameter) / 2.0 / (angle * std::f64::consts::PI / 360.0).tan();
        says.push(("headDiameter", head));
        says.push(("headDepth", (depth * 1e4 + 0.5).floor() / 1e4));
    }
    let mut out = hole.clone();
    for (field, value) in says {
        if let Some(written) = hole.get(field).and_then(Json::as_f64) {
            if (written - value).abs() > 1e-9 {
                return Err(format!("{field}: \"{size}\" is {} mm (got {}); leave {field} out, or clear size", Json::Num(value).text(), Json::Num(written).text()));
            }
        }
        out.set(field, value);
    }
    Ok(out)
}

/// A row of a dimension table by the value of its first column: `standard_row("iso-4762", "M4")`,
/// `standard_row("bearings", "608")`. The tables are the keys of `standards().get("tables")`.
pub fn standard_row(table: &str, key: &str) -> Result<Json, String> {
    let tables = entries(standards().get("tables"));
    let Some((_, rows)) = tables.iter().find(|(k, _)| k == table) else {
        return Err(format!("no table \"{table}\" (the tables are {})", tables.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(", ")));
    };
    let rows = items(Some(rows));
    let first = |r: &Json| match r {
        Json::Obj(e) => e.first().map(|(_, v)| v.as_str().map(str::to_string).unwrap_or_else(|| v.text())).unwrap_or_default(),
        _ => String::new(),
    };
    rows.iter()
        .find(|r| first(r).to_lowercase() == key.to_lowercase())
        .cloned()
        .ok_or_else(|| format!("table \"{table}\" has no row \"{key}\" (it has {})", rows.iter().map(first).collect::<Vec<_>>().join(", ")))
}
