use clap::Parser;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

mod checks;

#[derive(Parser)]
#[command(
    name = "cclint",
    about = "Lint your Claude Code config. Find what's wasting tokens.",
    version
)]
struct Cli {
    /// Project directory to lint (default: current directory)
    #[arg(default_value = ".")]
    path: String,

    /// Also check global config (~/.claude/)
    #[arg(long, default_value = "true")]
    global: bool,

    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Debug, serde::Serialize)]
struct LintResult {
    errors: Vec<Issue>,
    warnings: Vec<Issue>,
    info: Vec<Issue>,
    score: u32,
}

#[derive(Debug, serde::Serialize)]
struct Issue {
    severity: String,
    file: String,
    message: String,
    suggestion: Option<String>,
    token_impact: Option<String>,
}

impl Issue {
    fn error(file: &str, message: &str) -> Self {
        Self {
            severity: "error".into(),
            file: file.into(),
            message: message.into(),
            suggestion: None,
            token_impact: None,
        }
    }
    fn warning(file: &str, message: &str) -> Self {
        Self {
            severity: "warning".into(),
            file: file.into(),
            message: message.into(),
            suggestion: None,
            token_impact: None,
        }
    }
    fn info(file: &str, message: &str) -> Self {
        Self {
            severity: "info".into(),
            file: file.into(),
            message: message.into(),
            suggestion: None,
            token_impact: None,
        }
    }
    fn with_suggestion(mut self, s: &str) -> Self {
        self.suggestion = Some(s.into());
        self
    }
    fn with_impact(mut self, s: &str) -> Self {
        self.token_impact = Some(s.into());
        self
    }
}

fn main() {
    let cli = Cli::parse();
    let project_path = PathBuf::from(&cli.path);

    let global_dir = dirs::home_dir().map(|h| h.join(".claude"));

    let mut issues: Vec<Issue> = Vec::new();

    // Check project CLAUDE.md
    let project_claude_md = project_path.join("CLAUDE.md");
    if project_claude_md.exists() {
        issues.extend(checks::check_claude_md(&project_claude_md, "CLAUDE.md"));
    }

    // Check global CLAUDE.md
    if cli.global {
        if let Some(ref gdir) = global_dir {
            let global_claude_md = gdir.join("CLAUDE.md");
            if global_claude_md.exists() {
                issues.extend(checks::check_claude_md(
                    &global_claude_md,
                    "~/.claude/CLAUDE.md",
                ));
            }

            // Check settings.json
            let settings = gdir.join("settings.json");
            if settings.exists() {
                issues.extend(checks::check_settings(&settings));
            }

            // Check skills
            let skills_dir = gdir.join("skills");
            if skills_dir.exists() {
                issues.extend(checks::check_skills(&skills_dir));
            }

            // Check commands
            let commands_dir = gdir.join("commands");
            if commands_dir.exists() {
                issues.extend(checks::check_commands(&commands_dir));
            }
        }
    }

    // Check project .claude/ directory
    let project_claude_dir = project_path.join(".claude");
    if project_claude_dir.exists() {
        let settings = project_claude_dir.join("settings.json");
        if settings.exists() {
            issues.extend(checks::check_settings(&settings));
        }
    }

    // Calculate score
    let errors: Vec<_> = issues.iter().filter(|i| i.severity == "error").cloned().collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == "warning").cloned().collect();
    let infos: Vec<_> = issues.iter().filter(|i| i.severity == "info").cloned().collect();

    let score = calculate_score(&errors, &warnings);

    let result = LintResult {
        errors,
        warnings,
        info: infos,
        score,
    };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        print_report(&result);
    }

    if !result.errors.is_empty() {
        std::process::exit(1);
    }
}

fn calculate_score(errors: &[Issue], warnings: &[Issue]) -> u32 {
    let base: i32 = 100;
    let penalty = (errors.len() as i32 * 15) + (warnings.len() as i32 * 5);
    (base - penalty).max(0) as u32
}

fn print_report(result: &LintResult) {
    println!();
    println!("{}", "  cclint — Claude Code Config Linter".bold());
    println!(
        "{}",
        "  Find what's wasting tokens in your config.".dimmed()
    );
    println!();

    // Score
    let score_color = if result.score >= 80 {
        "green"
    } else if result.score >= 50 {
        "yellow"
    } else {
        "red"
    };
    let score_str = format!("  Config Health Score: {}/100", result.score);
    match score_color {
        "green" => println!("{}", score_str.green().bold()),
        "yellow" => println!("{}", score_str.yellow().bold()),
        _ => println!("{}", score_str.red().bold()),
    }
    println!();

    if result.errors.is_empty() && result.warnings.is_empty() && result.info.is_empty() {
        println!("{}", "  No issues found. Your config looks good!".green());
        println!();
        return;
    }

    for issue in &result.errors {
        print_issue("ERROR", issue, |s| s.red());
    }
    for issue in &result.warnings {
        print_issue("WARN", issue, |s| s.yellow());
    }
    for issue in &result.info {
        print_issue("INFO", issue, |s| s.cyan());
    }

    println!();
}

fn print_issue(label: &str, issue: &Issue, color_fn: fn(&str) -> colored::ColoredString) {
    println!(
        "  {} {} {}",
        color_fn(&format!("[{}]", label)),
        issue.file.dimmed(),
        issue.message
    );
    if let Some(ref suggestion) = issue.suggestion {
        println!("         {} {}", "fix:".bold(), suggestion);
    }
    if let Some(ref impact) = issue.token_impact {
        println!("         {} {}", "impact:".dimmed(), impact);
    }
}

impl Clone for Issue {
    fn clone(&self) -> Self {
        Self {
            severity: self.severity.clone(),
            file: self.file.clone(),
            message: self.message.clone(),
            suggestion: self.suggestion.clone(),
            token_impact: self.token_impact.clone(),
        }
    }
}
