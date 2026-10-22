//! The folder door's calls: what each operation sends, to the host's tools and to the platform's door.

use commandagi::git;

#[test]
fn a_call_names_its_tool_and_its_arguments() {
    let c = git::commit("tighten").paths(["a.txt", "b c.txt"]);
    assert_eq!((c.method, c.op, c.tool), ("POST", "commit", "git_commit"));
    assert_eq!(c.args.text(), r#"{"message":"tighten","paths":["a.txt","b c.txt"]}"#);
    assert_eq!(git::branch("feature").tool, "git_create_branch", "the host's and the platform's tool for a new branch");
    assert_eq!(git::push().remote("origin").force().args.text(), r#"{"remote":"origin","force":true}"#);
}

#[test]
fn the_platform_request_puts_a_read_in_the_query_and_a_change_in_the_body() {
    let (method, path, body) = git::log().branch("main").platform("me", "projects/bracket");
    assert_eq!((method, path.as_str(), body), ("GET", "/accounts/me/git/log?folder=projects/bracket&branch=main", None));
    let (method, path, body) = git::remote_add("origin", "https://example.com/r.git").platform("acct:user:u1", "");
    assert_eq!((method, path.as_str()), ("POST", "/accounts/acct%3Auser%3Au1/git/remote/add?folder="));
    assert_eq!(body.as_deref(), Some(r#"{"name":"origin","url":"https://example.com/r.git"}"#));
}

#[test]
fn a_host_is_an_http_endpoint_on_the_loopback() {
    let h = git::Host::at("http://127.0.0.1:4545/mcp", "t").unwrap();
    assert_eq!((h.addr.as_str(), h.path.as_str()), ("127.0.0.1:4545", "/mcp"));
    assert!(git::Host::at("https://example.com/mcp", "t").is_err());
}
