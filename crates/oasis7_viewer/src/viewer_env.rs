const VIEWER_ENV_PREFIX: &str = "OASIS7_VIEWER_";

fn viewer_env_key_with_prefix(key: &str, prefix: &str) -> Option<String> {
    let suffix = key.strip_prefix(VIEWER_ENV_PREFIX)?;
    Some(format!("{prefix}{suffix}"))
}

fn viewer_env_candidates(key: &str) -> Vec<String> {
    let Some(primary) = viewer_env_key_with_prefix(key, VIEWER_ENV_PREFIX) else {
        return vec![key.to_string()];
    };
    let mut candidates = vec![primary.clone()];
    if key != primary {
        candidates.push(key.to_string());
    }
    candidates
}

pub(crate) fn resolve_viewer_env_with<F>(lookup: &F, key: &str) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    for candidate in viewer_env_candidates(key) {
        if let Some(value) = lookup(candidate.as_str()) {
            return Some(value);
        }
    }
    None
}

pub(crate) fn viewer_env_var(key: &str) -> Option<String> {
    resolve_viewer_env_with(&|candidate| std::env::var(candidate).ok(), key)
}

pub(crate) fn viewer_env_present(key: &str) -> bool {
    viewer_env_candidates(key)
        .into_iter()
        .any(|candidate| std::env::var_os(candidate).is_some())
}

#[cfg(test)]
mod tests {
    use super::resolve_viewer_env_with;
    use std::collections::HashMap;

    #[test]
    fn resolve_viewer_env_with_prefers_oasis7_key() {
        let values = HashMap::from([
            ("OASIS7_VIEWER_PANEL_MODE", "observe"),
            ("VIEWER_PANEL_MODE", "free"),
        ]);
        let resolved = resolve_viewer_env_with(
            &|key| values.get(key).map(|value| value.to_string()),
            "OASIS7_VIEWER_PANEL_MODE",
        );
        assert_eq!(resolved.as_deref(), Some("observe"));
    }

    #[test]
    fn resolve_viewer_env_with_ignores_removed_old_brand_key() {
        let values = HashMap::from([("AGENT_WORLD_VIEWER_PANEL_MODE", "legacy")]);
        let resolved = resolve_viewer_env_with(
            &|key| values.get(key).map(|value| value.to_string()),
            "OASIS7_VIEWER_PANEL_MODE",
        );
        assert_eq!(resolved, None);
    }
}
