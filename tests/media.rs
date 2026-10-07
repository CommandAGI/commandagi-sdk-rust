// The media vocabulary against the graphs the TypeScript SDK declares for the same elements (`media.test.ts`): a
// clip's media is a source node named by its relative path, out becomes a duration, titles carry their text; a song's
// chain is synth → effects → track → master with notes inline; each call rides on its node (meta.source) and on what
// the node holds (meta.sources); what the vocabulary cannot say is refused by name.

use commandagi::design::declare;
use commandagi::design::media::*;

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

fn input(g: &Json, id: &str, port: &str) -> String {
    node(g, id).get("inputs").and_then(|i| i.get(port)).unwrap_or_else(|| panic!("no {id}.{port}")).text()
}

fn site(line: u32, column: u32) -> String {
    format!(r#"{{"site":[{line},{column}]}}"#)
}

#[test]
fn a_video_declares_sources_clips_tracks_and_the_composite() {
    let line = line!() + 4;
    let g = graph(
        video([
            track([
                clip([transition().kind("crossDissolve").duration(0.5), keyframe().property("opacity").time(0).value(0)])
                    .src("media/take.webm")
                    .start(1)
                    .in_(2)
                    .out(6)
                    .speed(2),
                title([]).text("Hello").start(0).duration(2).font_size(48),
            ])
            .name("V1"),
            track([clip([]).src("media/tone.wav").duration(3).volume(0.5)]).name("A1").kind("audio"),
            marker().time(4).name("Drop"),
        ])
        .name("Cut")
        .fps(25),
    );
    assert_eq!(input(&g, "src_media_take.webm", "__asset"), golden(r#"{"kind": "file", "$file": "media/take.webm"}"#));
    assert_eq!(input(&g, "clip_take.webm", "duration"), "2", "out - in, at twice the speed");
    assert_eq!(input(&g, "clip_take.webm", "source"), golden(r#"{"wire": {"node": "src_media_take.webm", "port": "media"}}"#));
    assert_eq!(input(&g, "clip_take.webm", "transitionIn"), golden(r#"{"kind": "crossDissolve", "duration": 0.5}"#));
    let meta = node(&g, "clip_take.webm").get("meta").unwrap();
    assert_eq!(meta.get("source").unwrap().text(), site(line, 17));
    assert_eq!(meta.get("sources").unwrap().text(), format!(r#"{{"transition":{},"keyframe:kf_clip_take.webm":{}}}"#, site(line, 23), site(line, 73)));
    assert_eq!(node(&g, "clip_Title").get("inputs").unwrap().get("text").unwrap().get("fontSize"), Some(&Json::Num(48.0)));
    assert_eq!(input(&g, "track_V1", "clips.2"), golden(r#"{"wire": {"node": "clip_Title", "port": "frames"}}"#));
    assert_eq!(input(&g, "track_A1", "kind"), r#""audio""#);
    assert_eq!(input(&g, "composite", "tracks.1"), golden(r#"{"wire": {"node": "track_V1", "port": "frames"}}"#));
    assert_eq!(node(&g, "composite").get("inputs").unwrap().get("settings").unwrap().get("fps"), Some(&Json::Num(25.0)));
    let composite_sources = node(&g, "composite").get("meta").unwrap().get("sources").unwrap().text();
    assert_eq!(composite_sources, format!(r#"{{"marker:marker_1":{}}}"#, site(line + 10, 13)));
}

#[test]
fn a_clips_intro_and_outro_are_its_animations() {
    let g = graph(video([track([title([intro().preset("rise").duration(0.34), outro().preset("fade")]).text("Hi").duration(2)]).name("V1")]));
    assert_eq!(input(&g, "clip_Title", "animIn"), golden(r#"{"preset": "rise", "duration": 0.34}"#));
    assert_eq!(input(&g, "clip_Title", "animOut"), golden(r#"{"preset": "fade", "duration": 1}"#), "a second when the duration is not given");
    let keys: Vec<String> = match node(&g, "clip_Title").get("meta").unwrap().get("sources") {
        Some(Json::Obj(e)) => e.iter().map(|(k, _)| k.clone()).collect(),
        _ => vec![],
    };
    assert_eq!(keys, ["intro", "outro"]);
    let titled = |children: Vec<El>| bad(video([track([title(children).duration(1)])]));
    assert_eq!(titled(vec![intro().preset("wobble")]), "intro(): preset is one of fade, slideL, slideR, slideU, slideD, pop, rise, spin, not \"wobble\"");
    assert_eq!(titled(vec![intro().preset("fade"), intro().preset("pop")]), "title(): a clip has one intro()");
    assert_eq!(titled(vec![outro()]), "outro() names its preset (fade, slideL, slideR, slideU, slideD, pop, rise, spin)");
}

#[test]
fn a_song_declares_the_chain_synth_effects_track_master() {
    let line = line!() + 6;
    let g = graph(
        song([track([
            synth().wave("triangle").attack(0.02),
            reverb().mix(0.5),
            clip([
                note().pitch("C4").start(0).duration(1),
                note().pitch(64).start(1),
            ])
            .start(4),
        ])
        .name("Keys")])
        .name("Loop")
        .tempo(96)
        .time_signature("3/4"),
    );
    assert_eq!(input(&g, "master", "tempo"), golden(r#"[{"atBeat": 0, "bpm": 96}]"#));
    assert_eq!(input(&g, "master", "timeSig"), golden(r#"{"num": 3, "den": 4}"#));
    assert_eq!(input(&g, "master", "audio.1"), golden(r#"{"wire": {"node": "track_Keys", "port": "audio"}}"#));
    assert_eq!(input(&g, "track_Keys", "audio"), golden(r#"{"wire": {"node": "fx_Keys_reverb", "port": "audio"}}"#));
    assert_eq!(input(&g, "fx_Keys_reverb", "audio"), golden(r#"{"wire": {"node": "inst_Keys", "port": "audio"}}"#));
    assert_eq!(input(&g, "inst_Keys", "midi.1"), golden(r#"{"wire": {"node": "clip_Keys_1", "port": "midi"}}"#));
    assert_eq!(
        input(&g, "clip_Keys_1", "notes"),
        golden(r#"[{"id": "clip_Keys_1_n1", "pitch": 60, "start": 0, "dur": 1, "vel": 0.8}, {"id": "clip_Keys_1_n2", "pitch": 64, "start": 1, "dur": 1, "vel": 0.8}]"#)
    );
    let sources = node(&g, "clip_Keys_1").get("meta").unwrap().get("sources").unwrap().text();
    assert_eq!(sources, format!(r#"{{"note:clip_Keys_1_n1":{},"note:clip_Keys_1_n2":{}}}"#, site(line, 17), site(line + 1, 17)));
    assert_eq!(node(&g, "track_Keys").get("meta").unwrap().get("order"), Some(&Json::Num(0.0)));
    assert_eq!(pitch_of(Some(&Json::from("F#3"))), Some(54));
    assert_eq!(pitch_of(Some(&Json::from("Bb2"))), Some(46));
    assert_eq!(pitch_name(61), "C#4");
    assert_eq!(
        input(&graph(song([]).tempo(json(r#"[{"atBeat": 0, "bpm": 90}, {"atBeat": 8, "bpm": 120}]"#))), "master", "tempo"),
        golden(r#"[{"atBeat": 0, "bpm": 90}, {"atBeat": 8, "bpm": 120}]"#)
    );
}

#[test]
fn a_song_holds_audio_clips_a_sampler_and_a_cycle() {
    let g = graph(
        song([
            track([gain().gain(0.5), clip([]).src("media/tone.wav").start(2).length(3).in_(0.25).volume(0.8), clip([]).src("media/tone.wav").name("Again").start(8)]).name("Tone"),
            track([sampler().src("media/tone.wav").root("C5").release(1), clip([note().pitch("E5")])]).name("Bells"),
        ])
        .cycle_start(4)
        .cycle_end(12),
    );
    assert_eq!(input(&g, "master", "cycle"), golden(r#"{"start": 4, "end": 12}"#));
    assert_eq!(input(&g, "player_Tone", "audio.2"), golden(r#"{"wire": {"node": "clip_Again", "port": "audio"}}"#));
    assert_eq!(input(&g, "fx_Tone_gain", "audio"), golden(r#"{"wire": {"node": "player_Tone", "port": "audio"}}"#));
    assert_eq!(
        node(&g, "clip_tone").get("inputs").unwrap().text(),
        golden(r#"{"name": "tone", "src": "media/tone.wav", "start": 2, "length": 3, "offsetSeconds": 0.25, "gain": 0.8, "loop": false}"#)
    );
    assert_eq!(
        input(&g, "inst_Bells", "spec"),
        golden(r#"{"kind": "sampler", "src": "media/tone.wav", "baseNote": 72, "gain": 1, "env": {"attack": 0.01, "decay": 0.15, "sustain": 0.6, "release": 1}}"#)
    );
}

#[test]
fn refuses_what_the_vocabulary_cannot_say() {
    let video_track = |children: Vec<El>| bad(video([track(children)]));
    assert_eq!(video_track(vec![clip([]).src("data:video/mp4;base64,AAAA").duration(1)]), "clip().src(\"data:video/mp4;base64,AAAA\"): src is a path relative to this file, not data:video/mp4;base64,AAAA");
    assert_eq!(video_track(vec![clip([]).src("a.wav").duration(1)]), "clip().src(\"a.wav\"): a sound goes on an audio track (.kind(\"audio\"))");
    assert_eq!(video_track(vec![clip([]).src("a.mp4")]), "clip().src(\"a.mp4\"): give its out (or duration) in seconds");
    assert_eq!(video_track(vec![clip([]).src("a.mp4").out(2).fill("red")]), "clip().src(\"a.mp4\"): fill is not read on a clip()");
    assert_eq!(video_track(vec![clip([]).src("a.mp4").in_(2).out(1)]), "clip().src(\"a.mp4\"): out (1) is after in (2)");
    assert_eq!(video_track(vec![clip([effect([])]).src("a.mp4").duration(1)]), "clip().src(\"a.mp4\"): an effect() names its type");
    assert_eq!(video_track(vec![clip([effect([]).type_("glow").amount("x")]).src("a.mp4").duration(1)]), "effect().type_(\"glow\"): amount is a number, not \"x\"");
    assert_eq!(video_track(vec![title([]).text("T").duration(1).opacity(2)]), "title().text(\"T\"): opacity is at most 1, not 2");
    let song_track = |children: Vec<El>| bad(song([track(children)]));
    assert_eq!(song_track(vec![clip([])]), "track(): its clips need a synth() or a sampler() to play them");
    assert_eq!(song_track(vec![synth(), clip([note().pitch("H2")])]), "note(): pitch is a MIDI number (0–127) or a name (\"C4\", \"F#3\"), not \"H2\"");
    assert_eq!(song_track(vec![clip([]).src("/abs/tone.wav")]), "clip().src(\"/abs/tone.wav\"): src is a path relative to this file, not /abs/tone.wav");
    assert_eq!(song_track(vec![clip([]).src("take.mp4")]), "clip().src(\"take.mp4\"): take.mp4 is not a sound file this studio reads");
    assert_eq!(song_track(vec![synth(), clip([]).src("tone.wav")]), "track(): a track that plays an instrument holds clips of notes; an audio clip (src) goes on a track without one");
    assert_eq!(song_track(vec![sampler().src("a.wav").root("Q")]), "sampler(): root is the pitch the file sounds at, a MIDI number (0–127) or a name (\"A4\"), not \"Q\"");
    assert_eq!(song_track(vec![sampler()]), "sampler() names its sound file (.src(\"media/tone.wav\"), relative to this file)");
    assert_eq!(bad(song([]).cycle_start(4)), "song(): a cycle names both its cycle_start and its cycle_end (beats)");
    assert_eq!(bad(song([]).cycle_start(4).cycle_end(4)), "song(): cycle_end (4) comes after cycle_start (4)");
    assert_eq!(bad(song([]).time_signature("4/5")), "song(): time_signature is \"beats/unit\" (\"4/4\", \"6/8\")");
    assert_eq!(bad(song([]).tempo(json(r#"[{"atBeat": 4, "bpm": 90}]"#))), "song(): tempo is beats per minute (120), or [{ atBeat: 0, bpm: 120 }, …] sorted by beat");
}
