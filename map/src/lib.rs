//! WHERE A `.rs` DOCUMENT DECLARES EACH ELEMENT — its source map, computed by `syn`, Rust's own parser, so an editor
//! writes an edit back into the file as a minimal text edit. The Rust half of the TypeScript sandbox's
//! `jsx-source.ts` and the Python SDK's `source.py`.
//!
//! Compiled to WebAssembly (`rs-map.wasm`) and shipped with the code workers: the browser computes the map without
//! compiling the file, and the local host refuses, before it compiles a file, what would let the file read this
//! computer at compile time.
//!
//! `map(text)` gives the one source-map shape every SDK gives (`graph.meta.sourceMap`, the contract the write-back
//! engine reads): every call of a snake-case function by name or path (`resistor("R1")`, `schematic::group(…)`), in
//! source order, with the methods chained on it as its attributes (`.sch_x(114.3)`, `props`) and its arguments in
//! order (`args`: the positional attributes and the children parameter). An argument that is `[…]`, `vec![…]` or a
//! tuple is a children `list`, and each element written as one of its entries is a child of the call with a `slot`
//! (where a sibling may be written beside it), unless it is in a closure or a loop (`looped`). A call whose value is
//! bound or passed on has a `spread`: an attribute it does not write may come from there. A value is a literal when
//! it is one, `-x`, `json(r#"…"#)` (its parsed JSON: the value helper is never an element) or an array or `vec![…]`
//! of these. It also lists the `use` declarations (where a new constructor is imported), and what the sandbox refuses.
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
            return out
                .with("elements", Json::Arr(vec![]))
                .with("imports", Json::Arr(vec![]))
                .with("refused", Json::Arr(vec![]))
                .with("error", format!("line {}: {e}", at.line));
        }
    };
    let mut m = Mapper { text, units: &units, elements: Vec::new(), imports: Vec::new(), refused: Vec::new(), parent: None, in_loop: false, item: None, root: false };
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
            let mut j = Json::obj().with("index", rank[i] as f64);
            if let Json::Obj(fields) = el.json.clone() {
                for (k, v) in fields {
                    j.set(&k, v);
                }
            }
            j.set("parent", el.parent.map(|p| Json::Num(rank[p] as f64)).unwrap_or(Json::Null));
            for &(arg, entry, child) in &el.refs {
                let Some(Json::Arr(args)) = field(&mut j, "args") else { continue };
                let target = match entry {
                    None => Some(&mut args[arg]),
                    Some(n) => match field(&mut args[arg], "list").and_then(|l| field(l, "entries")) {
                        Some(Json::Arr(entries)) => Some(&mut entries[n]),
                        _ => None,
                    },
                };
                if let Some(t) = target {
                    t.set("element", rank[child] as f64);
                }
            }
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
    /// The elements written as its arguments: (argument, list entry or none, the element).
    refs: Vec<(usize, Option<usize>, usize)>,
}

/// Where a call that is a child sits among its siblings: the ends of the entries around it.
#[derive(Clone, Copy)]
struct Slot {
    /// The end of the entry before it, or the offset after the list's `[` / `(`.
    before: u32,
    /// The start of the entry after it, or the offset of the list's `]` / `)`.
    after: u32,
    /// A comma follows it.
    comma: bool,
}

/// The expression visited next is an argument of the element being mapped, or an entry of its children list (a slot).
#[derive(Clone, Copy)]
struct Item {
    slot: Option<Slot>,
}

/// A children list written as an argument: `[…]`, `vec![…]` or a tuple `(…)`.
struct List {
    kind: &'static str,
    entries: Vec<Expr>,
    open: Span,
    close: Span,
    trailing: bool,
}

struct Mapper<'a> {
    text: &'a str,
    units: &'a Units,
    elements: Vec<Element>,
    imports: Vec<Json>,
    refused: Vec<Json>,
    parent: Option<usize>,
    in_loop: bool,
    item: Option<Item>,
    /// The expression visited next is what `fn document()` returns: the file's root, whose value goes nowhere else.
    root: bool,
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

    /// A literal's value: a literal, `-x`, `json(r#"…"#)` (its parsed JSON), or an array or `vec![…]` of these.
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
            Expr::Call(c) if is_json(c) => match c.args.first() {
                Some(Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. })) => Json::parse(&s.value()).ok(),
                _ => None,
            },
            Expr::Array(a) => a.elems.iter().map(|x| self.lit(x)).collect::<Option<Vec<_>>>().map(Json::Arr),
            Expr::Macro(m) if m.mac.path.is_ident("vec") => {
                let items = m.mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ok()?;
                items.iter().map(|x| self.lit(x)).collect::<Option<Vec<_>>>().map(Json::Arr)
            }
            _ => None,
        }
    }

    /// An argument or a list entry: its span, its line, and its value (a literal) or its text.
    fn arg(&self, e: &Expr) -> Json {
        let (s, t) = self.off(e.span());
        let j = Json::obj().with("start", s as f64).with("end", t as f64).with("line", e.span().start().line as f64);
        match self.lit(e) {
            Some(v) => j.with("literal", true).with("value", v),
            None => j.with("literal", false).with("expr", self.source(e.span())),
        }
    }

    /// The children list an argument is: `[ … ]`, `vec![ … ]` or a tuple `( … )`.
    fn list_of(e: &Expr) -> Option<List> {
        match e {
            Expr::Array(a) => Some(List { kind: "array", entries: a.elems.iter().cloned().collect(), open: a.bracket_token.span.open(), close: a.bracket_token.span.close(), trailing: a.elems.trailing_punct() }),
            Expr::Tuple(t) => Some(List { kind: "tuple", entries: t.elems.iter().cloned().collect(), open: t.paren_token.span.open(), close: t.paren_token.span.close(), trailing: t.elems.trailing_punct() && t.elems.len() > 1 }),
            Expr::Macro(m) if m.mac.path.is_ident("vec") => {
                let syn::MacroDelimiter::Bracket(b) = &m.mac.delimiter else { return None };
                let items = m.mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated).ok()?;
                Some(List { kind: "vec", entries: items.iter().cloned().collect(), open: b.span.open(), close: b.span.close(), trailing: items.trailing_punct() })
            }
            _ => None,
        }
    }

    /// Visit an expression; → the element it is, when it is one.
    fn expr(&mut self, e: &Expr) -> Option<usize> {
        if let Some(index) = self.element(e) {
            return Some(index);
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
        None
    }

    /// A call by path with the methods chained on it: `resistor("R1").resistance("3k").sch_x(114.3)`.
    fn element(&mut self, outer: &Expr) -> Option<usize> {
        let mut methods: Vec<&syn::ExprMethodCall> = Vec::new();
        let mut e = outer;
        while let Expr::MethodCall(m) = e {
            methods.push(m);
            e = &m.receiver;
        }
        let Expr::Call(call) = e else { return None };
        let Expr::Path(func) = &*call.func else { return None };
        // A constructor is snake case by a path of modules (`rect`, `twod::rect`); `String::from` and `Some` are not.
        if is_json(call) || !func.path.segments.iter().all(|s| s.ident.to_string().trim_start_matches("r#").starts_with(|c: char| c.is_ascii_lowercase() || c == '_')) {
            return None;
        }
        let last = func.path.segments.last()?;
        methods.reverse();

        let item = self.item.take();
        let root = std::mem::take(&mut self.root);
        let (start, _) = self.off(call.span());
        let (_, end) = self.off(outer.span());
        let begin = call.span().start();
        let line_start = self.text[..self.text.len().min(call.span().byte_range().start)].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = start - self.units.at[line_start] + 1;
        let (_, open) = self.off(call.paren_token.span.open());
        let (close, _) = self.off(call.paren_token.span.close());
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
                let mut p = Json::obj().with("name", m.method.to_string()).with("start", s as f64).with("end", t as f64);
                match m.args.first() {
                    Some(a) if m.args.len() == 1 => {
                        let (vs, ve) = self.off(a.span());
                        p.set("valueStart", vs as f64);
                        p.set("valueEnd", ve as f64);
                        match self.lit(a) {
                            Some(v) => p = p.with("literal", true).with("value", v),
                            None => p = p.with("literal", false).with("expr", self.source(a.span())),
                        }
                    }
                    _ => p = p.with("literal", false).with("expr", m.args.iter().map(|a| self.source(a.span())).collect::<Vec<_>>().join(", ")),
                }
                p.with("line", m.method.span().start().line as f64)
            })
            .collect();
        j.set("props", Json::Arr(props));
        j.set("args", Json::Arr(Vec::new()));
        j.set("open", open as f64);
        j.set("close", close as f64);
        // A call whose value is bound or passed on (not an argument or a list entry of an element) may have
        // attributes set elsewhere.
        let spread = Json::obj().with("line", begin.line as f64).with("expr", format!("{}(…), whose value the code passes on", self.source(func.span())));
        j.set("spread", if item.is_some() || root { Json::Null } else { spread });
        let slot = item.and_then(|it| it.slot);
        j.set("slot", match slot {
            Some(s) => Json::obj().with("before", s.before as f64).with("after", s.after as f64).with("comma", s.comma),
            None => Json::Null,
        });
        j.set("looped", self.in_loop);
        j.set("placed", slot.is_some() && !self.in_loop);
        let index = self.elements.len();
        self.elements.push(Element { start, end, parent: self.parent, json: j, refs: Vec::new() });

        // Its arguments: positional attributes and children. A list argument's entries are its children.
        let saved = (self.parent, self.item);
        self.parent = Some(index);
        let mut args = Vec::new();
        for (k, a) in call.args.iter().enumerate() {
            let mut arg = self.arg(a);
            match Self::list_of(a) {
                Some(list) => {
                    let (_, after_open) = self.off(list.open);
                    let (close_at, _) = self.off(list.close);
                    let spans: Vec<(u32, u32)> = list.entries.iter().map(|x| self.off(x.span())).collect();
                    let mut entries = Vec::new();
                    for (n, x) in list.entries.iter().enumerate() {
                        let before = if n == 0 { after_open } else { spans[n - 1].1 };
                        let after = spans.get(n + 1).map(|s| s.0).unwrap_or(close_at);
                        let comma = n + 1 < list.entries.len() || list.trailing;
                        entries.push(self.arg(x));
                        self.item = Some(Item { slot: Some(Slot { before, after, comma }) });
                        if let Some(child) = self.expr(x) {
                            self.elements[index].refs.push((k, Some(n), child));
                        }
                        self.item = None;
                    }
                    arg.set(
                        "list",
                        Json::obj()
                            .with("kind", list.kind)
                            .with("open", after_open as f64)
                            .with("close", close_at as f64)
                            .with("trailing", list.trailing)
                            .with("entries", Json::Arr(entries)),
                    );
                }
                None => {
                    self.item = Some(Item { slot: None });
                    if let Some(child) = self.expr(a) {
                        self.elements[index].refs.push((k, None, child));
                    }
                    self.item = None;
                }
            }
            args.push(arg);
        }
        self.elements[index].json.set("args", Json::Arr(args));
        for m in &methods {
            for a in &m.args {
                self.visit_expr(a);
            }
        }
        (self.parent, self.item) = saved;
        Some(index)
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
        self.expr(e);
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        for a in &f.attrs {
            self.visit_attribute(a);
        }
        let n = f.block.stmts.len();
        for (i, st) in f.block.stmts.iter().enumerate() {
            // The returned element is the document: an attribute it does not write is written nowhere else.
            match st {
                syn::Stmt::Expr(e, None) if i + 1 == n && f.sig.ident == "document" => {
                    self.root = true;
                    self.visit_expr(e);
                    self.root = false;
                }
                _ => self.visit_stmt(st),
            }
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

fn field<'a>(j: &'a mut Json, key: &str) -> Option<&'a mut Json> {
    match j {
        Json::Obj(entries) => entries.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
        _ => None,
    }
}

/// `json(…)`: the value helper, a literal, not an element.
fn is_json(call: &syn::ExprCall) -> bool {
    matches!(&*call.func, Expr::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "json"))
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
