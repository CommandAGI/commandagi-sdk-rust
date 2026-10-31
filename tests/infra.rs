// Exits, sessions and vantages in Rust against the documents the TypeScript SDK's infra.test.ts pins for the same
// elements: what the file does not say stays absent, and what does not read is refused with why.

use commandagi::design::infra::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, .. } => (format, document),
        Declared::Graph(_) => panic!("an infrastructure file is a document"),
    }
}

fn hetzner() -> El {
    exit().name("fsn-1").provider("hetzner").region("fsn1").country("DE").size("cax11").image("debian-12")
}

fn refused(root: El, why: &str) {
    let got = declare(root).unwrap_err();
    assert!(got.contains(why), "{got:?} says {why:?}");
}

fn parsed(text: &str) -> Json {
    Json::parse(text).unwrap()
}

#[test]
fn an_exit_is_what_it_says() {
    let (format, doc) = document(hetzner().continent("EU").max_monthly_usd(6.99));
    assert_eq!(format, "exit");
    assert_eq!(doc, parsed(r#"{"name":"fsn-1","provider":"hetzner","region":"fsn1","country":"DE","continent":"EU","size":"cax11","image":"debian-12","maxMonthlyUsd":6.99}"#));
    let (_, plain) = document(hetzner());
    assert_eq!(
        exit_with_defaults(&plain),
        parsed(r#"{"name":"fsn-1","provider":"hetzner","region":"fsn1","country":"DE","continent":null,"size":"cax11","image":"debian-12","protocol":"wireguard","enabled":true,"price":null,"maxMonthlyUsd":null}"#)
    );
    let (_, off) = document(hetzner().enabled(false));
    assert_eq!(exit_with_defaults(&off).get("enabled"), Some(&Json::Bool(false)));
}

#[test]
fn a_bad_exit_is_refused() {
    refused(hetzner().name("Fsn 1"), "name is lower-case letters");
    refused(hetzner().name("-x"), "name is lower-case letters");
    refused(hetzner().provider("linode"), r#"provider is "hetzner", "vultr", not "linode""#);
    refused(hetzner().country("de"), r#"country is an ISO 3166 code in capitals ("DE"), not "de""#);
    refused(hetzner().continent("Europe"), r#"continent is "AF""#);
    refused(hetzner().protocol("openvpn"), r#"protocol is "wireguard""#);
    refused(hetzner().enabled("yes"), "enabled is true or false");
    refused(hetzner().max_monthly_usd("6.99"), "maxMonthlyUsd is a price in USD");
    refused(hetzner().attr("ip", "1.2.3.4"), "ip is not read");
    refused(exit().provider("hetzner"), "needs name");
}

#[test]
fn a_vantage_is_a_device_a_place_and_a_session() {
    let (format, doc) = document(vantage().name("fsn-1").path("worker").exit("fsn-1").location(json(r#"{"city":"Falkenstein","country":"DE","precision":"city"}"#)));
    assert_eq!(format, "vantage");
    assert_eq!(doc, parsed(r#"{"name":"fsn-1","path":"worker","exit":"fsn-1","location":{"city":"Falkenstein","country":"DE","precision":"city"}}"#));
    assert_eq!(
        vantage_with_defaults(&doc),
        parsed(r#"{"name":"fsn-1","path":"worker","host":null,"unit":null,"snapshot":null,"exit":"fsn-1","location":{"city":"Falkenstein","country":"DE","precision":"city"},"locale":null,"timezone":null,"session":null,"device":null,"clock":null,"enabled":true}"#)
    );
}

#[test]
fn a_vantage_has_no_words_for_a_person() {
    for k in ["displayName", "birthday", "gender", "persona", "profile", "history", "age", "email"] {
        refused(vantage().name("v-1").path("host").attr(k, "x"), "is not read");
    }
    let v = || vantage().name("v-1").path("host").host("laptop-linux");
    refused(v().device(json(r#"{"kind":"desktop","owner":"Jane"}"#)), "device.owner is not read");
    refused(v().location(json(r#"{"street":"1 Main St"}"#)), "location.street is not read");
}

#[test]
fn a_bad_vantage_is_refused() {
    let v = || vantage().name("v-1").path("host").host("laptop-linux");
    refused(v().name("Jane Doe"), "name is the folder's word");
    refused(v().path("satellite"), r#"path is "browser""#);
    refused(vantage().name("b-1").path("browser"), "a browser is driven by a host");
    refused(vantage().name("w-1").path("worker").host("x"), "the worker's path has no host");
    refused(v().unit("x#y"), r#"for path="phone" only"#);
    refused(v().snapshot("s"), "snapshot is for a VM");
    refused(v().exit("Germany"), "exit is an exit's name");
    refused(v().exit("ab"), "exit is an exit's name");
    refused(v().locale("English"), "locale is a BCP 47 tag");
    refused(v().timezone("CST 6"), "timezone is an IANA zone");
    refused(v().location(json(r#"{"lat":120}"#)), "location.lat is a number from -90 to 90");
    refused(v().location(json(r#"{"country":"us"}"#)), "location.country is an ISO 3166 code");
    refused(v().clock(50), "clock is { sd } in ms");
    refused(v().clock(json(r#"{"sd":-1}"#)), "clock is { sd } in ms");
}

#[test]
fn a_session_names_its_monitors_and_holds_no_credential() {
    let (format, doc) = document(session([uses().monitor("../../../social/x/nvidia")]).name("x-web").site("https://x.com").kind("browser-profile").profile("x-web").refreshed("2026-10-12"));
    assert_eq!(format, "session");
    assert_eq!(doc, parsed(r#"{"name":"x-web","site":"https://x.com","kind":"browser-profile","profile":"x-web","refreshed":"2026-10-12","uses":[{"monitor":"../../../social/x/nvidia"}]}"#));
    assert_eq!(
        session_with_defaults(&doc),
        parsed(r#"{"name":"x-web","site":"https://x.com","kind":"browser-profile","host":null,"profile":"x-web","snapshot":null,"refreshed":"2026-10-12","note":null,"uses":["../../../social/x/nvidia"]}"#)
    );
}

#[test]
fn a_bad_session_is_refused() {
    let web = || session([]).name("x-web").site("https://x.com").kind("browser-profile").profile("x-web");
    refused(web().attr("password", "hunter2"), "password is not read");
    refused(web().site("http://x.com"), "site is an https address");
    refused(web().site("https://x.com/a b"), "site is an https address");
    refused(web().kind("cookie"), r#"kind is "browser-profile""#);
    refused(web().profile("X Web"), "profile is lower-case");
    refused(web().snapshot("s"), "a browser-profile has a profile, not a snapshot");
    refused(web().refreshed("2026-13-01"), "refreshed is a day");
    refused(web().note(""), "note is a word");
    let vm = || session([]).name("vm").site("https://x.com").kind("vm-snapshot");
    refused(vm(), "a vm-snapshot names its snapshot");
    refused(vm().snapshot("s").profile("p"), "a vm-snapshot has a snapshot, not a profile");
    refused(session([uses().monitor("a"), uses().monitor("a")]).name("x-web").site("https://x.com").kind("browser-profile").profile("x-web"), r#"two uses() in session() have monitor "a""#);
}
