// The 2D vocabulary against the nodes the TypeScript SDK declares for the same elements (`twod.test.ts`): the
// composite and its layer groups, the paint stack's stroke chains carrying the layer's fields, photo adjustments by
// tag, masks, the nest; the encodings (d, point tuples, src → the image file); the refusals by name; the sites.

use commandagi::design::declare;
use commandagi::design::twod::*;

fn graph(root: El) -> Json {
    declare(root).unwrap().graph().unwrap()
}

fn bad(root: El) -> String {
    declare(root).unwrap_err()
}

fn golden(text: &str) -> String {
    Json::parse(text).unwrap().text()
}

fn node<'a>(g: &'a Json, id: &str) -> &'a Json {
    g.get("nodes").and_then(|n| n.get(id)).unwrap_or_else(|| panic!("no node {id} in {}", g.text()))
}

fn inputs(g: &Json, id: &str) -> String {
    node(g, id).get("inputs").unwrap().text()
}

fn w(node: &str) -> String {
    format!(r#"{{"wire": {{"node": "{}", "port": "out"}}}}"#, node)
}

#[test]
fn a_drawing_declares_the_composite_its_layer_groups_and_d_as_subpaths() {
    let g = graph(
        drawing([layer([
            rect().x(1).y(2).w(3).h(4).fill("#000000"),
            group([ellipse().cx(5).cy(6).rx(7).ry(7), blur([path().d("M 0 0 L 10 0 C 1 2 3 4 5 6 Q 7 8 9 9 Z").label("Edge")]).radius(2)]).name("Badge"),
        ])
        .name("Layer 1")])
        .name("Poster")
        .width(800)
        .height(600)
        .background("#ffffff"),
    );
    assert_eq!(g.get("meta").unwrap().text(), golden(r##"{"name": "Poster", "width": 800, "height": 600, "background": "#ffffff"}"##));
    assert_eq!(g.get("outputs").unwrap().text(), r#"["composite"]"#);
    assert_eq!(inputs(&g, "composite"), golden(&format!(r##"{{"background": "#ffffff", "layers.1": {}}}"##, w("group_2"))));
    assert_eq!(node(&g, "composite").get("label").unwrap().as_str(), Some("Output"));
    assert_eq!(inputs(&g, "group_2"), golden(&format!(r#"{{"name": "Layer 1", "children.1": {}, "children.2": {}}}"#, w("rect"), w("group"))));
    assert_eq!(inputs(&g, "group"), golden(&format!(r#"{{"name": "Badge", "children.1": {}, "children.2": {}}}"#, w("ellipse"), w("blur"))));
    assert_eq!(inputs(&g, "blur"), golden(&format!(r#"{{"radius": 2, "in": {}}}"#, w("Edge"))));
    assert_eq!(node(&g, "Edge").get("label").unwrap().as_str(), Some("Edge"));
    assert_eq!(
        node(&g, "Edge").get("inputs").unwrap().get("subpaths").unwrap().text(),
        golden(
            r#"[{"start": {"x": 0, "y": 0}, "segs": [{"to": {"x": 10, "y": 0}}, {"c1": {"x": 1, "y": 2}, "c2": {"x": 3, "y": 4}, "to": {"x": 5, "y": 6}},
                {"c1": {"x": 7, "y": 8}, "to": {"x": 9, "y": 9}}], "closed": true}]"#
        )
    );
}

#[test]
fn a_drawings_placed_image_and_raster_layer_name_their_image_files() {
    let g = graph(drawing([layer([image().src("photos/harbour.jpg").x(10).y(20).w(40).h(30), raster_layer().src("scan.png")]).name("Pictures")]).width(100).height(80));
    assert_eq!(inputs(&g, "image"), golden(r#"{"x": 10, "y": 20, "w": 40, "h": 30, "__asset": {"kind": "file", "$file": "photos/harbour.jpg", "mime": "image/jpeg"}}"#));
    assert_eq!(inputs(&g, "raster-layer"), golden(r#"{"__asset": {"kind": "file", "$file": "scan.png", "mime": "image/png"}}"#));
    let drawn = |child: El| bad(drawing([layer([child])]));
    assert_eq!(drawn(image().src("data:image/png;base64,AAAA")), "image(): src names an image file by relative path (\"scan.png\"); pixels are not written in code");
    assert_eq!(drawn(raster_layer().src("/home/a.png")), "raster_layer(): src is a path relative to this file, not /home/a.png");
    assert_eq!(drawn(image().x(1)), "image(): src names its image file by relative path (\"photo.png\")");
}

#[test]
fn a_paint_documents_strokes_chain_onto_their_layer_and_carry_its_fields() {
    let g = graph(
        painting([
            fill([]).name("Paper").color([1, 1, 1, 1]),
            layer([
                stroke([]).points([[1.0, 2.0, 0.5, 0.0], [3.0, 4.0, 0.5, 16.0]]).brush(json(r#"{"kind": "round", "size": 4}"#)).color([0, 0, 0, 1]),
                stroke([]).points([[5, 6, 1, 0, 10, 20]]).color([1, 0, 0, 1]),
            ])
            .name("Ink")
            .opacity(0.5)
            .src("Sketch.assets/ink.png"),
        ])
        .name("Sketch")
        .width(100)
        .height(80)
        .background([1, 1, 1, 1]),
    );
    assert_eq!(
        inputs(&g, "doc"),
        golden(&format!(r#"{{"name": "Sketch", "width": 100, "height": 80, "background": [1, 1, 1, 1], "layers.1": {}, "layers.2": {}}}"#, w("paint.fill"), w("paint.stroke_2")))
    );
    assert_eq!(node(&g, "paint.layer").get("inputs").unwrap().get("__asset").unwrap().text(), golden(r#"{"kind": "file", "$file": "Sketch.assets/ink.png", "mime": "image/png"}"#));
    assert_eq!(
        inputs(&g, "paint.stroke"),
        golden(&format!(
            r#"{{"name": "Ink", "visible": true, "opacity": 0.5, "blend": "normal",
                "points": [{{"x": 1, "y": 2, "pressure": 0.5, "t": 0}}, {{"x": 3, "y": 4, "pressure": 0.5, "t": 16}}],
                "brush": {{"kind": "round", "size": 4}}, "color": [0, 0, 0, 1], "src": {}}}"#,
            w("paint.layer")
        ))
    );
    assert_eq!(
        node(&g, "paint.stroke_2").get("inputs").unwrap().get("points").unwrap().text(),
        golden(r#"[{"x": 5, "y": 6, "pressure": 1, "t": 0, "tiltX": 10, "tiltY": 20}]"#)
    );
    assert_eq!(g.get("meta").unwrap().text(), golden(r#"{"domain": "paint", "name": "Sketch"}"#));
}

#[test]
fn a_photos_adjustments_are_tags_and_a_rasters_children_its_filters() {
    let g = graph(
        photo([raster([gaussian_blur([]).radius(3)]).name("Photo").src("Harbour.png"), exposure([]).name("Exposure").ev(0.35).offset(0).gamma(1)])
            .name("Harbour")
            .width(1280)
            .height(720),
    );
    assert_eq!(inputs(&g, "photo.adjust"), golden(r#"{"name": "Exposure", "adjustment": {"type": "exposure", "ev": 0.35, "offset": 0, "gamma": 1}}"#));
    assert_eq!(
        inputs(&g, "photo.filter"),
        golden(&format!(r#"{{"name": "Photo", "visible": true, "opacity": 1, "blend": "normal", "filter": {{"type": "gaussianBlur", "radius": 3}}, "src": {}}}"#, w("photo.raster")))
    );
}

#[test]
fn a_mask_is_the_one_layer_it_holds_and_carries_its_call() {
    let line = line!() + 3;
    let g = graph(photo([
        exposure([
            mask([gradient([])]),
        ])
        .name("Sky")
        .ev(-0.5),
        raster([gaussian_blur([mask([fill([]).color([1, 1, 1, 1])])]).radius(2)]).src("a.png"),
    ]));
    let adjust = node(&g, "photo.adjust");
    assert_eq!(adjust.get("inputs").unwrap().get("mask").unwrap().text(), golden(&w("photo.gradient")));
    let meta = adjust.get("meta").unwrap();
    assert_eq!(meta.get("sources").unwrap().text(), format!(r#"{{"mask":{{"site":[{line},13]}}}}"#));
    assert!(meta.get("source").is_some());
    assert_eq!(node(&g, "photo.filter").get("inputs").unwrap().get("mask").unwrap().text(), golden(&w("photo.fill")));
    assert!(node(&g, "photo.raster").get("inputs").unwrap().get("mask").is_none());
    assert_eq!(bad(photo([exposure([mask([])])])), "mask() in exposure() holds one layer");
    let p = graph(painting([layer([stroke([]).points(Vec::<i32>::new()), mask([fill([])])])]));
    assert_eq!(p.get("nodes").unwrap().get("paint.layer").unwrap().get("inputs").unwrap().get("mask").unwrap().text(), golden(&w("paint.fill")));
}

#[test]
fn a_nest_declares_its_sheet_stock_options_and_parts() {
    let g = graph(
        nest([
            sheet().width_mm(600).height_mm(400).margin_mm(5),
            stock().material_id("plywood").thickness_mm(3).machine(json(r#"{"powerW": 40}"#)),
            options().resolution_mm(1).spacing_mm(1).rotations([0, 90]).max_sheets(8),
            part().id("tab").label("Tab").quantity(2).outline([[0, 0], [10, 0], [10, 5]]),
        ])
        .safe_z_mm(5),
    );
    assert_eq!(
        inputs(&g, "nest"),
        golden(&format!(r#"{{"safeZMm": 5, "sheet": {}, "stock": {}, "options": {}, "parts.1": {}}}"#, w("nest.sheet"), w("nest.stock"), w("nest.options"), w("tab")))
    );
    let mut tab = node(&g, "tab").clone();
    assert!(tab.remove("meta").is_some(), "the part carries its call");
    assert_eq!(tab.text(), golden(r#"{"id": "tab", "type": "nest.part", "inputs": {"quantity": 2, "outline": [[0, 0], [10, 0], [10, 5]]}, "label": "Tab"}"#));
}

#[test]
fn every_node_carries_the_call_it_came_from() {
    let line = line!() + 1;
    let g = graph(drawing([layer([rect().x(0).disabled(true)])]));
    let site = |id: &str| node(&g, id).get("meta").unwrap().get("source").unwrap().get("site").unwrap().text();
    assert_eq!(site("composite"), format!("[{line},19]"));
    assert_eq!(site("group"), format!("[{line},28]"));
    assert_eq!(site("rect"), format!("[{line},35]"));
    assert_eq!(node(&g, "rect").get("disabled"), Some(&Json::Bool(true)));
}

#[test]
fn refuses_by_name_what_the_vocabulary_cannot_say() {
    assert_eq!(bad(painting([layer([]).src("data:image/png;base64,AAAA")])), "layer(): src names an image file by relative path (\"scan.png\"); pixels are not written in code");
    assert_eq!(bad(painting([stroke([]).points(Vec::<i32>::new())])), "stroke() is painted on a layer: write it inside one");
    assert_eq!(bad(painting([layer([stroke([]).opacity(1)])])), "stroke(): opacity is the layer's (write it on the layer the stroke is painted on)");
    assert_eq!(bad(drawing([rect()])), "rect() is inside a layer() (a drawing's children are its layers)");
    assert_eq!(bad(drawing([layer([layer([])])])), "layer(): a layer() is a child of the drawing(); inside it, group with group()");
    assert_eq!(bad(drawing([layer([blur([])])])), "blur() wraps the one node it changes");
    assert_eq!(bad(drawing([layer([path().d("m 0 0 l 1 1")])])), "path(): d is written in absolute commands (M L H V C Q Z), not m");
    assert_eq!(bad(drawing([layer([path().d("M 0")])])), "path(): d ends where a number is needed");
    assert_eq!(bad(drawing([layer([path().d("M 0 0 Z 4")])])), "path(): d has no command before 4");
    assert_eq!(bad(drawing([layer([brush_stroke().points([[1, 2, 3]])])])), "brush_stroke() points[0] is [x, y]");
    assert_eq!(bad(drawing([]).dpi(3)), "drawing(): dpi is not read (a drawing has name, width, height, background)");
    assert_eq!(bad(nest([part().id("a")])), "a nest needs its sheet()");
    assert_eq!(bad(nest([sheet(), stock(), options(), part()])), "part(): a part has an id (the name its placements and overrides use)");
    assert_eq!(bad(photo([gaussian_blur([])])), "gaussian_blur() is painted on a layer: write it inside one");
    assert_eq!(bad(photo([layer([])])), "layer() is not read in a photo() (see commandagi::design::twod)");
}

#[test]
fn a_drawing_is_its_first_artboard_and_each_artboard_after_its_layers_is_another() {
    let g = graph(
        drawing([
            layer([rect().x(1).y(2).w(3).h(4)]).name("Front"),
            artboard([layer([ellipse().cx(5).cy(6).rx(7).ry(7)]).name("Text")]).name("Back").width(400).height(300).background("#eeeeee"),
            artboard([]).width(200).height(100),
        ])
        .name("Card")
        .width(400)
        .height(300)
        .background("#ffffff"),
    );
    assert_eq!(g.get("outputs").unwrap().text(), golden(r#"["composite", "Back", "Artboard_3"]"#));
    assert_eq!(
        g.get("meta").unwrap().get("pages").unwrap().text(),
        golden(
            r##"[{"id": "composite", "name": "Card", "width": 400, "height": 300, "background": "#ffffff", "compositeId": "composite"},
                {"id": "Back", "name": "Back", "width": 400, "height": 300, "background": "#eeeeee", "compositeId": "Back"},
                {"id": "Artboard_3", "name": "Artboard 3", "width": 200, "height": 100, "compositeId": "Artboard_3"}]"##
        )
    );
    assert_eq!(inputs(&g, "Back"), golden(&format!(r##"{{"background": "#eeeeee", "layers.1": {}}}"##, w("group_2"))));
    assert!(graph(drawing([layer([])])).get("meta").unwrap().get("pages").is_none());
    assert_eq!(bad(drawing([artboard([]), layer([])])), "drawing(): its layers come before its artboard()s (the drawing is the first artboard)");
}
