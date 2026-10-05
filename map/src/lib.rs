//! WHERE A `.rs` DOCUMENT DECLARES EACH ELEMENT — its source map, computed by `syn`, Rust's own parser, so an editor
//! writes an edit back into the file as a minimal text edit. The Rust half of the TypeScript sandbox's
//! `jsx-source.ts` and the Python SDK's `source.py`.
//!
//! Compiled to WebAssembly (`rs-map.wasm`) and shipped with the code workers: the browser computes the map without
//! compiling the file, and the local host refuses, before it compiles a file, what would let the file read this
//! computer at compile time.
//!
//! `map(text)` lists every call of a function by path (`resistor("R1")`, `sheet::group(…)`) in source order, with the
//! method calls chained on it as its attributes (`.sch_x(114.3)`): each one's span, its argument's span, and its value
//! when the argument is a literal (else the expression's text). A call written as an item of another call's array
//! argument (`group("Divider", [ … ])`) is a child of that call, unless it is inside a closure or a loop. It lists the
//! `use` declarations (where a new constructor is imported), and what the sandbox refuses.
//!
//! Offsets are UTF-16 code units from the start of the text (what a JavaScript string indexes); lines are 1-based; a
//! `site` is the line and 1-based column in characters, which is what `std::panic::Location` gives a running file.

use commandagi::design::Json;
use proc_macro2::{Span, TokenStream, TokenTree};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, Token};

/// The macros a file may call. Every other one is refused: `include!`, `include_str!`, `include_bytes!`, `env!` and
/// `option_env!` read this computer while the file compiles, and a macro the file defines could expand to them.
const MACROS: &[&str] = &[
    "format", "vec", "assert", "assert_eq", "assert_ne", "debug_assert", "debug_assert_eq", "debug_assert_ne", "panic", "unreachable", "todo",
    "unimplemented", "matches", "concat", "stringify", "write", "writeln", "format_args", "println", "print", "eprintln", "eprint", "dbg", "line",
    "column", "cfg", "compile_error",
];
/// Names a `use` may not bring in (the macros above that read the computer, by another name).
const UNSAFE_NAMES: &[&str] = &["include", "include_str", "include_bytes", "env", "option_env", "concat_idents", "file", "module_path", "global_asm", "asm"];
/// Words after which `!` is a negation, not a macro call.
const KEYWORDS: &[&str] = &["if", "while", "match", "return", "in", "else", "break", "let", "mut", "ref", "move", "yield", "as", "for", "loop"];

/// The map of `text`, as JSON.
pub fn map(text: &str) -> Json {
    let units = Units::new(text);
    let mut out = Json::obj().with("language", "rs").with("length", units.length as f64);
    let file = match syn::parse_file(text) {
        Ok(file) => file,
        Err(e) => {
            let at = e.span().start();
            return out.with("error", format!("line {}: {e}", at.line)).with("elements", Json::Arr(vec![])).with("imports", Json::Arr(vec![]));
        }
    };
    let mut m = Mapper { text, units: &units, elements: Vec::new(), imports: Vec::new(), refused: Vec::new(), parent: None, in_loop: false, item: None };
    if let Ok(tokens) = text.parse::<TokenStream>() {
        m.scan_macros(tokens);
    }
    for attr in &file.attrs {
        m.refuse(attr.span(), "an inner attribute (`#![…]`)".into());
    }
    m.visit_file(&file);
    let elements = std::mem::take(&mut m.elements);
    let mut order: Vec<usize> = (0..elements.len()).collect();
    order.sort_by_key(|&i| (elements[i].start, std::cmp::Reverse(elements[i].end)));
    let mut rank = vec![0; elements.len()];
    for (r, &i) in order.iter().enumerate() {
        rank[i] = r;
    }
    let sorted: Vec<Json> = order
        .iter()
        .map(|&i| {
            let el = &elements[i];
            let mut j = el.json.clone();
            j.set("index", rank[i] as f64);
            j.set("parent", el.parent.map(|p| Json::Num(rank[p] as f64)).unwrap_or(Json::Null));
            j
        })
        .collect();
    out = out.with("elements", Json::Arr(sorted)).with("imports", Json::Arr(m.imports)).with("refused", Json::Arr(m.refused));
    out
}

/// UTF-16 offsets of a text's byte offsets.
struct Units {
    at: Vec<u32>,
    length: u32,
}

impl Units {
    fn new(text: &str) -> Units {
        let mut at = vec![0u32; text.len() + 1];
        let mut u = 0u32;
        for (b, c) in text.char_indices() {
            for k in 0..c.len_utf8() {
                at[b + k] = u;
            }
            u += c.len_utf16() as u32;
        }
        at[text.len()] = u;
        Units { at, length: u }
    }
}

struct Element {
    start: u32,
    end: u32,
    parent: Option<usize>,
    json: Json,
}

/// A call that is an item of its parent's array: the ends of the items around it.
#[derive(Clone, Copy)]
struct Item {
    parent: usize,
    /// The end of the item before it, or the offset after the array's `[`.
    before: u32,
    /// The start of the item after it, or the offset of the array's `]`.
    after: u32,
    /// A comma follows it.
    comma: bool,
}

struct Mapper<'a> {
    text: &'a str,
    units: &'a Units,
    elements: Vec<Element>,
    imports: Vec<Json>,
    refused: Vec<Json>,
    parent: Option<usize>,
    in_loop: bool,
    /// The expression visited next is this item of an array.
    item: Option<Item>,
}

impl Mapper<'_> {
    fn off(&self, span: Span) -> (u32, u32) {
        let r = span.byte_range();
        (self.units.at[r.start.min(self.text.len())], self.units.at[r.end.min(self.text.len())])
    }

    fn source(&self, span: Span) -> String {
        let r = span.byte_range();
        self.text.get(r).unwrap_or("").to_string()
    }

    fn refuse(&mut self, span: Span, what: String) {
        self.refused.push(Json::obj().with("line", span.start().line as f64).with("what", what));
    }

    /// Every `name!` of the file, in its tokens (attributes and macro bodies included).
    fn scan_macros(&mut self, tokens: TokenStream) {
        let list: Vec<TokenTree> = tokens.into_iter().collect();
        for (i, t) in list.iter().enumerate() {
            match t {
                TokenTree::Group(g) => self.scan_macros(g.stream()),
                TokenTree::Ident(id) => {
                    let (Some(TokenTree::Punct(p)), Some(next)) = (list.get(i + 1), list.get(i + 2)) else { continue };
                    if p.as_char() != '!' || !matches!(next, TokenTree::Group(_) | TokenTree::Ident(_)) {
                        continue;
                    }
                    let name = id.to_string();
                    let name = name.trim_start_matches("r#");
                    if KEYWORDS.contains(&name) || MACROS.contains(&name) {
                        continue;
                    }
                    // `a.b!` is not a macro call, and `x ! y` with a space after a value is not one either.
                    if i > 0 && matches!(&list[i - 1], TokenTree::Punct(q) if q.as_char() == '.') {
                        continue;
                    }
                    self.refuse(id.span(), format!("`{name}!`"));
                }
                _ => {}
            }
        }
    }

    fn lit(&self, e: &Expr) -> Option<Json> {
        match e {
            Expr::Lit(l) => match &l.lit {
                syn::Lit::Str(s) => Some(Json::Str(s.value())),
                syn::Lit::Int(i) => i.base10_parse::<f64>().ok().map(Json::Num),
                syn::Lit::Float(f) => f.base10_parse::<f64>().ok().filter(|v| v.is_finite()).map(Json::Num),
                syn::Lit::Bool(b) => Some(Json::Bool(b.value)),
                _ => None,
            },
            Expr::Unary(u) if matches!(u.op, syn::UnOp::Neg(_)) => match self.lit(&u.expr)? {
                Json::Num(n) => Some(Json::Num(-n)),
                _ => None,
            },
            Expr::Paren(p) => self.lit(&p.expr),
            _ => None,
        }
    }

    /// An argument: its span and its value (literal) or its text.
    fn arg(&self, e: &Expr) -> Json {
        let (s, t) = self.off(e.span());
        let j = Json::obj().with("start", s as f64).with("end", t as f64);
        match self.lit(e) {
            Some(v) => j.with("literal", true).with("value", v),
            None => j.with("literal", false).with("expr", self.source(e.span())),
        }
    }

    /// The array a call takes its children from (`[ … ]` or `vec![ … ]`): its items and the brackets' spans.
    fn children_of(e: &Expr) -> Option<(Vec<Expr>, Span, Span, bool)> {
        match e {
            Expr::Array(a) => Some((a.elems.iter().cloned().collect(), a.bracket_token.span.open(), a.bracket_token.span.close(), a.elems.trailing_punct())),
            Expr::Macro(m) if m.mac.path.is_ident("vec") => {
                let syn::MacroDelimiter::Bracket(b) = &m.mac.delimiter else { return None };
                let items = m.mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ok()?;
                Some((items.iter().cloned().collect(), b.span.open(), b.span.close(), items.trailing_punct()))
            }
            _ => None,
        }
    }

    /// A call by path with the methods chained on it: `resistor("R1").resistance("3k").sch_x(114.3)`.
    fn element(&mut self, outer: &Expr) -> bool {
        let mut methods: Vec<&syn::ExprMethodCall> = Vec::new();
        let mut e = outer;
        while let Expr::MethodCall(m) = e {
            methods.push(m);
            e = &m.receiver;
        }
        let Expr::Call(call) = e else { return false };
        let Expr::Path(func) = &*call.func else { return false };
        let Some(last) = func.path.segments.last() else { return false };
        methods.reverse();

        let item = self.item.take();
        let (start, _) = self.off(call.span());
        let (_, end) = self.off(outer.span());
        let begin = call.span().start();
        let line_start = self.text[..self.text.len().min(call.span().byte_range().start)].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = start - self.units.at[line_start] + 1;
        let mut j = Json::obj()
            .with("tag", last.ident.to_string())
            .with("callee", self.source(func.span()))
            .with("start", start as f64)
            .with("end", end as f64)
            .with("line", begin.line as f64)
            .with("column", column as f64)
            .with("site", vec![Json::Num(begin.line as f64), Json::Num(begin.column as f64 + 1.0)]);
        let props: Vec<Json> = methods
            .iter()
            .map(|m| {
                let (s, _) = self.off(m.dot_token.span());
                let (_, t) = self.off(m.paren_token.span.close());
                let mut p = Json::obj().with("name", m.method.to_string()).with("start", s as f64).with("end", t as f64).with("line", m.method.span().start().line as f64);
                match m.args.first() {
                    Some(a) if m.args.len() == 1 => {
                        let (vs, ve) = self.off(a.span());
                        p.set("valueStart", vs as f64);
                        p.set("valueEnd", ve as f64);
                        match self.lit(a) {
                            Some(v) => p.with("literal", true).with("value", v),
                            None => p.with("literal", false).with("expr", self.source(a.span())),
                        }
                    }
                    _ => p.with("literal", false).with("expr", m.args.iter().map(|a| self.source(a.span())).collect::<Vec<_>>().join(", ")),
                }
            })
            .collect();
        j.set("props", Json::Arr(props));
        j.set("args", Json::Arr(call.args.iter().map(|a| self.arg(a)).collect()));
        // A call whose value is bound or passed on (not an item of an array) may have attributes set elsewhere.
        let placed = item.is_some_and(|it| Some(it.parent) == self.parent);
        let spread = Json::obj().with("line", begin.line as f64).with("expr", format!("{}(…), whose value the code passes on", self.source(func.span())));
        j.set("spread", if placed { Json::Null } else { spread });
        j.set("item", match item.filter(|_| placed) {
            Some(it) => Json::obj().with("before", it.before as f64).with("after", it.after as f64).with("comma", it.comma),
            None => Json::Null,
        });
        j.set("child", placed && !self.in_loop);
        j.set("loop", self.in_loop);
        let index = self.elements.len();
        self.elements.push(Element { start, end, parent: self.parent, json: j });

        // Its arguments: an array of elements makes it their parent; anything else is visited in its own right.
        let saved = (self.parent, self.item);
        self.parent = Some(index);
        for a in &call.args {
            match Self::children_of(a) {
                Some((items, open, close, trailing)) => {
                    let (_, after_open) = self.off(open);
                    let (close_at, _) = self.off(close);
                    let spans: Vec<(u32, u32)> = items.iter().map(|x| self.off(x.span())).collect();
                    let indent = spans.first().map(|&(s, _)| self.indent_at(s)).unwrap_or_default();
                    let last_end = spans.last().map(|s| s.1).unwrap_or(after_open);
                    self.elements[index].json.set(
                        "items",
                        Json::obj()
                            .with("open", after_open as f64)
                            .with("close", close_at as f64)
                            .with("count", items.len() as f64)
                            .with("lastEnd", last_end as f64)
                            .with("trailing", trailing)
                            .with("indent", indent),
                    );
                    for (k, x) in items.iter().enumerate() {
                        let before = if k == 0 { after_open } else { spans[k - 1].1 };
                        let after = spans.get(k + 1).map(|s| s.0).unwrap_or(close_at);
                        let comma = k + 1 < items.len() || trailing;
                        self.item = Some(Item { parent: index, before, after, comma });
                        self.visit_expr(x);
                        self.item = None;
                    }
                }
                None => self.visit_expr(a),
            }
        }
        for m in &methods {
            for a in &m.args {
                self.visit_expr(a);
            }
        }
        (self.parent, self.item) = saved;
        true
    }

    /// The indent of the line a UTF-16 offset is on.
    fn indent_at(&self, unit: u32) -> String {
        let byte = self.units.at.iter().position(|&u| u >= unit).unwrap_or(self.text.len()).min(self.text.len());
        let start = self.text[..byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
        self.text[start..].chars().take_while(|c| *c == ' ' || *c == '\t').collect()
    }

    fn looped(&mut self, f: impl FnOnce(&mut Self)) {
        let was = self.in_loop;
        self.in_loop = true;
        f(self);
        self.in_loop = was;
    }

    fn use_tree(&mut self, prefix: &mut Vec<String>, tree: &syn::UseTree, whole: (u32, u32)) {
        match tree {
            syn::UseTree::Path(p) => {
                prefix.push(p.ident.to_string());
                self.use_tree(prefix, &p.tree, whole);
                prefix.pop();
            }
            syn::UseTree::Glob(_) => self.imports.push(Json::obj().with("module", prefix.join("::")).with("star", true).with("names", Json::Arr(vec![])).with("group", Json::Null).with("start", whole.0 as f64).with("end", whole.1 as f64)),
            syn::UseTree::Name(_) | syn::UseTree::Rename(_) => {
                let name = self.use_name(tree);
                self.imports.push(Json::obj().with("module", prefix.join("::")).with("star", false).with("names", Json::Arr(vec![name])).with("group", Json::Null).with("start", whole.0 as f64).with("end", whole.1 as f64));
            }
            syn::UseTree::Group(g) => {
                let names: Vec<Json> = g.items.iter().filter(|t| matches!(t, syn::UseTree::Name(_) | syn::UseTree::Rename(_))).map(|t| self.use_name(t)).collect();
                let (_, open) = self.off(g.brace_token.span.open());
                let (close, _) = self.off(g.brace_token.span.close());
                let star = g.items.iter().any(|t| matches!(t, syn::UseTree::Glob(_)));
                self.imports.push(
                    Json::obj()
                        .with("module", prefix.join("::"))
                        .with("star", star)
                        .with("names", Json::Arr(names))
                        .with("group", vec![Json::Num(open as f64), Json::Num(close as f64)])
                        .with("start", whole.0 as f64)
                        .with("end", whole.1 as f64),
                );
                for t in &g.items {
                    if matches!(t, syn::UseTree::Path(_) | syn::UseTree::Group(_)) {
                        self.use_tree(prefix, t, whole);
                    }
                }
            }
        }
    }

    fn use_name(&self, tree: &syn::UseTree) -> Json {
        let (name, asname) = match tree {
            syn::UseTree::Name(n) => (n.ident.to_string(), Json::Null),
            syn::UseTree::Rename(r) => (r.ident.to_string(), Json::Str(r.rename.to_string())),
            _ => unreachable!(),
        };
        let (s, t) = self.off(tree.span());
        Json::obj().with("name", name).with("asname", asname).with("start", s as f64).with("end", t as f64)
    }

    fn check_use(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(p) => self.check_use(&p.tree),
            syn::UseTree::Group(g) => g.items.iter().for_each(|t| self.check_use(t)),
            syn::UseTree::Name(n) if UNSAFE_NAMES.contains(&n.ident.to_string().as_str()) => self.refuse(n.ident.span(), format!("`use` of `{}`", n.ident)),
            syn::UseTree::Rename(r) => {
                let (from, to) = (r.ident.to_string(), r.rename.to_string());
                if UNSAFE_NAMES.contains(&from.as_str()) || MACROS.contains(&to.as_str()) {
                    self.refuse(r.ident.span(), format!("`use` of `{from}` as `{to}`"));
                }
            }
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for Mapper<'_> {
    fn visit_expr(&mut self, e: &'ast Expr) {
        if matches!(e, Expr::Call(_) | Expr::MethodCall(_)) && self.element(e) {
            return;
        }
        self.item = None;
        match e {
            Expr::Closure(c) => self.looped(|m| visit::visit_expr_closure(m, c)),
            Expr::ForLoop(f) => {
                self.visit_expr(&f.expr);
                self.looped(|m| m.visit_block(&f.body));
            }
            Expr::While(w) => self.looped(|m| visit::visit_expr_while(m, w)),
            Expr::Loop(l) => self.looped(|m| visit::visit_expr_loop(m, l)),
            Expr::Macro(mac) => {
                // The arguments of a macro the file may call are expressions: a call in them is mapped too.
                if let Ok(items) = mac.mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated) {
                    for x in &items {
                        self.visit_expr(x);
                    }
                }
            }
            _ => visit::visit_expr(self, e),
        }
    }

    fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
        self.check_use(&u.tree);
        let whole = self.off(u.span());
        self.use_tree(&mut Vec::new(), &u.tree, whole);
    }

    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        if m.content.is_none() {
            self.refuse(m.ident.span(), format!("`mod {};` (a module in another file)", m.ident));
        }
        visit::visit_item_mod(self, m);
    }

    fn visit_item_extern_crate(&mut self, c: &'ast syn::ItemExternCrate) {
        self.refuse(c.ident.span(), format!("`extern crate {}`", c.ident));
    }

    fn visit_item_foreign_mod(&mut self, f: &'ast syn::ItemForeignMod) {
        self.refuse(f.abi.extern_token.span, "an `extern` block (a function of the host)".into());
    }

    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        if a.path().is_ident("path") || a.path().is_ident("link") {
            let name = a.path().segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
            self.refuse(a.span(), format!("the attribute `#[{name}]`"));
        }
        visit::visit_attribute(self, a);
    }
}

// ── The WebAssembly interface: no imports; the page writes the text into memory it asked for, then reads the JSON. ──

static mut BUFFER: Vec<u8> = Vec::new();
static mut OUT: Vec<u8> = Vec::new();

/// Room for `len` bytes of text; → where to write them.
#[no_mangle]
pub extern "C" fn commandagi_map_alloc(len: u32) -> u32 {
    unsafe {
        let buffer = &mut *std::ptr::addr_of_mut!(BUFFER);
        *buffer = vec![0; len as usize];
        buffer.as_mut_ptr() as usize as u32
    }
}

/// Map the text written at `commandagi_map_alloc`'s answer; → 1 (the JSON is at `commandagi_map_out_ptr`), 0 (not UTF-8).
#[no_mangle]
pub extern "C" fn commandagi_map() -> u32 {
    unsafe {
        let buffer = &*std::ptr::addr_of!(BUFFER);
        let out = &mut *std::ptr::addr_of_mut!(OUT);
        match std::str::from_utf8(buffer) {
            Ok(text) => {
                *out = map(text).text().into_bytes();
                1
            }
            Err(_) => 0,
        }
    }
}

#[no_mangle]
pub extern "C" fn commandagi_map_out_ptr() -> u32 {
    unsafe { (*std::ptr::addr_of!(OUT)).as_ptr() as usize as u32 }
}

#[no_mangle]
pub extern "C" fn commandagi_map_out_len() -> u32 {
    unsafe { (*std::ptr::addr_of!(OUT)).len() as u32 }
}
