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
fn a_constrained_sketch_is_a_leaf_whose_geometry_is_its_own_data() {
    let geometry = r#"{"points": {"p1": {"id": "p1", "x": 0, "y": 0}, "p2": {"id": "p2", "x": 40, "y": 0}}, "segments": {"s1": {"id": "s1", "type": "line", "a": "p1", "b": "p2"}}, "constraints": {}, "pointOrder": ["p1", "p2"], "segmentOrder": ["s1"], "constraintOrder": []}"#;
    let g = graph(drawing([
        sketch().sketch(json(geometry)).x(40).y(60).unit("mm").drawing_scale(2).annotations(false),
    ]).width(100).height(80));
    assert_eq!(
        inputs(&g, "sketch"),
        golden(&format!(r#"{{"sketch": {geometry}, "x": 40, "y": 60, "unit": "mm", "drawingScale": 2, "annotations": false}}"#)),
    );
    assert_eq!(
        golden(&node(&g, "composite").get("inputs").unwrap().get("layers.1").unwrap().text()),
        golden(&w("sketch")),
        "written straight on the drawing, it is a layer of it",
    );
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
fn a_paintings_bucket_gradient_and_move_chain_on_a_layer_and_a_shape_is_a_layer() {
    let selection = r#"[{"rect": [10, 10, 40, 30], "feather": 4}, {"op": "subtract", "polygon": [[20, 20], [30, 20], [25, 30]]}]"#;
    let g = graph(
        painting([
            layer([
                stroke([]).points([[1, 2, 1, 0]]).color([0, 0, 0, 1]).selection(json(selection)),
                bucket().x(5).y(6).tolerance(20).contiguous(true).color([1, 0, 0, 1]),
                gradient_fill().shape("linear").from([0, 0]).to([100, 0]).stops(json(r#"[{"at": 0, "color": [0, 0, 0, 1]}, {"at": 1, "color": [1, 1, 1, 1]}]"#)),
                move_().dx(12).dy(-3).selection(json(r#"[{"wand": [5, 6], "tolerance": 32, "contiguous": false}]"#)),
            ])
            .name("Paint"),
            rect().name("Rectangle 1").x(4).y(5).w(30).h(20).fill([0, 0, 1, 1]).stroke([0, 0, 0, 1]).stroke_width(2),
            line().name("Line 1").from([0, 0]).to([10, 10]).stroke([1, 0, 0, 1]).stroke_width(3),
        ])
        .width(100)
        .height(80),
    );
    assert_eq!(node(&g, "paint.stroke").get("inputs").unwrap().get("selection").unwrap().text(), golden(selection));
    assert_eq!(
        inputs(&g, "paint.bucket"),
        golden(&format!(
            r#"{{"name": "Paint", "visible": true, "opacity": 1, "blend": "normal", "x": 5, "y": 6, "tolerance": 20, "contiguous": true, "color": [1, 0, 0, 1], "src": {}}}"#,
            w("paint.stroke")
        ))
    );
    assert_eq!(node(&g, "doc").get("inputs").unwrap().get("layers.1").unwrap().text(), golden(&w("paint.move")));
    assert_eq!(
        inputs(&g, "paint.shape"),
        golden(r#"{"name": "Rectangle 1", "x": 4, "y": 5, "w": 30, "h": 20, "fill": [0, 0, 1, 1], "stroke": [0, 0, 0, 1], "strokeWidth": 2, "shape": "rect"}"#)
    );
    assert_eq!(node(&g, "paint.shape_2").get("inputs").unwrap().get("shape").unwrap().text(), golden(r#""line""#));
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
fn a_mask_naming_a_layer_of_the_stack_is_wired_to_it() {
    let g = graph(photo([
        exposure([mask([]).of("Brush").enabled(false)]).name("Mood").ev(-0.5),
        fill([]).name("Brush"),
    ]));
    let adjust = node(&g, "photo.adjust");
    assert_eq!(adjust.get("inputs").unwrap().get("mask").unwrap().text(), golden(&w("photo.fill")));
    assert_eq!(adjust.get("inputs").unwrap().get("maskEnabled").unwrap().text(), "false");
    assert!(node(&g, "photo.fill").get("inputs").unwrap().get("mask").is_none());
    assert_eq!(bad(photo([exposure([mask([]).of("Nope")])])), "exposure()'s mask() is masked by \"Nope\", and no layer is named that");
    assert_eq!(
        bad(photo([exposure([mask([]).of("B")]), fill([]).name("B"), fill([]).name("B")])),
        "exposure()'s mask() is masked by \"B\", and 2 layers are named that"
    );
    assert_eq!(bad(photo([exposure([mask([fill([])]).of("B")]), fill([]).name("B")])), "mask(of=\"B\") in exposure() holds no layer: it names a layer of the stack");
}

#[test]
fn a_nest_engrave_is_a_nest_engrave_node_on_the_engraves_channel() {
    let g = graph(
        nest([
            sheet().width_mm(300).height_mm(200).margin_mm(5),
            stock().material_id("plywood").thickness_mm(3).machine(json(r#"{"powerW": 40}"#)),
            options().resolution_mm(1),
            engrave().id("logo").src("logo.png").x_mm(20).y_mm(20).width_mm(60).height_mm(30).dpi(254).dither("ordered"),
        ])
        .safe_z_mm(5),
    );
    assert_eq!(
        inputs(&g, "nest"),
        golden(&format!(r#"{{"safeZMm": 5, "sheet": {}, "stock": {}, "options": {}, "engraves.1": {}}}"#, w("nest.sheet"), w("nest.stock"), w("nest.options"), w("logo")))
    );
    let mut logo = node(&g, "logo").clone();
    assert!(logo.remove("meta").is_some(), "the engrave carries its call");
    assert_eq!(
        logo.text(),
        golden(r#"{"id": "logo", "type": "nest.engrave", "inputs": {"src": "logo.png", "xMm": 20, "yMm": 20, "widthMm": 60, "heightMm": 30, "dpi": 254, "dither": "ordered"}}"#)
    );
    assert_eq!(bad(nest([sheet(), stock(), options(), engrave().src("a.png")])), "engrave(): an engrave has an id (the name the nest's code uses)");
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
    assert_eq!(bad(painting([layer([stroke([]).opacity(1)])])), "stroke(): opacity is the layer's (write it on the layer it is painted on)");
    assert_eq!(bad(drawing([sketch()])), "sketch(): sketch is its geometry, as the editor holds it ({ points, segments, constraints, … })");
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

#[test]
fn a_link_is_a_draw_instance_node_that_names_another_drawing() {
    let g = graph(
        drawing([layer([
            link().label("Badge").src("parts/Badge.draw.rs").x(40).y(60).inputs(json(r#"{"size": 48, "tone": "warm", "outlined": true}"#)),
            link().src("cloud://global/examples/2d/Badge.draw.tsx"),
        ])
        .name("Badge")])
        .width(400)
        .height(300),
    );
    let child = |port: &str| node(&g, "group").get("inputs").and_then(|i| i.get(port)).and_then(|w| w.get("wire")).and_then(|w| w.get("node")).and_then(Json::as_str).unwrap().to_string();
    let first = node(&g, &child("children.1"));
    assert_eq!(first.get("type").and_then(Json::as_str), Some("draw.instance"));
    assert_eq!(first.get("label").and_then(Json::as_str), Some("Badge"));
    assert_eq!(
        first.get("inputs").unwrap().text(),
        golden(r#"{"__instance": {"path": "parts/Badge.draw.rs"}, "x": 40, "y": 60, "inputs": {"size": 48, "tone": "warm", "outlined": true}}"#)
    );
    assert_eq!(inputs(&g, &child("children.2")), golden(r#"{"__instance": {"assetPath": "cloud://global/examples/2d/Badge.draw.tsx"}}"#));
}

#[test]
fn a_link_is_checked_against_its_schema() {
    let refused = |l: El| bad(drawing([layer([l])]));
    assert_eq!(refused(link()), "link(): src: is required");
    assert_eq!(refused(link().src("a.draw.rs").x("40")), "link(): x: must be number (got \"string\")");
    assert_eq!(refused(link().src("a.draw.rs").scale_x(2)), "link(): scaleX: is not one of its fields");
    assert_eq!(refused(link().src("a.draw.rs").inputs(json(r#"{"size": [1]}"#))), "link(): inputs/size: does not match an allowed shape (got \"array\")");
    assert_eq!(refused(link().src(" ")), "link(): src names a drawing by its path from this file's folder (\"parts/Badge.draw.rs\") or by an address");
}

#[test]
fn a_paintings_filter_chains_among_its_strokes() {
    let g = graph(
        painting([layer([
            stroke([]).points([[1.0, 2.0, 0.5, 0.0]]).color([0, 0, 0, 1]),
            gaussian_blur([]).radius(3),
            stroke([]).points([[5.0, 6.0, 0.5, 0.0]]).color([1, 0, 0, 1]),
            noise([mask([fill([]).color([1, 1, 1, 1])])]).amount(0.1).monochrome(true).seed(7),
        ])
        .name("Paint")
        .opacity(0.8)])
        .width(100)
        .height(80),
    );
    let port = |id: &str, port: &str| node(&g, id).get("inputs").and_then(|i| i.get(port)).unwrap().text();
    assert_eq!(
        inputs(&g, "paint.filter"),
        golden(&format!(r#"{{"name": "Paint", "visible": true, "opacity": 0.8, "blend": "normal", "filter": {{"type": "gaussianBlur", "radius": 3}}, "src": {}}}"#, w("paint.stroke")))
    );
    assert_eq!(port("paint.stroke_2", "src"), golden(&w("paint.filter")));
    assert_eq!(port("paint.filter_2", "filter"), golden(r#"{"type": "noise", "amount": 0.1, "monochrome": true, "seed": 7}"#));
    assert_eq!(port("paint.filter_2", "mask"), golden(&w("paint.fill")));
    assert_eq!(port("doc", "layers.1"), golden(&w("paint.filter_2")));
    assert_eq!(bad(painting([gaussian_blur([]).radius(3)])), "gaussian_blur() is painted on a layer: write it inside one");
    assert_eq!(bad(painting([layer([sharpen([]).amount(0.5).opacity(0.5)])])), "sharpen(): opacity is the layer's (write it on the layer it filters)");
}

#[test]
fn a_filter_is_checked_against_its_schema_in_a_photo_and_in_a_painting() {
    let in_photo = |f: El| bad(photo([raster([f]).src("a.png")]));
    let in_painting = |f: El| bad(painting([layer([f])]));
    for refused in [&in_photo as &dyn Fn(El) -> String, &in_painting] {
        assert_eq!(refused(gaussian_blur([]).radius(300)), "gaussian_blur(): radius: must be at most 250 (got 300)");
        assert_eq!(refused(gaussian_blur([]).radius(3).amount(1)), "gaussian_blur(): amount: is not one of its fields");
        assert_eq!(refused(noise([]).amount(0.1).monochrome("yes")), "noise(): monochrome: must be boolean (got \"string\")");
    }
    // A required field with a declared default may be left out: a blur with no radius is the editor's 4 px blur.
    for doc in [photo([raster([gaussian_blur([])]).src("a.png")]), painting([layer([gaussian_blur([])])])] {
        let g = graph(doc);
        let Some(Json::Obj(nodes)) = g.get("nodes") else { panic!("no nodes") };
        let filters: Vec<String> = nodes.iter().filter_map(|(_, n)| n.get("inputs")?.get("filter")).map(Json::text).collect();
        assert_eq!(filters, [r#"{"type":"gaussianBlur","radius":4}"#]);
    }
}

#[test]
fn the_filter_tags_and_the_link_are_the_schemas_roots() {
    let Some(Json::Obj(roots)) = twod_schema().get("roots") else { panic!("the schema has roots") };
    let mut tags: Vec<&str> = roots.iter().map(|(k, _)| k.as_str()).collect();
    tags.sort();
    let mut declared: Vec<&str> = PHOTO_FILTERS.iter().copied().chain(["link"]).collect();
    declared.sort();
    assert_eq!(tags, declared);
}
