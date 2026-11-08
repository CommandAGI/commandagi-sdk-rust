// An opportunity and an experiment in Rust against the documents the TypeScript SDK's opportunities.test.ts pins for
// the same elements: a listing with an approved draft, an experiment with its decision; each refusal with why.

use commandagi::design::opportunities::*;
use commandagi::design::{declare, Declared};

fn document(root: El) -> (String, Json, Json) {
    match declare(root).unwrap() {
        Declared::Document { format, document, sources } => (format, document, sources),
        Declared::Graph(_) => panic!("an opportunity is a document"),
    }
}

fn refused(root: El) -> String {
    match declare(root) {
        Err(e) => e,
        Ok(_) => panic!("declared"),
    }
}

const AT: &str = "2026-11-08T10:00:00Z";

#[test]
fn the_examples_are_their_documents() {
    let first = contact().ref_("../relationships/cards/fetch-ai.vcf").role("marketplace");
    let site = first.source_json().unwrap();
    let (format, doc, sources) = document(
        opportunity([
            first,
            action().id("check-listing").title("Read the listing back").due("2026-11-10").owner("agent:market-manager"),
            evidence().id("api").fact("Agentverse registers agents through its API").source("https://docs.agentverse.ai").way("read the page").by("agent:claude-code").at(AT),
            move_().from("draft").to("approved").at("2026-11-08T11:00:00Z").by("person:jacob-valdez"),
            move_().from("approved").to("submitted").at("2026-11-08T12:00:00Z").by("agent:market-manager"),
            draft().id("listing").channel("agentverse").subject("CommandAGI").body("Plans, simulates and runs robots.").approved_by("person:jacob-valdez").approved_at("2026-11-08T11:00:00Z"),
        ])
        .title("CommandAGI's agent on Agentverse")
        .kind("listing")
        .stage("submitted")
        .status("waiting")
        .source("https://agentverse.ai")
        .deadline("2026-12-01")
        .value("0")
        .currency("USD")
        .eligibility("eligible")
        .owner("agent:market-manager")
        .channel("agentverse"),
    );
    assert_eq!(format, "opportunity");
    assert_eq!(sources.get("contact#../relationships/cards/fetch-ai.vcf"), Some(&site));
    assert_eq!(
        doc,
        Json::parse(r#"{"title": "CommandAGI's agent on Agentverse", "kind": "listing", "stage": "submitted", "value": "0",
            "status": "waiting", "source": "https://agentverse.ai", "deadline": "2026-12-01", "currency": "USD", "eligibility": "eligible",
            "owner": "agent:market-manager", "channel": "agentverse",
            "contacts": [{"ref": "../relationships/cards/fetch-ai.vcf", "role": "marketplace"}],
            "actions": [{"id": "check-listing", "title": "Read the listing back", "due": "2026-11-10", "owner": "agent:market-manager"}],
            "evidence": [{"id": "api", "fact": "Agentverse registers agents through its API", "source": "https://docs.agentverse.ai", "way": "read the page", "by": "agent:claude-code", "at": "2026-11-08T10:00:00Z"}],
            "moves": [{"from": "draft", "to": "approved", "at": "2026-11-08T11:00:00Z", "by": "person:jacob-valdez"},
                      {"from": "approved", "to": "submitted", "at": "2026-11-08T12:00:00Z", "by": "agent:market-manager"}],
            "drafts": [{"id": "listing", "channel": "agentverse", "subject": "CommandAGI", "body": "Plans, simulates and runs robots.", "approvedBy": "person:jacob-valdez", "approvedAt": "2026-11-08T11:00:00Z"}]}"#)
        .unwrap()
    );
    let (format, exp, _) = document(
        experiment([
            arm().id("number").file("hooks/number.txt"),
            arm().id("question").file("hooks/question.txt"),
            objective().id("read").kind("quality").metric("read-through").direction("up"),
            objective().id("reach").kind("distribution").metric("impressions").direction("up"),
            decision().arm("number").at("2026-11-20T09:00:00Z").by("agent:optimizer").reason("read-through 41% against 22%, reach equal"),
        ])
        .title("Which first line keeps a reader")
        .hypothesis("A concrete number beats a question")
        .status("decided")
        .channel("linkedin"),
    );
    assert_eq!(format, "experiment");
    assert_eq!(
        exp,
        Json::parse(r#"{"title": "Which first line keeps a reader", "hypothesis": "A concrete number beats a question", "status": "decided", "channel": "linkedin",
            "arms": [{"id": "number", "file": "hooks/number.txt"}, {"id": "question", "file": "hooks/question.txt"}],
            "objectives": [{"id": "read", "kind": "quality", "metric": "read-through", "direction": "up"},
                           {"id": "reach", "kind": "distribution", "metric": "impressions", "direction": "up"}],
            "decision": {"arm": "number", "at": "2026-11-20T09:00:00Z", "by": "agent:optimizer", "reason": "read-through 41% against 22%, reach equal"}}"#)
        .unwrap()
    );
    assert_eq!(stages_of("listing"), vec!["draft", "approved", "submitted", "live", "withdrawn", "refused"]);
}

#[test]
fn an_opportunity_or_experiment_that_does_not_read_is_refused_with_why() {
    let base = |kids: Vec<El>| opportunity(kids).title("x").kind("collaborator").stage("found");
    for (root, why) in [
        (base(vec![]).kind("application"), "an application is its folder of applications/"),
        (base(vec![]).kind("client"), "a client is a <Deal> of its company"),
        (base(vec![]).stage("won"), "stage is a stage of a collaborator (found, contacted, talking, working, ended, dropped)"),
        (base(vec![]).stage("ended").status("open"), "is ended, which closes it: its status is closed"),
        (base(vec![]).status("closed"), "is closed, so its stage is one that closes a collaborator"),
        (base(vec![]).value("100"), "value needs its currency"),
        (base(vec![]).value("-1").currency("USD"), "value is a decimal amount"),
        (base(vec![]).currency("USD"), "names a currency and no value"),
        (base(vec![]).owner("jacob"), "owner is a principal"),
        (base(vec![]).handle("CommandAGI"), "names a handle and no channel"),
        (base(vec![]).deadline("next week"), "deadline is an ISO 8601 day"),
        (base(vec![]).eligibility("maybe"), "eligibility is eligible, eligible-with-partner, not-eligible, unknown"),
        (base(vec![move_().from("found").to("talking").at(AT).by("person:ada")]), "is found, but its last move_() (2026-11-08T10:00:00Z) went to talking"),
        (base(vec![move_().to("found").at(AT).by("ada")]), "by is the principal who moved it"),
        (base(vec![evidence().id("e").fact("f")]), "names where the fact was read: a ref (a file) or a source"),
        (base(vec![draft().id("d").body("b").approved_by("person:ada")]), "approvedBy and approvedAt come together"),
        (base(vec![action().id("a").title("t").decision("yes")]), "decision is true when only its owner can decide it"),
        (opportunity([draft().id("d").body("b")]).title("x").kind("listing").stage("live"), "is live: a listing is public only after a principal approved its draft()"),
    ] {
        let e = refused(root);
        assert!(e.contains(why), "{e} does not say {why}");
    }
    let reach = || objective().id("r").kind("distribution").metric("impressions").direction("up");
    let dec = |arm_id: &str| decision().arm(arm_id).at(AT).by("agent:o").reason("r");
    for (root, why) in [
        (experiment([reach()]).title("x"), "measures only distribution: name a quality or an impact objective too"),
        (experiment([objective().id("q").kind("virality").metric("m").direction("up")]).title("x"), "kind is quality, distribution, impact"),
        (experiment([arm().id("a")]).title("x").status("decided"), "is decided: its decision() says which arm"),
        (experiment([arm().id("a"), dec("b")]).title("x").status("decided"), "decision() arm is one of its arms (a)"),
        (experiment([arm().id("a"), dec("a")]).title("x").status("running"), "has a decision(), so its status is decided"),
    ] {
        let e = refused(root);
        assert!(e.contains(why), "{e} does not say {why}");
    }
}
