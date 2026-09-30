//! Bounded, conservative count evidence from one pytest terminal summary.
//!
//! Counts intentionally cover reported passed + failed tests only. Pytest
//! skips, collection/fixture errors and deselections are not reclassified as
//! executed successes or failures; the real process exit remains authoritative.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PytestTestRunMetadata {
    pub tests_detected: bool,
    pub tests_run_count: Option<u64>,
    pub tests_passed: Option<u64>,
    pub tests_failed: Option<u64>,
    pub zero_tests_run: Option<bool>,
}

/// Recognize pytest only from the typed executable and argument vector.
pub fn is_supported_pytest_argv(executable: &str, args: &[String]) -> bool {
    let basename = executable.rsplit(['/', '\\']).next().unwrap_or_default();
    if basename == "pytest" {
        return true;
    }
    let Some(suffix) = basename.strip_prefix("python") else {
        return false;
    };
    if !suffix.is_empty() && !suffix.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
        return false;
    }
    let mut args = args.iter().map(String::as_str);
    let mut option = args.next();
    while matches!(option, Some("-B" | "-u" | "-I" | "-E" | "-s" | "-S")) {
        option = args.next();
    }
    option == Some("-m") && args.next() == Some("pytest")
}

/// Parse a *completed, untruncated* execution's separate bounded stdout and
/// stderr tails. Exactly one terminal line may supply counts. Multiple or
/// malformed pytest-like terminal lines are ambiguous and yield no counts.
pub fn parse_pytest_terminal_test_counts(
    stdout: &str,
    stderr: &str,
) -> Option<PytestTestRunMetadata> {
    let mut parsed = None;
    for output in [stdout, stderr] {
        for raw in output.lines() {
            let line = raw.trim().trim_end_matches('\r');
            if !pytest_summary_candidate(line) {
                continue;
            }
            if parsed.is_some() {
                return None;
            }
            parsed = Some(parse_pytest_summary_line(line)?);
        }
    }
    parsed
}

fn pytest_summary_candidate(line: &str) -> bool {
    line.contains(" in ")
        && [" passed", " failed", " skipped", " error", " deselected"]
            .into_iter()
            .any(|marker| line.contains(marker))
}

fn parse_pytest_summary_line(line: &str) -> Option<PytestTestRunMetadata> {
    // The usual pytest banner is equals-fenced; its -q/-qq plain final line
    // is also accepted after typed argv proves direct pytest intent.
    let banner = line.starts_with("==") && line.ends_with("==");
    if line.contains('=') && !banner {
        return None;
    }
    let content = if banner {
        line.trim_matches('=').trim()
    } else {
        line
    };
    if content.len() > 1024 {
        return None;
    }
    let (categories, duration) = content.rsplit_once(" in ")?;
    // Pytest emits the wall clock in parentheses for longer test suites,
    // e.g. `in 109.10s (0:01:49)`; that suffix is display evidence, not an
    // additional count or execution result.
    let seconds_duration = if let Some((seconds, clock)) = duration.split_once(" (") {
        let clock = clock.strip_suffix(')')?;
        let mut parts = clock.split(':');
        let _hours = parts.next()?.parse::<u64>().ok()?;
        let minutes = parts.next()?.parse::<u64>().ok()?;
        let seconds_clock = parts.next()?.parse::<u64>().ok()?;
        if parts.next().is_some() || minutes >= 60 || seconds_clock >= 60 {
            return None;
        }
        seconds
    } else {
        duration
    };
    let seconds = seconds_duration.strip_suffix('s')?;
    if seconds.is_empty()
        || seconds.starts_with('.')
        || seconds.ends_with('.')
        || seconds.matches('.').count() > 1
        || !seconds.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        || !seconds.parse::<f64>().ok()?.is_finite()
    {
        return None;
    }
    let mut passed = None;
    let mut failed = None;
    let mut other = false;
    let mut seen = std::collections::HashSet::new();
    for fragment in categories.split(", ") {
        let mut words = fragment.split_whitespace();
        let count = words.next()?.parse::<u64>().ok()?;
        let kind = words.next()?;
        if words.next().is_some() || !seen.insert(kind) {
            return None;
        }
        match kind {
            "passed" => passed = Some(count),
            "failed" => failed = Some(count),
            "skipped" | "error" | "errors" | "deselected" | "warning" | "warnings" => {
                other |= count > 0;
            }
            // xfail/xpass and reruns have distinct execution semantics: do
            // not publish partial counts under a supposedly complete total.
            _ => return None,
        }
    }
    let run_count = passed
        .or(failed)
        .map(|_| passed.unwrap_or(0).checked_add(failed.unwrap_or(0)))
        .flatten();
    Some(PytestTestRunMetadata {
        tests_detected: true,
        tests_run_count: run_count,
        tests_passed: run_count.map(|_| passed.unwrap_or(0)),
        tests_failed: run_count.map(|_| failed.unwrap_or(0)),
        // Skip/collection-error-only summaries do not prove zero execution.
        zero_tests_run: run_count.and_then(|count| (count > 0 || !other).then_some(count == 0)),
    })
}

#[cfg(test)]
mod tests {
    use super::{is_supported_pytest_argv, parse_pytest_terminal_test_counts};

    #[test]
    fn direct_module_and_pytest_commands_only() {
        for command in [
            "python -m pytest -q",
            "python3 -B -m pytest -q -p no:cacheprovider tests/test_workflow.py",
            "/usr/bin/python3.11 -m pytest tests/test_api.py",
            "/tmp/venv/bin/pytest -q",
        ] {
            let words: Vec<_> = command.split_whitespace().collect();
            let args = words[1..]
                .iter()
                .map(|word| word.to_string())
                .collect::<Vec<_>>();
            assert!(is_supported_pytest_argv(words[0], &args), "{command}");
        }
        for command in [
            "python -m unittest discover",
            "python -c print(2 passed in 0.1s)",
            "python -m pytest…",
            "sh -c python -m pytest",
        ] {
            let words: Vec<_> = command.split_whitespace().collect();
            let args = words[1..]
                .iter()
                .map(|word| word.to_string())
                .collect::<Vec<_>>();
            assert!(!is_supported_pytest_argv(words[0], &args), "{command}");
        }
    }

    #[test]
    fn terminal_summary_counts_pass_fail_only_and_preserves_skip_error_uncertainty() {
        let counts = parse_pytest_terminal_test_counts(
            "..Fs [100%]\n======== 2 passed, 1 failed, 1 skipped, 1 error in 0.12s ========\n",
            "",
        )
        .unwrap();
        assert!(counts.tests_detected);
        assert_eq!(counts.tests_run_count, Some(3));
        assert_eq!(counts.tests_passed, Some(2));
        assert_eq!(counts.tests_failed, Some(1));
        assert_eq!(counts.zero_tests_run, Some(false));

        let skipped = parse_pytest_terminal_test_counts("2 skipped in 0.12s\n", "").unwrap();
        assert_eq!(skipped.tests_run_count, None);
        assert_eq!(skipped.zero_tests_run, None);

        let long_run = parse_pytest_terminal_test_counts(
            "============= 581 passed in 109.10s (0:01:49) =============\n",
            "",
        )
        .unwrap();
        assert_eq!(long_run.tests_run_count, Some(581));
    }

    #[test]
    fn absent_malformed_ambiguous_and_unsupported_summaries_do_not_claim_counts() {
        for stdout in [
            "2 passed\n",
            "=== 2 passed in missing ===\n",
            "=== 2 passed in 109.10s (0:91:49) ===\n",
            "=== 2 passed in 109.10s (bad:01:49) ===\n",
            "2 passed, nope failed in 0.1s\n",
            "2 passed in 999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999s\n",
            "1 passed in 0.1s\n2 passed in 0.2s\n",
            "2 xpassed, 1 failed in 0.1s\n",
            "Collected 2 items, 2 passed in 0.1s\n",
        ] {
            assert_eq!(
                parse_pytest_terminal_test_counts(stdout, ""),
                None,
                "{stdout}"
            );
        }
    }
}
