//! OFFICE DOCUMENTS IN RUST — a workbook (`.sheet.rs`), a page (`.page.rs`) and a deck (`.deck.rs`), declared as the
//! very document the CommandAGI sheets, docs and decks editors open. The same documents, part for part, as the
//! TypeScript SDK's office JSX (`office.ts`) and the Python SDK's `commandagi.design.office`:
//!
//! ```
//! use commandagi::design::office::*;
//!
//! fn document() -> El {
//!     workbook([sheet([
//!         column().at("A").width(160),
//!         cell().at("A1").value("Item").bold(true),
//!         cell().at("B1").value(1200).num_fmt("$#,##0"),
//!         cell().at("B2").formula("=B1*12"),
//!     ])
//!     .name("Q1")])
//!     .title("Budget")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! ```
//! use commandagi::design::office::*;
//!
//! fn document() -> El {
//!     page([
//!         h1("Launch notes"),
//!         p(("The first sentence, with ", b("bold"), " and a ", a("link").href("https://commandagi.com"), ".")),
//!         bullet("a list item"),
//!     ])
//!     .title("Notes")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! The elements (attributes are snake case of the TypeScript names: `.frozen_rows(1)`, `.num_fmt(…)`, `.font_size(…)`):
//!
//! ```text
//! workbook([sheets]).title(…)                                           the workbook
//! sheet([cells]).name .rows .cols .frozen_rows .frozen_cols .color     a grid sheet (200 rows, 26 columns unless it says)
//! cell().at("B4").value(…) | .formula("=…") .num_fmt .bold .italic .align .bg .color .wrap
//! column().at("A").width(…)   row().at(3).height(…)                    a column's width or a row's height, in pixels
//! page([blocks]).title .paper .font                                    the page; paper "letter" or "a4"; font "sans", "serif", "mono"
//! h1 h2 h3 p quote (text).align(…)                                     a text block: its text and marks; align "left", "center" …
//! bullet numbered (text)   todo(text).checked(true)                    a list item, a checklist item
//! pre("code").lang(…)   divider()   image().src(…).alt(…)              a code block, a rule, a picture
//! b i u s code (text)   a(text).href(…)   br()                          marks inside a text
//! deck([slides]).name .width .height .dpi .style                        the deck (1280 × 720 slide units unless it says);
//!                                                                       style "plain", "ink", "editorial" or "signal"
//! slide([elements]).layout .name .notes .background .hidden
//! text(text).placeholder .x .y .w .h .rotation .font_size .color .bold .italic .underline .align .valign …
//! shape().shape .x .y .w .h .fill .stroke .stroke_width .corner_radius   image().src .x .y .w .h .fit .alt
//! every element also takes .opacity .label .locked .group
//! ```
//!
//! THE TEXT OF A BLOCK IS ITS TEXT PARAMETER, never an attribute: a string, or a tuple of texts and marks
//! (`p(("One ", b("two"), "."))`); a deck's text box takes a `p` per paragraph (`text([p("One"), p("Two")])`).
//!
//! A workbook and a page are documents of their own, with where each part was written in `sources` (`workbook`,
//! `sheet:<id>`, `cell:<id>!A1`, `column:<id>!B`, `row:<id>!3`, `page`, `block:<id>`). A deck is the deck's own op
//! graph (`deck.doc`, `deck.slide`, `deck.text` …), each node with `meta.source`. Anything else is refused by name.

use super::element::{attributes, elements, rust_name, Child};
use super::ir::{channels, wire};
use super::read::{boolean, call, defined, len, names, number, tag, text as text_of};
use super::source::{meta, Sources};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};
use std::collections::BTreeMap;

elements! {
    /// The workbook: its sheets.
    workbook: children;
    /// A grid sheet: its cells, columns and rows.
    sheet: children;
    /// One cell: `cell().at("B2").value(1800)` or `.formula("=B1*12")`.
    cell: leaf;
    /// A column's width in pixels: `column().at("A").width(160)`.
    column: leaf;
    /// A row's height in pixels: `row().at(3).height(40)`.
    row: leaf;
    /// The page: its blocks.
    page: children;
    /// A heading.
    h1: text;
    /// A heading.
    h2: text;
    /// A heading.
    h3: text;
    /// A paragraph; in a deck's text box, one paragraph of it.
    p: text;
    /// A list item.
    bullet: text;
    /// A numbered list item.
    numbered: text;
    /// A checklist item: `todo("Write the report.").checked(true)`.
    todo: text;
    /// A quote.
    quote: text;
    /// A code block: its text as one string.
    pre: text;
    /// A rule.
    divider: leaf;
    /// A picture: a page's block, or a deck's element.
    image: leaf;
    /// Bold.
    b: text;
    /// Italic.
    i: text;
    /// Underline.
    u: text;
    /// Strike through.
    s: text;
    /// Code.
    code: text;
    /// A link: `a("link").href("https://commandagi.com")`.
    a: text;
    /// A line break.
    br: leaf;
    /// The deck: its slides.
    deck: children;
    /// A slide on a layout, named by the layout's name: its elements.
    slide: children;
    /// A text box: its text, or a `p` per paragraph.
    text: text;
    /// A shape: `shape().shape("ellipse").x(…).y(…).w(…).h(…)`.
    shape: leaf;
}

attributes! {
    /// The office documents' attributes, chained on their elements.
    pub trait OfficeAttrs {
        /// A workbook's or a page's title.
        title;
        /// A sheet's, a slide's or a deck's name.
        name;
        /// A sheet's rows.
        rows;
        /// A sheet's columns.
        cols;
        /// A sheet's frozen rows.
        frozen_rows;
        /// A sheet's frozen columns.
        frozen_cols;
        /// A colour: a sheet's tab, a cell's text, a text box's text.
        color;
        /// Where a cell is ("B4"), a column's letters ("B"), a row's number (3).
        at;
        /// A cell's value: text, a number or true/false.
        value;
        /// A cell's formula: it starts with "=".
        formula;
        /// A cell's number format: `"$#,##0"`.
        num_fmt;
        /// Bold.
        bold;
        /// Italic.
        italic;
        /// Underline.
        underline;
        /// A cell's or a text box's alignment.
        align;
        /// A cell's background.
        bg;
        /// A cell's text wraps.
        wrap;
        /// A column's width in pixels; a deck's width in slide units.
        width;
        /// A row's height in pixels; a deck's height in slide units.
        height;
        /// A todo is done.
        checked;
        /// A code block's language.
        lang;
        /// A picture's file, by its path relative to this file.
        src;
        /// A picture's alternative text.
        alt;
        /// A link's address.
        href;
        /// A deck's dots per inch.
        dpi;
        /// A slide's layout, by its name: "Title", "Title and body", "Section header", "Blank".
        layout;
        /// A slide's speaker notes.
        notes;
        /// A slide's background.
        background;
        /// A slide is hidden.
        hidden;
        /// The layout placeholder an element takes its box from: "title", "subtitle", "body".
        placeholder;
        /// Across, in slide units.
        x;
        /// Down, in slide units.
        y;
        /// Wide, in slide units.
        w;
        /// High, in slide units.
        h;
        /// Rotation, in degrees.
        rotation;
        /// Stacking order.
        z;
        /// Shown.
        visible;
        /// Opacity, 0 to 1.
        opacity;
        /// An element's label.
        label;
        /// A text box's font size, in slide units.
        font_size;
        /// A text box's vertical alignment: "top", "middle", "bottom".
        valign;
        /// A text box's font family.
        font_family;
        /// A text box's line height.
        line_height;
        /// What a text box does with text that does not fit: "visible", "clip", "ellipsis".
        overflow;
        /// A shape's shape: "rect", "ellipse", "triangle", "line".
        shape;
        /// A shape's fill.
        fill;
        /// A shape's stroke.
        stroke;
        /// A shape's stroke width.
        stroke_width;
        /// A shape's corner radius.
        corner_radius;
        /// How a deck's picture fits its box: "fill", "contain", "cover", "none".
        fit;
        /// A page's paper: "letter" or "a4".
        paper;
        /// A page's typeface: "sans", "serif" or "mono".
        font;
        /// A deck's style: "plain", "ink", "editorial" or "signal".
        style;
        /// An element the editor does not move.
        locked;
        /// A group's name: elements with one group name select and move as one.
        group;
    }
}

pub(crate) const FAMILY: Family = Family { module: "office", roots: &["workbook", "page", "deck"], declare };

/// The deck's node types (the decks editor's own).
pub const DECK_DOC: &str = "deck.doc";
pub const DECK_SLIDE: &str = "deck.slide";
pub const DECK_TEXT: &str = "deck.text";
pub const DECK_IMAGE: &str = "deck.image";
pub const DECK_SHAPE: &str = "deck.shape";

fn declare(root: El) -> Result<Declared, String> {
    match root.tag {
        "workbook" => read_workbook(&root),
        "page" => read_page(&root),
        _ => Ok(Declared::Graph(read_deck(&root)?)),
    }
}

/// How a message names an office element: its call, with its name or where it is.
fn place(el: &El) -> String {
    call(el, &["name", "at"])
}

fn only(el: &El, allowed: &[&str]) -> Result<(), String> {
    match el.attrs.iter().find(|(k, _)| !allowed.contains(&k.as_str())) {
        Some((k, _)) => {
            let takes = if allowed.is_empty() { "nothing".to_string() } else { names(allowed) };
            Err(format!("{}: {} is not read on a {} (it takes {takes})", place(el), rust_name(k), tag(el.tag)))
        }
        None => Ok(()),
    }
}

fn txt(el: &El, prop: &str) -> Result<Option<String>, String> {
    text_of(el, prop, &place(el))
}
fn num(el: &El, prop: &str) -> Result<Option<f64>, String> {
    number(el, prop, &place(el))
}
fn flag(el: &El, prop: &str) -> Result<Option<bool>, String> {
    boolean(el, prop, &place(el))
}
fn one_of(el: &El, prop: &str, values: &[&str]) -> Result<Option<String>, String> {
    let v = txt(el, prop)?;
    match v {
        Some(s) if !values.contains(&s.as_str()) => Err(format!("{}: {} is one of {}", place(el), rust_name(prop), values.join(", "))),
        v => Ok(v),
    }
}
fn js(v: Option<String>) -> Option<Json> {
    v.map(Json::from)
}
fn jn(v: Option<f64>) -> Option<Json> {
    v.map(Json::Num)
}
fn jb(v: Option<bool>) -> Option<Json> {
    v.map(Json::Bool)
}

// ── Workbook ───────────────────────────────────────────────────────────────────────────────────────────────────

const ALIGN: &[&str] = &["left", "center", "right"];

/// An A1 reference's column letters and row number: `^([A-Z]{1,3})([1-9]\d{0,6})$`.
fn a1(at: &str) -> Option<(&str, u64)> {
    let split = at.find(|c: char| !c.is_ascii_uppercase()).unwrap_or(at.len());
    let (letters, digits) = at.split_at(split);
    let ok = (1..=3).contains(&letters.len())
        && (1..=7).contains(&digits.len())
        && digits.bytes().all(|c| c.is_ascii_digit())
        && !digits.starts_with('0');
    if ok {
        digits.parse().ok().map(|r| (letters, r))
    } else {
        None
    }
}

fn col_index(letters: &str) -> u64 {
    letters.bytes().fold(0u64, |n, c| n * 26 + (c - 64) as u64) - 1
}

fn sizes(map: &BTreeMap<u64, f64>) -> Option<Json> {
    if map.is_empty() {
        return None;
    }
    Some(Json::Obj(map.iter().map(|(k, v)| (k.to_string(), Json::Num(*v))).collect()))
}

/// A `workbook(…)` as the workbook JSON the sheets editor opens, with where each part was written.
fn read_workbook(root: &El) -> Result<Declared, String> {
    only(root, &["title"])?;
    let mut sources = Sources::new();
    sources.put("workbook", root);
    let mut seen: Vec<String> = Vec::new();
    let mut sheets = Vec::new();
    for (i, el) in root.child_elements().enumerate() {
        if el.tag != "sheet" {
            return Err(format!("{} is not read in a workbook() (it holds sheet()s)", tag(el.tag)));
        }
        only(el, &["name", "rows", "cols", "frozenRows", "frozenCols", "color"])?;
        let id = format!("sheet-{}", i + 1);
        let name = txt(el, "name")?.unwrap_or_else(|| format!("Sheet {}", i + 1));
        if seen.contains(&name.to_lowercase()) {
            return Err(format!("two sheets are called {name}"));
        }
        seen.push(name.to_lowercase());
        sources.put(format!("sheet:{id}"), el);
        let mut cells: Vec<(String, Json)> = Vec::new();
        let mut col_widths: BTreeMap<u64, f64> = BTreeMap::new();
        let mut row_heights: BTreeMap<u64, f64> = BTreeMap::new();
        let mut rows = num(el, "rows")?.unwrap_or(200.0);
        let mut cols = num(el, "cols")?.unwrap_or(26.0);
        for c in el.child_elements() {
            if c.tag == "column" || c.tag == "row" {
                let is_column = c.tag == "column";
                let size_prop = if is_column { "width" } else { "height" };
                only(c, &["at", size_prop])?;
                let index = match (is_column, c.attr("at")) {
                    (true, Some(Json::Str(s))) if (1..=3).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_uppercase()) => Some(col_index(s)),
                    (false, Some(Json::Num(r))) if r.fract() == 0.0 && *r >= 1.0 => Some(*r as u64 - 1),
                    _ => None,
                };
                let Some(index) = index else {
                    let what = if is_column { "a column's letters (\"B\")" } else { "a row's number (1 is the first)" };
                    return Err(format!("{}: at is {what}", place(c)));
                };
                let size = match num(c, size_prop)? {
                    Some(v) if v > 0.0 => v,
                    _ => return Err(format!("{}: {size_prop} is a number of pixels", place(c))),
                };
                let key = match c.attr("at") {
                    Some(Json::Str(s)) if is_column => s.clone(),
                    _ => (index + 1).to_string(),
                };
                if is_column { &mut col_widths } else { &mut row_heights }.insert(index, size);
                sources.put(format!("{}:{id}!{key}", c.tag), c);
                continue;
            }
            if c.tag != "cell" {
                return Err(format!("{} is not read in a sheet() (it holds cell(), column() and row())", tag(c.tag)));
            }
            only(c, &["at", "value", "formula", "numFmt", "bold", "italic", "align", "bg", "color", "wrap"])?;
            let at = txt(c, "at")?.map(|a| a.to_uppercase());
            let Some((at, (letters, row))) = at.as_deref().and_then(|a| a1(a).map(|m| (a.to_string(), m))) else {
                let given = c.attr("at").map(Json::text).unwrap_or_else(|| "nothing".into());
                return Err(format!("cell(): at is an A1 reference (\"B4\"), not {given}"));
            };
            if cells.iter().any(|(k, _)| *k == at) {
                return Err(format!("two cells of {name} are at {at}"));
            }
            let v = c.attr("value");
            if matches!(v, Some(Json::Arr(_) | Json::Obj(_))) {
                return Err(format!("{}: value is text, a number or true/false", place(c)));
            }
            let f = txt(c, "formula")?;
            if f.as_deref().is_some_and(|f| !f.starts_with('=')) {
                return Err(format!("{}: a formula starts with = (\"=SUM(B2:B4)\")", place(c)));
            }
            if f.is_some() && v.is_some() {
                return Err(format!("{}: a cell has a value or a formula, not both", place(c)));
            }
            let fmt = defined(vec![
                ("numFmt", js(txt(c, "numFmt")?)),
                ("bold", jb(flag(c, "bold")?)),
                ("italic", jb(flag(c, "italic")?)),
                ("align", js(one_of(c, "align", ALIGN)?)),
                ("bg", js(txt(c, "bg")?)),
                ("color", js(txt(c, "color")?)),
                ("wrap", jb(flag(c, "wrap")?)),
            ]);
            let has_fmt = len(&fmt) > 0;
            let value = if f.is_none() { v.cloned() } else { None };
            cells.push((at.clone(), defined(vec![("v", value), ("f", js(f)), ("fmt", has_fmt.then_some(fmt))])));
            rows = rows.max(row as f64);
            cols = cols.max((col_index(letters) + 1) as f64);
            sources.put(format!("cell:{id}!{at}"), c);
        }
        let (fr, fc) = (num(el, "frozenRows")?, num(el, "frozenCols")?);
        let frozen = (fr.is_some() || fc.is_some()).then(|| Json::obj().with("rows", fr.unwrap_or(0.0)).with("cols", fc.unwrap_or(0.0)));
        sheets.push(defined(vec![
            ("id", Some(id.into())),
            ("name", Some(name.into())),
            ("color", js(txt(el, "color")?)),
            ("kind", Some("grid".into())),
            ("rows", Some(rows.into())),
            ("cols", Some(cols.into())),
            ("cells", Some(Json::Obj(cells))),
            ("colWidths", sizes(&col_widths)),
            ("rowHeights", sizes(&row_heights)),
            ("frozen", frozen),
        ]));
    }
    if sheets.is_empty() {
        return Err("a workbook() holds at least one sheet()".into());
    }
    let mut document = Json::obj().with("format", "workbook").with("version", 1i64).with("sheets", Json::Arr(sheets));
    if let Some(title) = txt(root, "title")? {
        document.set("meta", Json::obj().with("title", title));
    }
    Ok(Declared::Document { format: "workbook".into(), document, sources: sources.json() })
}

// ── Page ───────────────────────────────────────────────────────────────────────────────────────────────────────

/// A text block's tag → its block type.
fn text_block(t: &str) -> Option<&'static str> {
    Some(match t {
        "h1" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "p" => "p",
        "bullet" => "ul",
        "numbered" => "ol",
        "todo" => "todo",
        "quote" => "quote",
        _ => return None,
    })
}
const MARKS: &[&str] = &["b", "i", "u", "s", "code"];

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// A text block's children as the block's inline HTML (the docs editor's subset: b, i, u, s, code, a, br).
pub fn inline_html(children: &[Child], at: &str) -> Result<String, String> {
    fn walk(children: &[Child], at: &str, out: &mut String) -> Result<(), String> {
        for c in children {
            let el = match c {
                Child::Text(t) => {
                    out.push_str(&escape(t));
                    continue;
                }
                Child::El(el) => el,
            };
            if el.tag == "br" {
                only(el, &[])?;
                out.push_str("<br>");
            } else if el.tag == "a" {
                only(el, &["href"])?;
                let href = txt(el, "href")?.unwrap_or_default();
                out.push_str(&format!("<a href=\"{}\">", href.replace('&', "&amp;").replace('"', "&quot;")));
                walk(&el.children, at, out)?;
                out.push_str("</a>");
            } else if MARKS.contains(&el.tag) {
                only(el, &[])?;
                out.push_str(&format!("<{}>", el.tag));
                walk(&el.children, at, out)?;
                out.push_str(&format!("</{}>", el.tag));
            } else {
                return Err(format!("{at}: {} is not a mark (the marks are b, i, u, s, code, a and br)", tag(el.tag)));
            }
        }
        Ok(())
    }
    let mut out = String::new();
    walk(children, at, &mut out)?;
    Ok(out)
}

/// The plain text of a `pre`'s children.
fn plain_text(children: &[Child], at: &str) -> Result<String, String> {
    let mut out = String::new();
    for c in children {
        match c {
            Child::Text(t) => out.push_str(t),
            Child::El(_) => return Err(format!("{at}: a code block holds text, not elements")),
        }
    }
    Ok(out)
}

/// A `page(…)` as the page the docs editor opens, with where each block was written.
fn read_page(root: &El) -> Result<Declared, String> {
    only(root, &["title", "paper", "font"])?;
    let mut sources = Sources::new();
    sources.put("page", root);
    let mut blocks = Vec::new();
    for (i, el) in root.child_elements().enumerate() {
        let id = format!("block-{}", i + 1);
        sources.put(format!("block:{id}"), el);
        let block = if let Some(kind) = text_block(el.tag) {
            let aligned = ALIGNED.contains(&el.tag);
            only(el, if kind == "todo" { &["checked"] } else if aligned { &["align"] } else { &[] })?;
            let html = inline_html(&el.children, &tag(el.tag))?;
            let checked = if kind == "todo" { Some(Json::Bool(flag(el, "checked")?.unwrap_or(false))) } else { None };
            let align = if aligned { js(one_of(el, "align", &["left", "center", "right", "justify"])?) } else { None };
            defined(vec![("id", Some(id.into())), ("type", Some(kind.into())), ("html", Some(html.into())), ("checked", checked), ("align", align)])
        } else {
            match el.tag {
                "pre" => {
                    only(el, &["lang"])?;
                    let html = escape(&plain_text(&el.children, "pre()")?);
                    defined(vec![("id", Some(id.into())), ("type", Some("code".into())), ("html", Some(html.into())), ("lang", js(txt(el, "lang")?))])
                }
                "divider" => {
                    only(el, &[])?;
                    Json::obj().with("id", id).with("type", "divider").with("html", "")
                }
                "image" => {
                    only(el, &["src", "alt"])?;
                    defined(vec![
                        ("id", Some(id.into())),
                        ("type", Some("image".into())),
                        ("html", Some("".into())),
                        ("src", Some(txt(el, "src")?.unwrap_or_default().into())),
                        ("alt", js(txt(el, "alt")?)),
                    ])
                }
                _ => {
                    return Err(format!(
                        "{} is not a block of a page() (the blocks are h1, h2, h3, p, bullet, numbered, todo, quote, pre, divider and image)",
                        tag(el.tag)
                    ))
                }
            }
        };
        blocks.push(block);
    }
    let mut document = Json::obj().with("format", "page").with("version", 1i64).with("blocks", Json::Arr(blocks));
    if let Some(title) = txt(root, "title")? {
        document.set("meta", Json::obj().with("title", title));
    }
    let setup = defined(vec![("paper", js(one_of(root, "paper", PAPERS)?)), ("font", js(one_of(root, "font", PAGE_FONTS)?))]);
    if len(&setup) > 0 {
        document.set("page", setup);
    }
    Ok(Declared::Document { format: "page".into(), document, sources: sources.json() })
}

// ── Deck ───────────────────────────────────────────────────────────────────────────────────────────────────────

const BOX: &[&str] = &["x", "y", "w", "h", "rotation"];
const COMMON: &[&str] = &["x", "y", "w", "h", "rotation", "placeholder", "z", "visible", "opacity", "label", "locked", "group"];
/// The deck's styles: a few curated looks (the decks editor's own themes), not a theme editor.
pub const DECK_STYLES: &[&str] = &["plain", "ink", "editorial", "signal"];
/// The blocks that take an alignment, and the page's paper and typefaces (a few, not a font menu).
const ALIGNED: &[&str] = &["h1", "h2", "h3", "p", "quote"];
pub const PAPERS: &[&str] = &["letter", "a4"];
pub const PAGE_FONTS: &[&str] = &["sans", "serif", "mono"];

/// A run's style: its marks in the order they were written (`bold`, `link` …).
type Style = Vec<(String, Json)>;

fn styled(style: &Style, key: &str, value: Json) -> Style {
    let mut out = style.clone();
    match out.iter_mut().find(|(k, _)| k == key) {
        Some(entry) => entry.1 = value,
        None => out.push((key.to_string(), value)),
    }
    out
}

/// A text box's children as the deck's rich text: a `p` per paragraph, or text and marks for one paragraph.
pub fn rich_text(children: &[Child], at: &str) -> Result<Json, String> {
    fn runs_of(c: &Child, style: &Style, at: &str, out: &mut Vec<(String, Style)>) -> Result<(), String> {
        let el = match c {
            Child::Text(t) => {
                // A run of the same style as the one before it is the same run.
                match out.last_mut() {
                    Some(prev) if prev.1 == *style => prev.0.push_str(t),
                    _ => out.push((t.clone(), style.clone())),
                }
                return Ok(());
            }
            Child::El(el) => el,
        };
        if el.tag == "a" {
            only(el, &["href"])?;
            let link = styled(style, "link", txt(el, "href")?.unwrap_or_default().into());
            return el.children.iter().try_for_each(|x| runs_of(x, &link, at, out));
        }
        let mark = match el.tag {
            "b" => "bold",
            "i" => "italic",
            "u" => "underline",
            "s" => "strike",
            _ => return Err(format!("{at}: {} is not a mark of a text box (b, i, u, s, a; a p() per paragraph)", tag(el.tag))),
        };
        only(el, &[])?;
        let marked = styled(style, mark, Json::Bool(true));
        el.children.iter().try_for_each(|x| runs_of(x, &marked, at, out))
    }
    fn runs(children: &[Child], at: &str) -> Result<Json, String> {
        let mut out = Vec::new();
        children.iter().try_for_each(|c| runs_of(c, &Vec::new(), at, &mut out))?;
        let runs = out.into_iter().map(|(text, style)| {
            let mut run = vec![("text".to_string(), Json::from(text))];
            run.extend(style);
            Json::Obj(run)
        });
        Ok(Json::obj().with("runs", Json::Arr(runs.collect())))
    }
    let paragraphs = if children.iter().any(|c| matches!(c, Child::El(el) if el.tag == "p")) {
        children
            .iter()
            .map(|c| match c {
                Child::El(el) if el.tag == "p" => {
                    only(el, &[])?;
                    runs(&el.children, at)
                }
                _ => Err(format!("{at}: a text with paragraphs holds only p()s")),
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        vec![runs(children, at)?]
    };
    Ok(Json::obj().with("paragraphs", Json::Arr(paragraphs)))
}

fn node(id: &str, kind: &str, inputs: Json, el: &El) -> Json {
    let mut n = Json::obj().with("id", id).with("type", kind).with("inputs", inputs);
    if let Some(m) = meta(el) {
        n.set("meta", m);
    }
    n
}

fn with_channels(mut inputs: Json, set: &str, wires: Vec<Json>) -> Json {
    for (k, v) in channels(set, wires) {
        inputs.set(&k, v);
    }
    inputs
}

/// A `deck(…)` as the deck's op graph: `doc`, then `slide-N`, then `slide-N.M` for its elements. A slide names its
/// layout by name (`layout`); the editor binds that name to its stock layouts.
fn read_deck(root: &El) -> Result<Json, String> {
    only(root, &["name", "width", "height", "dpi", "style"])?;
    let mut nodes: Vec<(String, Json)> = Vec::new();
    let mut slides = Vec::new();
    for (i, el) in root.child_elements().enumerate() {
        if el.tag != "slide" {
            return Err(format!("{} is not read in a deck() (it holds slide()s)", tag(el.tag)));
        }
        only(el, &["layout", "name", "notes", "background", "hidden"])?;
        let id = format!("slide-{}", i + 1);
        let mut elements = Vec::new();
        for (k, c) in el.child_elements().enumerate() {
            let eid = format!("{id}.{}", k + 1);
            let mut b = Vec::new();
            for p in BOX {
                b.push((*p, jn(num(c, p)?)));
            }
            let b = defined(b);
            let placeholder = txt(c, "placeholder")?;
            let mut common = vec![
                ("box", (len(&b) > 0).then(|| b.clone())),
                ("placeholder", js(placeholder.clone())),
                ("z", jn(num(c, "z")?)),
                ("visible", jb(flag(c, "visible")?)),
                ("opacity", jn(num(c, "opacity")?)),
                ("label", js(txt(c, "label")?)),
                ("locked", jb(flag(c, "locked")?)),
                ("group", js(txt(c, "group")?)),
            ];
            let kind = match c.tag {
                "text" => {
                    only(c, &[COMMON, &["fontSize", "color", "bold", "italic", "underline", "align", "valign", "fontFamily", "lineHeight", "overflow"]].concat())?;
                    common.extend([
                        ("text", Some(rich_text(&c.children, &format!("text() of {id}"))?)),
                        ("fontSizePx", jn(num(c, "fontSize")?)),
                        ("color", js(txt(c, "color")?)),
                        ("bold", jb(flag(c, "bold")?)),
                        ("italic", jb(flag(c, "italic")?)),
                        ("underline", jb(flag(c, "underline")?)),
                        ("align", js(one_of(c, "align", &["left", "center", "right", "justify"])?)),
                        ("valign", js(one_of(c, "valign", &["top", "middle", "bottom"])?)),
                        ("fontFamily", js(txt(c, "fontFamily")?)),
                        ("lineHeight", jn(num(c, "lineHeight")?)),
                        ("overflow", js(one_of(c, "overflow", &["visible", "clip", "ellipsis"])?)),
                    ]);
                    DECK_TEXT
                }
                "shape" => {
                    only(c, &[COMMON, &["shape", "fill", "stroke", "strokeWidth", "cornerRadius"]].concat())?;
                    common.extend([
                        ("shape", Some(one_of(c, "shape", &["rect", "ellipse", "triangle", "line"])?.unwrap_or_else(|| "rect".into()).into())),
                        ("fill", js(txt(c, "fill")?)),
                        ("stroke", js(txt(c, "stroke")?)),
                        ("strokeWidth", jn(num(c, "strokeWidth")?)),
                        ("cornerRadius", jn(num(c, "cornerRadius")?)),
                    ]);
                    DECK_SHAPE
                }
                "image" => {
                    only(c, &[COMMON, &["src", "fit", "alt"]].concat())?;
                    common.extend([
                        ("src", Some(txt(c, "src")?.unwrap_or_default().into())),
                        ("fit", js(one_of(c, "fit", &["fill", "contain", "cover", "none"])?)),
                        ("alt", js(txt(c, "alt")?)),
                    ]);
                    DECK_IMAGE
                }
                _ => return Err(format!("{} is not an element of a slide() (text, shape, image)", tag(c.tag))),
            };
            let placed = placeholder.is_some_and(|p| !p.is_empty());
            if !placed && ["x", "y", "w", "h"].iter().any(|p| b.get(p).is_none()) {
                return Err(format!("the {} {} of {id} needs x, y, w and h (or a placeholder to take them from)", tag(c.tag), k + 1));
            }
            nodes.push((eid.clone(), node(&eid, kind, defined(common), c)));
            elements.push(wire(&eid, "out"));
        }
        let inputs = defined(vec![
            ("layout", Some(txt(el, "layout")?.unwrap_or_else(|| "Title and body".into()).into())),
            ("name", js(txt(el, "name")?)),
            ("notes", js(txt(el, "notes")?)),
            ("background", js(txt(el, "background")?)),
            ("hidden", jb(flag(el, "hidden")?)),
        ]);
        nodes.push((id.clone(), node(&id, DECK_SLIDE, with_channels(inputs, "elements", elements), el)));
        slides.push(wire(&id, "out"));
    }
    let name = txt(root, "name")?.unwrap_or_else(|| "Deck".into());
    let inputs = defined(vec![
        ("name", Some(name.as_str().into())),
        ("width", Some(num(root, "width")?.unwrap_or(1280.0).into())),
        ("height", Some(num(root, "height")?.unwrap_or(720.0).into())),
        ("dpi", jn(num(root, "dpi")?)),
        ("style", js(one_of(root, "style", DECK_STYLES)?)),
    ]);
    nodes.push(("doc".into(), node("doc", DECK_DOC, with_channels(inputs, "slides", slides), root)));
    Ok(Json::obj()
        .with("id", format!("deck:{name}"))
        .with("nodes", Json::Obj(nodes))
        .with("outputs", Json::Arr(vec!["doc".into()]))
        .with("meta", Json::obj().with("domain", "deck").with("name", name)))
}
