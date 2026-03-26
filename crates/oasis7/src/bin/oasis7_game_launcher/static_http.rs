use super::*;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub(super) fn handle_http_connection(
    mut stream: TcpStream,
    root_dir: &Path,
    deployment_mode: DeploymentMode,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| format!("failed to set read timeout: {err}"))?;

    let mut buffer = [0u8; 8192];
    let bytes = stream
        .read(&mut buffer)
        .map_err(|err| format!("failed to read request: {err}"))?;
    if bytes == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes]);
    let Some(line) = request.lines().next() else {
        write_http_response(&mut stream, 400, "text/plain", b"Bad Request", false)
            .map_err(|err| format!("failed to write 400 response: {err}"))?;
        return Ok(());
    };

    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");

    let head_only = method.eq_ignore_ascii_case("HEAD");
    if !method.eq_ignore_ascii_case("GET") && !head_only {
        write_http_response(&mut stream, 405, "text/plain", b"Method Not Allowed", false)
            .map_err(|err| format!("failed to write 405 response: {err}"))?;
        return Ok(());
    }

    let resolved = match resolve_static_asset_path(root_dir, target) {
        Ok(resolved) => resolved,
        Err(_) => {
            write_http_response(&mut stream, 400, "text/plain", b"Bad Request", head_only)
                .map_err(|err| format!("failed to write 400 response: {err}"))?;
            return Ok(());
        }
    };

    match resolved {
        Some(path) => {
            let body = fs::read(&path).map_err(|err| {
                format!("failed to read static asset `{}`: {err}", path.display())
            })?;
            let viewer_auth_bootstrap =
                resolve_viewer_auth_bootstrap_for_embedded_server(deployment_mode);
            let body = sanitize_index_html_for_embedded_server(
                path.as_path(),
                body.as_slice(),
                viewer_auth_bootstrap.as_ref(),
            );
            write_http_response(
                &mut stream,
                200,
                content_type_for_path(path.as_path()),
                body.as_slice(),
                head_only,
            )
            .map_err(|err| format!("failed to write 200 response: {err}"))?;
        }
        None => {
            write_http_response(&mut stream, 404, "text/plain", b"Not Found", head_only)
                .map_err(|err| format!("failed to write 404 response: {err}"))?;
        }
    }

    Ok(())
}

pub(super) fn resolve_static_asset_path(
    root_dir: &Path,
    raw_target: &str,
) -> Result<Option<PathBuf>, String> {
    let path_only = raw_target
        .split('?')
        .next()
        .unwrap_or(raw_target)
        .split('#')
        .next()
        .unwrap_or(raw_target);

    let relative = sanitize_relative_request_path(path_only)?;
    let direct_path = if relative.as_os_str().is_empty() {
        root_dir.join("index.html")
    } else {
        root_dir.join(relative.as_path())
    };

    if direct_path.is_file() {
        return Ok(Some(direct_path));
    }

    let has_extension = Path::new(path_only)
        .file_name()
        .and_then(|name| Path::new(name).extension())
        .is_some();
    if !has_extension {
        let spa_index = root_dir.join("index.html");
        if spa_index.is_file() {
            return Ok(Some(spa_index));
        }
    }

    Ok(None)
}

pub(super) fn sanitize_relative_request_path(raw_path: &str) -> Result<PathBuf, String> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Ok(PathBuf::new());
    }

    let normalized = trimmed.strip_prefix('/').unwrap_or(trimmed);
    let mut cleaned = PathBuf::new();
    for segment in normalized.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." || segment.contains('\\') {
            return Err("path traversal is not allowed".to_string());
        }
        cleaned.push(segment);
    }

    Ok(cleaned)
}

pub(super) fn content_type_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("map") => "application/json; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

pub(super) fn sanitize_index_html_for_embedded_server(
    path: &Path,
    body: &[u8],
    viewer_auth_bootstrap: Option<&ViewerAuthBootstrap>,
) -> Vec<u8> {
    if path.extension() != Some(OsStr::new("html")) {
        return body.to_vec();
    }
    let sanitized = if path.file_name() == Some(OsStr::new("index.html")) {
        strip_trunk_autoreload_script(body)
    } else {
        body.to_vec()
    };
    if let Some(viewer_auth_bootstrap) = viewer_auth_bootstrap {
        inject_viewer_auth_bootstrap_script(sanitized.as_slice(), viewer_auth_bootstrap)
    } else {
        sanitized
    }
}

fn strip_trunk_autoreload_script(body: &[u8]) -> Vec<u8> {
    let html = String::from_utf8_lossy(body);
    let marker = ".well-known/trunk/ws";
    let Some(marker_index) = html.find(marker) else {
        return body.to_vec();
    };
    let Some(script_start) = html[..marker_index].rfind("<script") else {
        return body.to_vec();
    };
    let Some(script_end_rel) = html[marker_index..].find("</script>") else {
        return body.to_vec();
    };
    let script_end = marker_index + script_end_rel + "</script>".len();

    let mut sanitized = String::with_capacity(html.len());
    sanitized.push_str(&html[..script_start]);
    sanitized.push_str(&html[script_end..]);
    sanitized.into_bytes()
}

fn inject_viewer_auth_bootstrap_script(body: &[u8], auth: &ViewerAuthBootstrap) -> Vec<u8> {
    let html = String::from_utf8_lossy(body);
    let script = build_viewer_auth_bootstrap_script(auth);
    let insert_at = html
        .rfind("</head>")
        .or_else(|| html.rfind("</body>"))
        .unwrap_or(html.len());
    let mut injected = String::with_capacity(html.len() + script.len() + 1);
    injected.push_str(&html[..insert_at]);
    injected.push_str(script.as_str());
    injected.push_str(&html[insert_at..]);
    injected.into_bytes()
}

pub(super) fn build_viewer_auth_bootstrap_script(auth: &ViewerAuthBootstrap) -> String {
    let payload = serde_json::json!({
        VIEWER_PLAYER_ID_ENV: auth.player_id,
        VIEWER_AUTH_PUBLIC_KEY_ENV: auth.public_key,
        VIEWER_AUTH_PRIVATE_KEY_ENV: auth.private_key,
    });
    let payload = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    format!(
        "<script>const __oasis7ViewerAuthEnv=Object.freeze({payload});window.{VIEWER_AUTH_BOOTSTRAP_OBJECT}=__oasis7ViewerAuthEnv;</script>"
    )
}

pub(super) fn resolve_viewer_auth_bootstrap_from_path(
    path: &Path,
) -> Result<ViewerAuthBootstrap, String> {
    let content =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let value: toml::Value = toml::from_str(content.as_str())
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let node = value
        .get(NODE_TABLE_KEY)
        .and_then(toml::Value::as_table)
        .ok_or_else(|| format!("{NODE_TABLE_KEY} table is missing in {}", path.display()))?;
    let private_key =
        resolve_required_toml_string(node, NODE_PRIVATE_KEY_FIELD, "node.private_key")?;
    let public_key = resolve_required_toml_string(node, NODE_PUBLIC_KEY_FIELD, "node.public_key")?;
    let player_id = resolve_viewer_player_id_override(env::var(VIEWER_PLAYER_ID_ENV).ok());
    Ok(ViewerAuthBootstrap {
        player_id,
        public_key,
        private_key,
    })
}

pub(super) fn resolve_viewer_auth_bootstrap_for_embedded_server(
    deployment_mode: DeploymentMode,
) -> Option<ViewerAuthBootstrap> {
    if deployment_mode.disables_browser_signer_bootstrap() {
        None
    } else {
        resolve_viewer_auth_bootstrap_from_path(Path::new(NODE_CONFIG_FILE_NAME)).ok()
    }
}
