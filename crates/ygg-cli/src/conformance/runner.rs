use std::time::Instant;

use super::registry::build_cases;

/// Options parsed from CLI for the conformance command.
pub(crate) struct ConformanceOptions {
    pub(crate) list: bool,
    pub(crate) case: Vec<String>,
    pub(crate) tag: Vec<String>,
    pub(crate) fail_fast: bool,
    pub(crate) slowest: usize,
}

pub(crate) type CaseFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>;
pub(crate) type CaseFn = fn() -> CaseFuture;

/// A single conformance case with metadata.
pub(crate) struct ConformanceCase {
    pub(crate) id: &'static str,
    pub(crate) tags: &'static [&'static str],
    pub(crate) run: CaseFn,
}

/// Format a duration for display.
fn fmt_duration(d: std::time::Duration) -> String {
    if d < std::time::Duration::from_millis(1) {
        format!("{:.0}µs", d.as_micros())
    } else if d < std::time::Duration::from_secs(1) {
        format!("{:.1}ms", d.as_secs_f64() * 1000.0)
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}

pub(crate) async fn run(opts: ConformanceOptions) -> anyhow::Result<()> {
    let all_cases = build_cases();

    // --- list mode: print ids + tags and exit ---
    if opts.list {
        for case in &all_cases {
            let tags = case.tags.join(", ");
            println!("{:<55} [{}]", case.id, tags);
        }
        println!("total: {} cases", all_cases.len());
        return Ok(());
    }

    // --- filter cases ---
    let selected: Vec<&ConformanceCase> = all_cases
        .iter()
        .filter(|case| {
            // --case <substring>: case id must contain at least one matching substring
            let case_match =
                opts.case.is_empty() || opts.case.iter().any(|p| case.id.contains(p.as_str()));
            // --tag <tag>: case must have at least one matching tag
            let tag_match =
                opts.tag.is_empty() || opts.tag.iter().any(|t| case.tags.contains(&t.as_str()));
            case_match && tag_match
        })
        .collect();

    if selected.is_empty() {
        eprintln!("no conformance cases matched the given filters");
        anyhow::bail!("no cases selected");
    }

    // --- execute cases ---
    let mut results: Vec<(&ConformanceCase, anyhow::Result<()>, std::time::Duration)> = Vec::new();
    let mut failed = false;

    for case in &selected {
        let start = Instant::now();
        let result = (case.run)().await;
        let elapsed = start.elapsed();
        let ok = result.is_ok();
        if !ok {
            failed = true;
        }
        match &result {
            Ok(()) => println!("{:<55} PASS  {}", case.id, fmt_duration(elapsed)),
            Err(err) => println!("{:<55} FAIL  {}  {}", case.id, fmt_duration(elapsed), err),
        }
        results.push((case, result, elapsed));
        if !ok && opts.fail_fast {
            break;
        }
    }

    // --- slowest report ---
    if opts.slowest > 0 && results.len() > 1 {
        let mut timed: Vec<(&ConformanceCase, std::time::Duration)> =
            results.iter().map(|(case, _, dur)| (*case, *dur)).collect();
        timed.sort_by(|a, b| b.1.cmp(&a.1));
        let n = opts.slowest.min(timed.len());
        println!();
        println!("slowest {} cases:", n);
        for (case, elapsed) in timed.iter().take(n) {
            let was_ok = results.iter().any(|(c, r, _)| c.id == case.id && r.is_ok());
            let status = if was_ok { "PASS" } else { "FAIL" };
            println!("  {:<55} {}  {}", case.id, status, fmt_duration(*elapsed));
        }
    }

    // --- summary ---
    let pass_count = results.iter().filter(|r| r.1.is_ok()).count();
    let fail_count = results.len() - pass_count;
    if failed {
        println!();
        anyhow::bail!(
            "conformance: {}/{} passed, {} failed",
            pass_count,
            results.len(),
            fail_count
        );
    }
    println!(
        "\nconformance: ok ({} cases, {})",
        results.len(),
        fmt_duration(results.iter().map(|r| r.2).sum::<std::time::Duration>())
    );
    Ok(())
}
