// A connector in Rust against the document the TypeScript SDK's connectors.test.ts pins for the same elements: each
// element is its record; a connector that does not read is refused with why.

use commandagi::design::connectors::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("a connector is a document"),
    }
}

fn root() -> El {
    connector([folder([]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600)
}

fn bad(c: El) -> String {
    declare(c).unwrap_err()
}

const EXAMPLE: &str = r#"{
  "name": "Poly Haven", "base": "https://api.polyhaven.com", "rate": 2, "cache": 86400,
  "home": "https://polyhaven.com", "hosts": ["dl.polyhaven.org"], "licence": "CC0-1.0", "credit": "Poly Haven",
  "credential": {"service": "data-gov", "field": "API_KEY", "query": "api_key", "optional": true},
  "licences": [{"value": "CC0", "spdx": "CC0-1.0"}],
  "folders": [
    {"path": "", "produce": [{"kind": "dir", "name": "models"}]},
    {"path": "models", "get": "/assets?type=models", "produce": [
      {"kind": "each", "in": "$", "as": "a", "produce": [
        {"kind": "dir", "name": "{a}", "title": "{a/name}", "author": "{a/authors|keys|join:, }"}]}]},
    {"path": "models/{slug}", "get": "/files/{slug}",
     "requests": [{"as": "info", "get": "/info/{slug}"}],
     "here": {"source": "https://polyhaven.com/a/{slug}"},
     "produce": [{"kind": "each", "in": "$/gltf", "as": "res", "produce": [
       {"kind": "file", "path": "{res}/{slug}_{res}.gltf", "url": "{res/gltf/url}", "size": "{res/gltf/size}"},
       {"kind": "file", "path": "{res}/pack.zip/a.glb", "zip": "{res/zip/url}", "member": "a.glb"}]}]}
  ]}"#;

#[test]
fn the_example_is_its_document() {
    let top = connector([
        credential().service("data-gov").field("API_KEY").query("api_key").optional(true),
        licence().value("CC0").spdx("CC0-1.0"),
        folder([dir().name("models")]).path(""),
        folder([each([dir().name("{a}").title("{a/name}").author("{a/authors|keys|join:, }")]).in_("$").as_("a")])
            .path("models")
            .get("/assets?type=models"),
        folder([
            request().as_("info").get("/info/{slug}"),
            here().source("https://polyhaven.com/a/{slug}"),
            each([
                file().path("{res}/{slug}_{res}.gltf").url("{res/gltf/url}").size("{res/gltf/size}"),
                file().path("{res}/pack.zip/a.glb").zip("{res/zip/url}").member("a.glb"),
            ])
            .in_("$/gltf")
            .as_("res"),
        ])
        .path("models/{slug}")
        .get("/files/{slug}"),
    ])
    .name("Poly Haven")
    .base("https://api.polyhaven.com")
    .rate(2)
    .cache(86400)
    .home("https://polyhaven.com")
    .hosts(["dl.polyhaven.org"])
    .licence("CC0-1.0")
    .credit("Poly Haven");
    let (format, doc, sources) = document(top);
    assert_eq!(format, "connector");
    assert!(sources.get("").is_some());
    assert!(sources.get("folder#models").is_some());
    assert_eq!(doc, Json::parse(EXAMPLE).unwrap());
}

#[test]
fn a_connector_that_does_not_read_is_refused_with_why() {
    for (c, why) in [
        (root().base("http://api.example.com"), "base is the API's https url"),
        (root().rate(50), "rate is the requests a second it makes at most"),
        (root().cache(5), "cache is the seconds a listing is kept"),
        (connector([dir().name("x")]).name("S").base("https://api.example.com").rate(1).cache(3600), "dir() stands in folder() or each(), not in connector()"),
        (connector([]).name("S").base("https://api.example.com").rate(1).cache(3600), "declares no folder()"),
        (connector([folder([]).path("a//b")]).name("S").base("https://api.example.com").rate(1).cache(3600), "path is names and {var} segments"),
        (
            connector([folder([file().path("x.glb")]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600),
            "reads its bytes from url, or from zip and member",
        ),
        (
            connector([folder([file().path("x.glb").zip("https://a/x.zip")]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600),
            "zip and member go together",
        ),
        (
            connector([folder([each([]).in_("$").as_("$")]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600),
            "as is a name",
        ),
        (
            connector([folder([]).path("").get("/x").page("start")]).name("S").base("https://api.example.com").rate(1).cache(3600),
            "it needs pageSize",
        ),
        (
            connector([credential().service("data-gov").field("API_KEY"), folder([]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600),
            "a query parameter or a header",
        ),
        (connector([folder([each([]).in_("$").as_("x").where_("k").is(1).not_(2)]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600), "where goes with is or with not: one of them"),
        (connector([folder([each([]).in_("$").as_("x").where_("k").not_(json("[1]"))]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600), "not is a text, a number or true/false"),
        (connector([folder([dir().name("a").id(3)]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600), "dir() id is a text"),
        (connector([folder([]).path("").get("/x").page("start")]).name("S").base("https://api.example.com").rate(1).cache(3600), "it needs pageSize (by offset) or next (by a token)"),
        (connector([folder([]).path("").get("/x").page("link").next("{$/n}")]).name("S").base("https://api.example.com").rate(1).cache(3600), "next is the template of the next page's token"),
        (
            connector([folder([]).path(""), folder([]).path("")]).name("S").base("https://api.example.com").rate(1).cache(3600),
            r#"two folder() in connector() have path """#,
        ),
    ] {
        let said = bad(c);
        assert!(said.contains(why), "{why}: {said}");
    }
}

#[test]
fn id_not_and_next_are_read() {
    let (_, doc, _) = document(
        connector([folder([each([dir().name("{a}").id("{a/id}")]).in_("$").as_("a").where_("k").not_("x")]).path("").get("/x").page("cursor").next("{$/n}")])
            .name("S")
            .base("https://api.example.com")
            .rate(1)
            .cache(3600),
    );
    let text = doc.text();
    assert!(text.contains(r#""next":"{$/n}""#) && text.contains(r#""not":"x""#) && text.contains(r#""id":"{a/id}""#), "{text}");
}
