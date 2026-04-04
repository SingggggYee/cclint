use crate::Issue;
use std::fs;
use std::path::Path;

/// Approximate tokens from byte count (1 token ≈ 4 chars)
fn estimate_tokens(text: &str) -> usize {
    text.len() / 4
}

/// Check CLAUDE.md for common issues
pub fn check_claude_md(path: &Path, display_name: &str) -> Vec<Issue> {
    let mut issues = vec![];

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![Issue::error(display_name, "Cannot read file")],
    };

    let tokens = estimate_tokens(&content);
    let lines = content.lines().count();

    // Size checks
    if tokens > 5000 {
        issues.push(
            Issue::error(display_name, &format!("Very large CLAUDE.md (~{} tokens, {} lines). This is loaded into context every turn.", tokens, lines))
                .with_suggestion("Split into smaller files or move reference content to skills/")
                .with_impact(&format!("~{} tokens added to every API call", tokens)),
        );
    } else if tokens > 2000 {
        issues.push(
            Issue::warning(display_name, &format!("Large CLAUDE.md (~{} tokens, {} lines)", tokens, lines))
                .with_suggestion("Consider trimming rules you rarely use")
                .with_impact(&format!("~{} tokens added to every API call", tokens)),
        );
    } else {
        issues.push(Issue::info(
            display_name,
            &format!("CLAUDE.md size OK (~{} tokens, {} lines)", tokens, lines),
        ));
    }

    // Content checks
    let lower = content.to_lowercase();

    // Redundant with defaults
    if lower.contains("be concise") && lower.contains("no preamble") && lower.contains("no summary") {
        issues.push(
            Issue::warning(display_name, "Multiple verbosity rules (be concise + no preamble + no summary)")
                .with_suggestion("Claude Code already defaults to concise output. One rule is enough, or remove entirely.")
                .with_impact("Redundant rules add tokens without benefit"),
        );
    }

    // Contradictory rules
    if lower.contains("always add comments") && lower.contains("minimal comments") {
        issues.push(
            Issue::error(display_name, "Contradictory rules: 'always add comments' vs 'minimal comments'")
                .with_suggestion("Pick one approach and remove the other"),
        );
    }

    if lower.contains("always add jsdoc") && lower.contains("no jsdoc") {
        issues.push(
            Issue::error(display_name, "Contradictory rules: 'always add JSDoc' vs 'no JSDoc'")
                .with_suggestion("Pick one approach and remove the other"),
        );
    }

    // Duplicate sections
    let headers: Vec<&str> = content
        .lines()
        .filter(|l| l.starts_with('#'))
        .map(|l| l.trim())
        .collect();
    let mut seen_headers = std::collections::HashSet::new();
    for h in &headers {
        let normalized = h.to_lowercase();
        if !seen_headers.insert(normalized.clone()) {
            issues.push(
                Issue::warning(display_name, &format!("Duplicate section header: {}", h))
                    .with_suggestion("Merge duplicate sections to reduce size"),
            );
        }
    }

    // Very long lines (often copy-pasted content)
    let long_lines = content.lines().filter(|l| l.len() > 500).count();
    if long_lines > 0 {
        issues.push(
            Issue::warning(display_name, &format!("{} lines over 500 chars (likely copy-pasted content)", long_lines))
                .with_suggestion("Move large reference content to a separate file in skills/")
                .with_impact("Long lines inflate context size unnecessarily"),
        );
    }

    // Generic/vague rules that don't help
    let vague_patterns = [
        ("write clean code", "Too vague. Specify what 'clean' means (naming conventions, max function length, etc.)"),
        ("follow best practices", "Too vague. Which practices? Be specific or remove."),
        ("be helpful", "Claude already tries to be helpful. This wastes tokens."),
        ("you are an expert", "Role-playing prompts are unnecessary in CLAUDE.md. Claude Code already knows it's a coding assistant."),
        ("you are a senior", "Role-playing prompts waste tokens. Remove."),
    ];

    for (pattern, suggestion) in vague_patterns {
        if lower.contains(pattern) {
            issues.push(
                Issue::warning(display_name, &format!("Vague rule detected: '{}'", pattern))
                    .with_suggestion(suggestion),
            );
        }
    }

    // Emoji usage
    if content.chars().any(|c| c >= '\u{1F600}' && c <= '\u{1F9FF}') {
        issues.push(
            Issue::info(display_name, "Contains emoji. Each emoji uses 2-3 tokens.")
                .with_suggestion("Remove emoji if they don't serve a functional purpose"),
        );
    }

    // No structure (no headers)
    if headers.is_empty() && lines > 10 {
        issues.push(
            Issue::warning(display_name, "No markdown headers found in a file with {} lines")
                .with_suggestion("Add headers (## Section) to help Claude parse your rules efficiently"),
        );
    }

    issues
}

/// Check settings.json for hook issues
pub fn check_settings(path: &Path) -> Vec<Issue> {
    let mut issues = vec![];
    let display = path.to_string_lossy().to_string();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![Issue::error(&display, "Cannot read settings.json")],
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => return vec![Issue::error(&display, &format!("Invalid JSON: {}", e))],
    };

    // Check hooks
    if let Some(hooks) = json.get("hooks").and_then(|h| h.as_object()) {
        let hook_count: usize = hooks
            .values()
            .filter_map(|v| v.as_array())
            .map(|a| a.len())
            .sum();

        if hook_count > 10 {
            issues.push(
                Issue::warning("settings.json", &format!("{} hooks configured. Each hook adds latency to every tool call.", hook_count))
                    .with_suggestion("Remove hooks you don't actively use"),
            );
        }

        // Check for hooks without matchers (run on EVERY tool call)
        for (event, handlers) in hooks {
            if let Some(arr) = handlers.as_array() {
                for handler in arr {
                    if handler.get("matcher").is_none() && event == "PreToolUse" {
                        issues.push(
                            Issue::warning("settings.json", "PreToolUse hook without matcher runs on EVERY tool call")
                                .with_suggestion("Add a matcher to limit which tools trigger this hook"),
                        );
                    }
                }
            }
        }
    }

    // Check dangerous settings
    if json.get("skipDangerousModePermissionPrompt") == Some(&serde_json::Value::Bool(true)) {
        issues.push(
            Issue::info("settings.json", "skipDangerousModePermissionPrompt is enabled")
                .with_suggestion("This skips safety confirmations. Make sure you understand the risks."),
        );
    }

    issues
}

/// Check skills directory
pub fn check_skills(dir: &Path) -> Vec<Issue> {
    let mut issues = vec![];

    let entries: Vec<_> = match fs::read_dir(dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return issues,
    };

    let skill_count = entries
        .iter()
        .filter(|e| e.path().is_dir())
        .count();

    if skill_count > 20 {
        issues.push(
            Issue::warning("~/.claude/skills/", &format!("{} skills installed. Each skill's description is loaded into context.", skill_count))
                .with_suggestion("Remove skills you don't use. Each unused skill wastes ~50-200 tokens per session.")
                .with_impact(&format!("~{}-{} tokens from skill descriptions alone", skill_count * 50, skill_count * 200)),
        );
    } else if skill_count > 0 {
        issues.push(Issue::info(
            "~/.claude/skills/",
            &format!("{} skills installed", skill_count),
        ));
    }

    // Check for skills without SKILL.md
    for entry in &entries {
        if entry.path().is_dir() {
            let skill_md = entry.path().join("SKILL.md");
            if !skill_md.exists() {
                issues.push(
                    Issue::warning(
                        &format!("skills/{}/", entry.file_name().to_string_lossy()),
                        "Skill directory without SKILL.md",
                    )
                    .with_suggestion("Remove this directory or add a SKILL.md"),
                );
            }
        }
    }

    issues
}

/// Check commands directory
pub fn check_commands(dir: &Path) -> Vec<Issue> {
    let mut issues = vec![];

    let entries: Vec<_> = match fs::read_dir(dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return issues,
    };

    let cmd_count = entries.len();

    if cmd_count > 30 {
        issues.push(
            Issue::warning("~/.claude/commands/", &format!("{} commands configured. Commands are listed in context when relevant.", cmd_count))
                .with_suggestion("Remove commands you don't use"),
        );
    } else if cmd_count > 0 {
        issues.push(Issue::info(
            "~/.claude/commands/",
            &format!("{} commands configured", cmd_count),
        ));
    }

    // Check for very large command files
    for entry in &entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let tokens = estimate_tokens(&content);
                if tokens > 1000 {
                    issues.push(
                        Issue::warning(
                            &format!("commands/{}", entry.file_name().to_string_lossy()),
                            &format!("Large command file (~{} tokens)", tokens),
                        )
                        .with_suggestion("Consider splitting into smaller commands")
                        .with_impact(&format!("~{} tokens when this command is loaded", tokens)),
                    );
                }
            }
        }
    }

    issues
}
