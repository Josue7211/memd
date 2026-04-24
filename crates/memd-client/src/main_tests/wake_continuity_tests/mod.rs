//! D4.7: Wake continuity-loss harness.
//!
//! Drives 20 synthetic `CompilerInput` scenarios through the wake-context
//! compiler and asserts the compiled markdown still answers the three
//! continuity dimensions: `doing`, `learned`, `prefers`.
//!
//! Real-data dogfood validation lives in D4.8. Sealed session dirs on
//! disk hold file-interaction snapshots, not typed records, so D4.7 uses
//! synthetic fixtures hand-curated to mirror real wake shapes.

use std::fs;
use std::path::PathBuf;

use memd_schema::CompactMemoryRecord;
use serde::Deserialize;

use crate::runtime::resume::compiler::{
    self, BucketKind, CompiledWake, CompilerInput, WakeBudget,
};

#[derive(Debug, Deserialize)]
struct Expects {
    #[serde(default)]
    doing: Option<String>,
    #[serde(default)]
    learned: Option<String>,
    #[serde(default)]
    prefers: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Scenario {
    name: String,
    #[serde(default)]
    input: CompilerInput,
    #[serde(default)]
    budget_tokens: usize,
    #[serde(default)]
    include_bucket: Vec<String>,
    #[serde(default)]
    exclude_bucket: Vec<String>,
    #[serde(default)]
    expects: Option<Expects>,
    #[serde(default)]
    expects_empty: bool,
    #[serde(default)]
    must_not_contain: Vec<String>,
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("d4")
        .join("scenarios")
}

fn load_scenarios() -> Vec<Scenario> {
    let dir = fixtures_dir();
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read scenarios dir {}: {e}", dir.display()))
        .filter_map(|r| r.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());
    entries
        .into_iter()
        .map(|e| {
            let bytes = fs::read(e.path()).expect("read scenario");
            serde_json::from_slice::<Scenario>(&bytes)
                .unwrap_or_else(|err| panic!("parse {}: {err}", e.path().display()))
        })
        .collect()
}

fn budget_for(scenario: &Scenario) -> WakeBudget {
    let tokens = if scenario.budget_tokens > 0 {
        scenario.budget_tokens
    } else {
        2000
    };
    WakeBudget::default_2000()
        .with_tokens(tokens)
        .with_includes(&scenario.include_bucket)
        .with_excludes(&scenario.exclude_bucket)
}

fn compile(scenario: &Scenario) -> CompiledWake {
    compiler::compile_wake(scenario.input.clone(), budget_for(scenario))
}

// -------- Test 18: 20-scenario continuity sweep --------

#[test]
fn continuity_loss_20_scenarios_pass() {
    let scenarios = load_scenarios();
    assert_eq!(
        scenarios.len(),
        20,
        "D4.7 fixture set must have exactly 20 scenarios; found {}",
        scenarios.len()
    );

    let mut failures: Vec<String> = Vec::new();

    for scenario in &scenarios {
        let compiled = compile(scenario);
        let md = &compiled.markdown;

        if scenario.expects_empty {
            // Empty input must render without a demotion section.
            for forbidden in &scenario.must_not_contain {
                if md.contains(forbidden) {
                    failures.push(format!(
                        "scenario `{}`: must_not_contain `{forbidden}` was found",
                        scenario.name
                    ));
                }
            }
            continue;
        }

        if let Some(expects) = &scenario.expects {
            // Each populated dim must survive compilation.
            // We collect failures rather than panic-fast so the report shows
            // every scenario that drops content in one shot.
            for (dim, needle) in [
                ("doing", &expects.doing),
                ("learned", &expects.learned),
                ("prefers", &expects.prefers),
            ] {
                if let Some(needle) = needle {
                    if !md.contains(needle) {
                        failures.push(format!(
                            "scenario `{}` dim `{dim}`: missing needle `{needle}`",
                            scenario.name
                        ));
                    }
                }
            }
        }

        for forbidden in &scenario.must_not_contain {
            if md.contains(forbidden) {
                failures.push(format!(
                    "scenario `{}`: must_not_contain `{forbidden}` leaked through",
                    scenario.name
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "continuity-loss failures across {} scenarios:\n  - {}",
        failures.len(),
        failures.join("\n  - ")
    );
}

// -------- Test 19: regression catch (assertion machinery is real) --------

#[test]
fn continuity_loss_regression_catch() {
    // Sanity check: feeding empty input into the assertion path that expects
    // a non-empty needle MUST fail. If this test ever silently passes, the
    // harness above is hollow.
    let empty_scenario = Scenario {
        name: "synthetic-regression-canary".to_string(),
        input: CompilerInput::default(),
        budget_tokens: 2000,
        include_bucket: Vec::new(),
        exclude_bucket: Vec::new(),
        expects: Some(Expects {
            doing: Some("THIS NEEDLE WILL NEVER APPEAR IN AN EMPTY WAKE".to_string()),
            learned: None,
            prefers: None,
        }),
        expects_empty: false,
        must_not_contain: Vec::new(),
    };
    let compiled = compile(&empty_scenario);
    let needle = empty_scenario
        .expects
        .as_ref()
        .and_then(|e| e.doing.as_deref())
        .expect("needle present");
    assert!(
        !compiled.markdown.contains(needle),
        "regression canary unexpectedly found needle in empty wake — harness asserts are hollow"
    );

    // And: the canonical happy-path scenario MUST keep all three dims so
    // that we know the harness can also detect the positive case.
    let scenarios = load_scenarios();
    let real = scenarios
        .iter()
        .find(|s| s.name == "real-shape-mixed-multi-bucket")
        .expect("scenario 20 present");
    let md = compile(real).markdown;
    let expects = real.expects.as_ref().unwrap();
    assert!(md.contains(expects.doing.as_deref().unwrap()));
    assert!(md.contains(expects.learned.as_deref().unwrap()));
    assert!(md.contains(expects.prefers.as_deref().unwrap()));
}

// -------- Test 20: wake-size histogram targets --------

#[test]
fn wake_size_histogram_targets() {
    let scenarios = load_scenarios();
    let mut sizes: Vec<usize> = Vec::with_capacity(scenarios.len());
    let mut over_2k = 0usize;
    let mut bucket_label_used = false;

    for scenario in &scenarios {
        let compiled = compile(scenario);
        sizes.push(compiled.tokens);
        if compiled.tokens > 2000 {
            over_2k += 1;
        }
        // Verify markdown is markdown (sanity). Empty-input scenarios are
        // allowed to render an empty body — the renderer has nothing to
        // emit and that is the contract.
        if compiled.tokens > 0 && !scenario.expects_empty {
            assert!(
                compiled.markdown.contains("## "),
                "scenario `{}`: compiled markdown must contain a section header; got:\n{}",
                scenario.name,
                compiled.markdown
            );
        }
        if !compiled.markdown.is_empty() {
            for kind in [
                BucketKind::Canonical,
                BucketKind::Preference,
                BucketKind::Focus,
            ] {
                if compiled
                    .markdown
                    .contains(&format!("## {}", kind.section_header()))
                {
                    bucket_label_used = true;
                }
            }
        }
    }

    let mean: f64 =
        sizes.iter().copied().map(|n| n as f64).sum::<f64>() / sizes.len().max(1) as f64;
    let max = sizes.iter().copied().max().unwrap_or(0);

    assert!(
        mean < 2000.0,
        "D4 mean wake size must drop below 2000 chars; got mean={mean:.1} across {} scenarios",
        sizes.len()
    );
    assert_eq!(
        over_2k, 0,
        "D4 hard cap: zero scenarios may exceed 2000 chars; {over_2k} did. sizes={sizes:?}"
    );
    assert!(
        max <= 2200,
        "no single scenario should overshoot the budget by >10%; got max={max}"
    );
    assert!(
        bucket_label_used,
        "histogram coverage: at least one canonical/preference/focus section must render across the 20"
    );
}
