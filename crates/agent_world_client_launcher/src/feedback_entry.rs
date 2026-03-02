use serde::Serialize;
use serde_json::Value;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const DEFAULT_FEEDBACK_DIR: &str = "feedback";
const FEEDBACK_LOG_SNAPSHOT_LIMIT: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FeedbackKind {
    Bug,
    Suggestion,
}

impl FeedbackKind {
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Suggestion => "suggestion",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FeedbackDraft {
    pub(crate) kind: FeedbackKind,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) output_dir: String,
}

impl Default for FeedbackDraft {
    fn default() -> Self {
        Self {
            kind: FeedbackKind::Bug,
            title: String::new(),
            description: String::new(),
            output_dir: DEFAULT_FEEDBACK_DIR.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeedbackDraftIssue {
    TitleRequired,
    DescriptionRequired,
    OutputDirRequired,
}

#[derive(Debug, Serialize)]
struct FeedbackReport {
    kind: FeedbackKind,
    title: String,
    description: String,
    created_at: String,
    launcher_config: Value,
    recent_logs: Vec<String>,
}

pub(crate) fn validate_feedback_draft(draft: &FeedbackDraft) -> Vec<FeedbackDraftIssue> {
    let mut issues = Vec::new();
    if draft.title.trim().is_empty() {
        issues.push(FeedbackDraftIssue::TitleRequired);
    }
    if draft.description.trim().is_empty() {
        issues.push(FeedbackDraftIssue::DescriptionRequired);
    }
    if draft.output_dir.trim().is_empty() {
        issues.push(FeedbackDraftIssue::OutputDirRequired);
    }
    issues
}

pub(crate) fn collect_recent_logs(logs: &VecDeque<String>) -> Vec<String> {
    let keep = logs.len().saturating_sub(FEEDBACK_LOG_SNAPSHOT_LIMIT);
    logs.iter().skip(keep).cloned().collect()
}

pub(crate) fn submit_feedback_report(
    draft: &FeedbackDraft,
    launcher_config: Value,
    recent_logs: Vec<String>,
) -> Result<PathBuf, String> {
    let output_dir = draft.output_dir.trim();
    if output_dir.is_empty() {
        return Err("feedback output directory cannot be empty".to_string());
    }

    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path)
        .map_err(|err| format!("create feedback directory `{output_dir}` failed: {err}"))?;

    let now = SystemTime::now();
    let report = FeedbackReport {
        kind: draft.kind,
        title: draft.title.trim().to_string(),
        description: draft.description.trim().to_string(),
        created_at: format_rfc3339_utc(now),
        launcher_config,
        recent_logs,
    };

    let file_name = format!(
        "{}-{}.json",
        format_filename_timestamp(now),
        draft.kind.slug()
    );
    let file_path = output_path.join(file_name);
    let content = serde_json::to_vec_pretty(&report)
        .map_err(|err| format!("serialize feedback report failed: {err}"))?;
    fs::write(&file_path, content).map_err(|err| {
        format!(
            "write feedback report `{}` failed: {err}",
            file_path.display()
        )
    })?;
    Ok(file_path)
}

fn unix_seconds(now: SystemTime) -> i64 {
    match now.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().min(i64::MAX as u64) as i64,
        Err(_) => 0,
    }
}

fn format_rfc3339_utc(now: SystemTime) -> String {
    let unix = unix_seconds(now);
    let (year, month, day, hour, minute, second) = split_unix_time(unix);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn format_filename_timestamp(now: SystemTime) -> String {
    let unix = unix_seconds(now);
    let (year, month, day, hour, minute, second) = split_unix_time(unix);
    format!("{year:04}{month:02}{day:02}T{hour:02}{minute:02}{second:02}Z")
}

fn split_unix_time(unix_seconds: i64) -> (i32, u32, u32, u32, u32, u32) {
    let days = unix_seconds.div_euclid(86_400);
    let seconds_of_day = unix_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = (seconds_of_day / 3_600) as u32;
    let minute = ((seconds_of_day % 3_600) / 60) as u32;
    let second = (seconds_of_day % 60) as u32;
    (year, month, day, hour, minute, second)
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::{
        collect_recent_logs, format_filename_timestamp, submit_feedback_report,
        validate_feedback_draft, FeedbackDraft, FeedbackDraftIssue, FeedbackKind,
    };
    use std::collections::VecDeque;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn validate_feedback_draft_reports_missing_required_fields() {
        let draft = FeedbackDraft {
            kind: FeedbackKind::Bug,
            title: " ".to_string(),
            description: "".to_string(),
            output_dir: "".to_string(),
        };

        let issues = validate_feedback_draft(&draft);
        assert!(issues.contains(&FeedbackDraftIssue::TitleRequired));
        assert!(issues.contains(&FeedbackDraftIssue::DescriptionRequired));
        assert!(issues.contains(&FeedbackDraftIssue::OutputDirRequired));
    }

    #[test]
    fn collect_recent_logs_keeps_latest_entries_only() {
        let mut logs = VecDeque::new();
        for index in 0..220 {
            logs.push_back(format!("line-{index}"));
        }

        let snapshot = collect_recent_logs(&logs);
        assert_eq!(snapshot.len(), 200);
        assert_eq!(snapshot.first().map(String::as_str), Some("line-20"));
        assert_eq!(snapshot.last().map(String::as_str), Some("line-219"));
    }

    #[test]
    fn submit_feedback_report_writes_json_bundle() {
        let temp_dir =
            std::env::temp_dir().join(format!("agent-world-feedback-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let draft = FeedbackDraft {
            kind: FeedbackKind::Suggestion,
            title: "Need better explanation".to_string(),
            description: "Mission progress should explain step conditions".to_string(),
            output_dir: temp_dir.to_string_lossy().to_string(),
        };

        let path = submit_feedback_report(
            &draft,
            serde_json::json!({"scenario": "llm_bootstrap"}),
            vec!["[stdout] launcher started".to_string()],
        )
        .expect("feedback report should be written");

        let data = std::fs::read_to_string(&path).expect("feedback report should exist");
        assert!(data.contains("\"kind\": \"suggestion\""));
        assert!(data.contains("\"scenario\": \"llm_bootstrap\""));
        assert!(data.contains("launcher started"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn format_filename_timestamp_uses_compact_utc_shape() {
        let timestamp = format_filename_timestamp(UNIX_EPOCH + Duration::from_secs(1_700_000_001));
        assert_eq!(timestamp.len(), 16);
        assert!(timestamp.ends_with('Z'));
        assert!(timestamp.contains('T'));
    }
}
