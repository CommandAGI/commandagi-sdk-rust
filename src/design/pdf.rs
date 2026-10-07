//! A PDF ASSEMBLED IN RUST — a `.pdf.rs`: pages taken from PDFs by ref, blank pages, comments, stamps, signatures,
//! redactions and form fields on them, form values, bookmarks, page labels and attachments. The same elements, and the
//! same documents, as the TypeScript SDK's `pdf.ts` and the Python SDK's `pdf.py`:
//!
//! ```
//! use commandagi::design::pdf::*;
//!
//! fn document() -> El {
//!     pdf([
//!         page([]).src("Contract.pdf").n(1),
//!         page([
//!             highlight().rects([[72, 700, 300, 712]]).author("Ada").text("Check this"),
//!             note([reply().text("Because.").author("Bob")]).at([500, 700]).text("Why?"),
//!             stamp().rect([400, 40, 560, 90]).name("Approved"),
//!         ])
//!         .src("Contract.pdf")
//!         .n(3)
//!         .rotate(90),
//!         page([]).size("a4"),
//!         fill().name("Name").value("Ada Lovelace"),
//!         bookmark([bookmark([]).title("Payment").page(2).top(500)]).title("Terms").page(2),
//!     ])
//!     .title("Signed contract")
//! }
//! # commandagi::design::declare(document()).unwrap();
//! ```
//!
//! Pages and page numbers are 1-based; coordinates are PDF points from the page's lower-left corner; a colour is
//! `[r, g, b]`, each 0..1; a `src` is a ref relative to the file's folder. Nothing adds a default. A run gives back
//! `{format, document, sources}` beside an empty graph.

use super::documents::{declare_document, DocTree, TagRule, Vocabulary};
use super::element::{attributes, elements};
use super::{Declared, Family};
pub use super::{fragment, json, El, Json, Value};

elements! {
    /// A PDF: its pages, form values, bookmarks, labels and attachments.
    pdf: children;
    /// A page of another PDF (`.src("Contract.pdf").n(3)`) or a blank page (`.size("a4")`), holding its marks.
    page: children;
    /// Marks the text under `.rects([...])`.
    highlight: leaf;
    underline: leaf;
    strikeout: leaf;
    squiggly: leaf;
    /// A sticky note at `.at([x, y])`, holding its replies.
    note: children;
    /// A reply in a note's thread: `.text(…)`; `.state("Completed")` resolves the thread.
    reply: leaf;
    /// A text box: `.rect([...]).text(…)`.
    textbox: leaf;
    /// Freehand strokes: `.strokes([[x, y, x, y], …])`.
    ink: leaf;
    rectangle: leaf;
    ellipse: leaf;
    /// A line or an arrow: `.from([x, y]).to([x, y]).arrow("end")`.
    line: leaf;
    polygon: leaf;
    polyline: leaf;
    /// A stamp: `.rect([...]).name("Approved")`, or `.image("seal.png")`.
    stamp: leaf;
    /// A picture of a signature: `.typed(…)`, `.image(…)` or `.strokes(…)` (0..1 in its rect).
    signature: leaf;
    /// What is under `.rect([...])` is removed when the PDF is built.
    redact: leaf;
    /// A form field: `.kind("text").name("Name").rect([...])`.
    field: leaf;
    /// A form field's value: `.name("Name").value("Ada")`.
    fill: leaf;
    /// A bookmark to `.page(n)`, holding its children.
    bookmark: children;
    /// Page labels from `.from(n)`: `.style("r")`.
    label: leaf;
    /// An attachment: `.src("data.csv")`.
    attach: leaf;
}

attributes! {
    /// The attributes of a PDF's elements, chained on them.
    pub trait PdfAttrs {
        title; author; subject; keywords; creator;
        id; src; n; rotate; size; width; height;
        text; color; opacity; date; rects; at; icon; open; rect; text_color; border; fill; align;
        strokes; from; to; arrow; points; name; label; image; typed; style; overlay;
        kind; value; options; checked; multiline; max_length; required; read_only; tooltip; default_; editable; state;
        page; top; bold; italic; url; prefix; start; description; mime_type;
    }
}

pub(crate) const FAMILY: Family = Family { module: "pdf", roots: &["pdf"], declare };

const PAGE_SIZES: &[&str] = &["a3", "a4", "a5", "letter", "legal", "tabloid"];
const PDF_FIELDS: &[&str] = &["title", "author", "subject", "keywords", "creator"];
const PAGE_FIELDS: &[&str] = &["id", "src", "n", "rotate", "size", "width", "height"];
const BOOKMARK_FIELDS: &[&str] = &["id", "title", "page", "top", "open", "bold", "italic", "color", "url"];
const LABEL_FIELDS: &[&str] = &["from", "style", "prefix", "start"];
const ATTACH_FIELDS: &[&str] = &["src", "name", "description", "mimeType"];
/// A review state a reply sets on its thread: "Completed" is resolved, "None" reopens it.
pub const REVIEW_STATES: &[&str] = &["Accepted", "Rejected", "Cancelled", "Completed", "None"];
const TEXT_MARK: &[&str] = &["id", "author", "text", "color", "opacity", "date", "rects"];

const PDF: Vocabulary = Vocabulary {
    format: "pdf",
    noun: "a PDF",
    root: "pdf",
    tags: &[
        TagRule::new("pdf", &[]).attrs(PDF_FIELDS),
        TagRule::new("page", &["pdf"]).key("id").attrs(PAGE_FIELDS),
        TagRule::new("highlight", &["page"]).key("id").required(&["rects"]).attrs(TEXT_MARK),
        TagRule::new("underline", &["page"]).key("id").required(&["rects"]).attrs(TEXT_MARK),
        TagRule::new("strikeout", &["page"]).key("id").required(&["rects"]).attrs(TEXT_MARK),
        TagRule::new("squiggly", &["page"]).key("id").required(&["rects"]).attrs(TEXT_MARK),
        TagRule::new("note", &["page"]).key("id").required(&["at"]).attrs(&["id", "author", "text", "color", "opacity", "date", "at", "icon", "open"]),
        TagRule::new("textbox", &["page"]).key("id").required(&["rect", "text"]).attrs(&["id", "author", "text", "color", "opacity", "date", "rect", "size", "textColor", "border", "fill", "align"]),
        TagRule::new("ink", &["page"]).key("id").required(&["strokes"]).attrs(&["id", "author", "text", "color", "opacity", "date", "strokes", "width"]),
        TagRule::new("rectangle", &["page"]).key("id").required(&["rect"]).attrs(&["id", "author", "text", "color", "opacity", "date", "rect", "width", "fill"]),
        TagRule::new("ellipse", &["page"]).key("id").required(&["rect"]).attrs(&["id", "author", "text", "color", "opacity", "date", "rect", "width", "fill"]),
        TagRule::new("line", &["page"]).key("id").required(&["from", "to"]).attrs(&["id", "author", "text", "color", "opacity", "date", "from", "to", "width", "arrow"]),
        TagRule::new("polygon", &["page"]).key("id").required(&["points"]).attrs(&["id", "author", "text", "color", "opacity", "date", "points", "width", "fill"]),
        TagRule::new("polyline", &["page"]).key("id").required(&["points"]).attrs(&["id", "author", "text", "color", "opacity", "date", "points", "width"]),
        TagRule::new("stamp", &["page"]).key("id").required(&["rect"]).attrs(&["id", "author", "text", "color", "opacity", "date", "rect", "name", "label", "image"]),
        TagRule::new("signature", &["page"]).key("id").required(&["rect"]).attrs(&["id", "author", "rect", "typed", "style", "image", "strokes", "color", "width"]),
        TagRule::new("redact", &["page"]).key("id").required(&["rect"]).attrs(&["id", "rect", "fill", "overlay"]),
        TagRule::new("field", &["page"]).key("id").required(&["kind", "name"]).attrs(&["id", "kind", "name", "rect", "rects", "value", "options", "checked", "multiline", "maxLength", "required", "readOnly", "tooltip", "default", "editable", "size"]),
        TagRule::new("reply", &["note"]).required(&["text"]).attrs(&["text", "author", "date", "state"]),
        TagRule::new("fill", &["pdf"]).key("name").required(&["name", "value"]).attrs(&["name", "value"]),
        TagRule::new("bookmark", &["pdf", "bookmark"]).key("id").required(&["title"]).attrs(BOOKMARK_FIELDS),
        TagRule::new("label", &["pdf"]).required(&["from"]).attrs(LABEL_FIELDS),
        TagRule::new("attach", &["pdf"]).required(&["src"]).attrs(ATTACH_FIELDS),
    ],
    from_tree: pdf_of,
};

fn declare(root: El) -> Result<Declared, String> {
    declare_document(&root, &PDF)
}

fn num(v: Option<&Json>) -> Option<f64> {
    v.and_then(|v| v.as_f64()).filter(|n| n.is_finite())
}
fn is_int(v: Option<&Json>) -> bool {
    num(v).is_some_and(|n| n.fract() == 0.0)
}
fn nums(v: &Json, len: Option<usize>) -> bool {
    matches!(v, Json::Arr(a) if len.is_none_or(|l| a.len() == l) && a.iter().all(|x| x.as_f64().is_some_and(f64::is_finite)))
}
fn rect(v: Option<&Json>) -> bool {
    v.is_some_and(|v| nums(v, Some(4)))
}
fn color(v: &Json) -> bool {
    nums(v, Some(3)) && matches!(v, Json::Arr(a) if a.iter().all(|x| x.as_f64().is_some_and(|n| (0.0..=1.0).contains(&n))))
}

fn own(t: &DocTree, keys: &[&str]) -> Json {
    let mut out = Json::obj();
    for k in keys {
        if let Some(v) = t.attr(k) {
            out.set(k, v.clone());
        }
    }
    out
}

fn check_mark(c: &DocTree) -> Result<(), String> {
    let w = format!("{}()", c.tag);
    if c.attr("rect").is_some() && !rect(c.attr("rect")) {
        return Err(format!("{w}: rect is [x0, y0, x1, y1] in points"));
    }
    if let Some(r) = c.attr("rects") {
        if !matches!(r, Json::Arr(a) if !a.is_empty() && a.iter().all(|x| nums(x, Some(4)))) {
            return Err(format!("{w}: rects is a list of [x0, y0, x1, y1]"));
        }
    }
    for k in ["at", "from", "to"] {
        if c.attr(k).is_some_and(|v| !nums(v, Some(2))) {
            return Err(format!("{w}: {k} is [x, y] in points"));
        }
    }
    for k in ["color", "textColor"] {
        if c.attr(k).is_some_and(|v| !color(v)) {
            return Err(format!("{w}: {k} is [r, g, b], each 0 to 1"));
        }
    }
    for k in ["fill", "border"] {
        if c.attr(k).is_some_and(|v| *v != Json::Null && !color(v)) {
            return Err(format!("{w}: {k} is [r, g, b], each 0 to 1, or null"));
        }
    }
    if c.attr("opacity").is_some() && !num(c.attr("opacity")).is_some_and(|n| (0.0..=1.0).contains(&n)) {
        return Err(format!("{w}: opacity is 0 to 1"));
    }
    if let Some(s) = c.attr("strokes") {
        if !matches!(s, Json::Arr(a) if a.iter().all(|x| nums(x, None) && matches!(x, Json::Arr(p) if p.len() >= 2 && p.len() % 2 == 0))) {
            return Err(format!("{w}: strokes is a list of [x, y, x, y …]"));
        }
    }
    if let Some(p) = c.attr("points") {
        if !(nums(p, None) && matches!(p, Json::Arr(a) if a.len() >= 4 && a.len() % 2 == 0)) {
            return Err(format!("{w}: points is [x, y, x, y …]"));
        }
    }
    if c.attr("arrow").is_some_and(|a| !["none", "start", "end", "both"].contains(&a.as_str().unwrap_or(""))) {
        return Err(format!(r#"{w}: arrow is "none", "start", "end" or "both""#));
    }
    if c.tag == "field" {
        let kind = c.attr("kind").and_then(Json::as_str).unwrap_or("");
        if !["text", "checkbox", "radio", "dropdown", "list", "signature"].contains(&kind) {
            return Err(r#"field(): kind is "text", "checkbox", "radio", "dropdown", "list" or "signature""#.into());
        }
        if kind == "radio" {
            let len = |k: &str| match c.attr(k) {
                Some(Json::Arr(a)) => Some(a.len()),
                _ => None,
            };
            if len("options").is_none() || len("options") != len("rects") {
                return Err(r#"field(): kind "radio" has options and rects, one rect per option"#.into());
            }
        } else if !rect(c.attr("rect")) {
            return Err("field(): rect is [x0, y0, x1, y1] in points".into());
        }
    }
    if c.tag == "signature" && ["typed", "image", "strokes"].iter().filter(|k| c.attr(k).is_some()).count() != 1 {
        return Err("signature(): is one of typed, image or strokes".into());
    }
    if c.tag == "stamp" && !["name", "image", "label"].iter().any(|k| c.attr(k).is_some()) {
        return Err("stamp(): has a name (Approved, Draft …), a label or an image".into());
    }
    for k in ["text", "author", "name", "label", "image", "typed", "overlay", "tooltip"] {
        if c.attr(k).is_some_and(|v| !matches!(v, Json::Str(_))) {
            return Err(format!("{w}: {k} is text"));
        }
    }
    for k in ["required", "readOnly", "multiline", "checked", "editable"] {
        if c.attr(k).is_some_and(|v| !matches!(v, Json::Bool(_))) {
            return Err(format!("{w}: {k} is true or false"));
        }
    }
    if c.attr("default").is_some_and(|v| !matches!(v, Json::Str(_)) && !matches!(v, Json::Arr(a) if a.iter().all(|x| matches!(x, Json::Str(_))))) {
        return Err(format!("{w}: default is text, or a list of texts"));
    }
    for r in c.children.iter().filter(|r| r.tag == "reply") {
        if r.attr("state").is_some_and(|s| !REVIEW_STATES.contains(&s.as_str().unwrap_or(""))) {
            return Err(r#"reply(): state is "Accepted", "Rejected", "Cancelled", "Completed" or "None""#.into());
        }
    }
    Ok(())
}

fn check_page(p: &DocTree) -> Result<(), String> {
    if let Some(src) = p.attr("src") {
        if !matches!(src, Json::Str(s) if !s.is_empty()) {
            return Err("page(): src is the ref of a PDF".into());
        }
        if !(is_int(p.attr("n")) && num(p.attr("n")).unwrap_or(0.0) >= 1.0) {
            return Err("page(): n is the page's number in src, from 1".into());
        }
        if ["size", "width", "height"].iter().any(|k| p.attr(k).is_some()) {
            return Err("page(): is a page of src, or a blank page of a size: not both".into());
        }
    } else {
        if p.attr("n").is_some() {
            return Err("page(): n is the page number in src".into());
        }
        match p.attr("size") {
            Some(s) if !PAGE_SIZES.contains(&s.as_str().unwrap_or("")) => return Err(format!("page(): size is {}", PAGE_SIZES.join(", "))),
            None if !(num(p.attr("width")).is_some_and(|w| w > 0.0) && num(p.attr("height")).is_some_and(|h| h > 0.0)) => {
                return Err("page(): is a page of a PDF (src and n) or a blank page (size, or width and height in points)".into())
            }
            _ => {}
        }
    }
    if p.attr("rotate").is_some() && !(is_int(p.attr("rotate")) && num(p.attr("rotate")).unwrap_or(1.0) % 90.0 == 0.0) {
        return Err("page(): rotate is a multiple of 90".into());
    }
    p.children.iter().try_for_each(check_mark)
}

fn bookmark_of(t: &DocTree) -> Result<Json, String> {
    if !matches!(t.attr("title"), Some(Json::Str(_))) {
        return Err("bookmark(): title is text".into());
    }
    if t.attr("page").is_some() && !(is_int(t.attr("page")) && num(t.attr("page")).unwrap_or(0.0) >= 1.0) {
        return Err("bookmark(): page is a page number, from 1".into());
    }
    let mut b = own(t, BOOKMARK_FIELDS);
    let kids = t.children.iter().filter(|c| c.tag == "bookmark").map(bookmark_of).collect::<Result<Vec<_>, _>>()?;
    if !kids.is_empty() {
        b.set("children", kids);
    }
    Ok(b)
}

fn pdf_of(t: &DocTree) -> Result<Json, String> {
    for k in PDF_FIELDS {
        if t.attr(k).is_some_and(|v| !matches!(v, Json::Str(_))) {
            return Err(format!("pdf(): {k} is text"));
        }
    }
    let (mut pages, mut fills, mut bookmarks, mut labels, mut attachments) = (vec![], vec![], vec![], vec![], vec![]);
    for c in &t.children {
        match c.tag {
            "page" => {
                check_page(c)?;
                let mut page = own(c, PAGE_FIELDS);
                let marks: Vec<Json> = c
                    .children
                    .iter()
                    .map(|m| {
                        let mut mark = Json::obj().with("type", m.tag);
                        for (k, v) in &m.attrs {
                            mark.set(k, v.clone());
                        }
                        let replies: Vec<Json> = m.children.iter().filter(|r| r.tag == "reply").map(DocTree::record).collect();
                        if !replies.is_empty() {
                            mark.set("replies", replies);
                        }
                        mark
                    })
                    .collect();
                if !marks.is_empty() {
                    page.set("marks", marks);
                }
                pages.push(page);
            }
            "fill" => {
                let ok = match c.attr("value") {
                    Some(Json::Str(_) | Json::Bool(_)) => true,
                    Some(Json::Arr(a)) => a.iter().all(|x| matches!(x, Json::Str(_))),
                    _ => false,
                };
                if !ok {
                    return Err("fill(): value is text, true or false, or a list of texts".into());
                }
                fills.push(own(c, &["name", "value"]));
            }
            "bookmark" => bookmarks.push(bookmark_of(c)?),
            "label" => {
                if !(is_int(c.attr("from")) && num(c.attr("from")).unwrap_or(0.0) >= 1.0) {
                    return Err("label(): from is a page number, from 1".into());
                }
                if c.attr("style").is_some_and(|s| !["D", "r", "R", "a", "A"].contains(&s.as_str().unwrap_or(""))) {
                    return Err(r#"label(): style is "D" (1 2 3), "r" (i ii), "R" (I II), "a" (a b) or "A" (A B)"#.into());
                }
                labels.push(own(c, LABEL_FIELDS));
            }
            "attach" => attachments.push(own(c, ATTACH_FIELDS)),
            _ => {}
        }
    }
    let mut out = own(t, PDF_FIELDS).with("pages", pages);
    for (k, v) in [("fill", fills), ("bookmarks", bookmarks), ("labels", labels), ("attachments", attachments)] {
        if !v.is_empty() {
            out.set(k, v);
        }
    }
    Ok(out)
}
