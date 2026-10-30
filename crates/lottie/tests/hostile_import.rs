//! Hostile-input tests: importing malformed or extreme Lottie documents and dotLottie archives
//! must return `Ok` or `Err`, never panic, hang or abort.

use effectcraft_lottie::{import, zip};
use effectcraft_project::Project;

fn run(bytes: &[u8]) {
    let mut p = Project::default();
    let mut store = |_: &effectcraft_lottie::ImportedImage| None;
    let _ = import(&mut p, bytes, "Hostile", "", &mut store);
}

fn run_json(s: &str) {
    run(s.as_bytes());
}

#[test]
fn extreme_header_numbers() {
    let nums = ["0", "-1", "1e308", "-1e308", "1e-308", "4294967296", "1e18", "0.5", "-0.0"];
    for w in nums {
        for fr in nums {
            for op in nums {
                run_json(&format!(r#"{{"w":{w},"h":{w},"fr":{fr},"ip":0,"op":{op},"layers":[]}}"#));
            }
        }
    }
}

#[test]
fn wrong_types_everywhere() {
    let vals = ["null", "true", "\"x\"", "[]", "{}", "[[[]]]", "-1", "1e400", "{\"a\":{}}"];
    for v in vals {
        run_json(&format!(r#"{{"w":{v},"h":{v},"fr":{v},"ip":{v},"op":{v},"layers":{v},"assets":{v},"fonts":{v},"markers":{v}}}"#));
        run_json(&format!(
            r#"{{"w":100,"h":100,"fr":30,"ip":0,"op":30,"assets":[{v}],"layers":[{{"ty":{v},"ind":{v},"parent":{v},"ks":{v},"shapes":{v},"masksProperties":{v},"ef":{v},"t":{v},"refId":{v},"ip":{v},"op":{v},"st":{v}}}]}}"#
        ));
    }
}

#[test]
fn self_and_mutually_referencing_precomps() {
    run_json(
        r#"{"w":100,"h":100,"fr":30,"ip":0,"op":30,
        "assets":[{"id":"a","layers":[{"ty":0,"refId":"a","w":10,"h":10,"ip":0,"op":30,"st":0}]},
                  {"id":"b","layers":[{"ty":0,"refId":"c","w":10,"h":10,"ip":0,"op":30,"st":0}]},
                  {"id":"c","layers":[{"ty":0,"refId":"b","w":10,"h":10,"ip":0,"op":30,"st":0}]}],
        "layers":[{"ty":0,"refId":"a","w":10,"h":10,"ip":0,"op":30,"st":0},
                  {"ty":0,"refId":"b","w":10,"h":10,"ip":0,"op":30,"st":0}]}"#,
    );
}

#[test]
fn layer_parent_cycles_and_dangling_parents() {
    run_json(
        r#"{"w":100,"h":100,"fr":30,"ip":0,"op":30,"layers":[
        {"ty":3,"ind":1,"parent":2,"ip":0,"op":30,"st":0},
        {"ty":3,"ind":2,"parent":1,"ip":0,"op":30,"st":0},
        {"ty":3,"ind":3,"parent":3,"ip":0,"op":30,"st":0},
        {"ty":3,"ind":4,"parent":99,"td":1,"tt":9,"ip":0,"op":30,"st":0}]}"#,
    );
}

#[test]
fn deeply_nested_json() {
    let depth = 100_000;
    run(format!("{}{}", "[".repeat(depth), "]".repeat(depth)).as_bytes());
    run(format!(r#"{{"w":1,"h":1,"layers":{}{}}}"#, "[".repeat(depth), "]".repeat(depth)).as_bytes());
}

#[test]
fn junk_bytes_and_truncated_archives() {
    run(b"");
    run(b"PK");
    run(b"PK\x03\x04");
    run(&[0xff; 64]);
    let good = zip::store(&[("animations/a.json".into(), br#"{"w":10,"h":10,"fr":30,"ip":0,"op":30,"layers":[]}"#.to_vec())]);
    for cut in 0..good.len() {
        run(&good[..cut]);
        let _ = zip::read_stored(&good[..cut]);
    }
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..2000 {
        let mut z = good.clone();
        for _ in 0..4 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let i = (seed >> 33) as usize % z.len();
            z[i] = (seed >> 11) as u8;
        }
        run(&z);
        let _ = zip::read_stored(&z);
    }
}

#[test]
fn archive_entry_sizes_larger_than_the_data() {
    let mut z = zip::store(&[("a.json".into(), b"{}".to_vec())]);
    z[18..22].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(zip::read_stored(&z).is_empty());
    z[26..28].copy_from_slice(&u16::MAX.to_le_bytes());
    assert!(zip::read_stored(&z).is_empty());
}

#[test]
fn bad_embedded_images_and_fonts() {
    run_json(
        r#"{"w":100,"h":100,"fr":30,"ip":0,"op":30,
        "fonts":{"list":[{"fName":1},{"fFamily":[]},{}]},
        "assets":[{"id":"i","w":1e9,"h":-5,"p":"data:image/png;base64,!!!!","e":1},
                  {"id":"j","w":0,"h":0,"p":"data:;base64,","u":"../../../etc/","e":0},
                  {"id":"k","p":"C:\x\y.png","u":"\\server\share\\"}],
        "layers":[{"ty":2,"refId":"i","ip":0,"op":30,"st":0},{"ty":2,"refId":"j","ip":0,"op":30,"st":0},{"ty":2,"refId":"k","ip":0,"op":30,"st":0}]}"#,
    );
}
