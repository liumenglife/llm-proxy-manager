use crate::modules::oauth;
use crate::modules::oauth_codex;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::Url;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::watch;

struct OAuthFlowState {
    auth_url: String,
    #[allow(dead_code)]
    redirect_uri: String,
    state: String,
    client_key: String,
    provider: Option<String>,
    pkce_verifier: Option<String>,
    cancel_tx: watch::Sender<bool>,
    code_tx: mpsc::Sender<Result<String, String>>,
    code_rx: Option<mpsc::Receiver<Result<String, String>>>,
}

#[derive(Clone, Serialize)]
struct OAuthUrlGeneratedPayload {
    url: String,
    provider: String,
}

static OAUTH_FLOW_STATE: OnceLock<Mutex<Option<OAuthFlowState>>> = OnceLock::new();

fn get_oauth_flow_state() -> &'static Mutex<Option<OAuthFlowState>> {
    OAUTH_FLOW_STATE.get_or_init(|| Mutex::new(None))
}

fn oauth_success_html() -> &'static str {
    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
    <html>\
    <body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
    <h1 style='color: green;'>✅ Authorization Successful!</h1>\
    <p>You can close this window and return to the application.</p>\
    <script>setTimeout(function() { window.close(); }, 2000);</script>\
    </body>\
    </html>"
}

fn oauth_fail_html() -> &'static str {
    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
    <html>\
    <body style='font-family: sans-serif; text-align: center; padding: 50px;'>\
    <h1 style='color: red;'>❌ Authorization Failed</h1>\
    <p>Failed to obtain Authorization Code. Please return to the app and try again.</p>\
    </body>\
    </html>"
}

fn existing_flow_matches_request(
    flow: &OAuthFlowState,
    requested_client_key: Option<&String>,
    provider: Option<&str>,
) -> bool {
    if flow.provider.as_deref() != provider {
        return false;
    }

    if let Some(requested_key) = requested_client_key {
        return flow.client_key == requested_key.to_ascii_lowercase();
    }

    true
}

fn oauth_url_generated_payload(
    auth_url: String,
    provider: Option<&str>,
) -> OAuthUrlGeneratedPayload {
    OAuthUrlGeneratedPayload {
        url: auth_url,
        provider: provider.unwrap_or("gemini").to_string(),
    }
}

async fn ensure_oauth_flow_prepared(
    app_handle: Option<tauri::AppHandle>,
    requested_client_key: Option<String>,
    provider: Option<&str>,
) -> Result<String, String> {
    if let Ok(mut state) = get_oauth_flow_state().lock() {
        if let Some(s) = state.as_mut() {
            if !existing_flow_matches_request(s, requested_client_key.as_ref(), provider) {
                let _ = s.cancel_tx.send(true);
                *state = None;
            }
        }
    }

    // Return URL if flow already exists and is still "fresh" (receiver hasn't been taken)
    if let Ok(mut state) = get_oauth_flow_state().lock() {
        if let Some(s) = state.as_mut() {
            if s.code_rx.is_some() {
                return Ok(s.auth_url.clone());
            } else {
                // Flow is already "in progress" (rx taken), but user requested a NEW one.
                // Force cancel the old one to allow a new attempt.
                let _ = s.cancel_tx.send(true);
                *state = None;
            }
        }
    }

    // Create loopback listeners.
    // Some browsers resolve `localhost` to IPv6 (::1). To avoid "localhost refused connection",
    // we try to listen on BOTH IPv6 and IPv4 with the same port when possible.
    let mut ipv4_listener: Option<TcpListener> = None;
    let mut ipv6_listener: Option<TcpListener> = None;

    // Prefer creating one listener on an ephemeral port first, then bind the other stack to same port.
    // If both are available -> use `http://localhost:<port>` as redirect URI.
    // If only one is available -> use an explicit IP to force correct stack.
    let port: u16;
    match TcpListener::bind("[::1]:0").await {
        Ok(l6) => {
            port = l6
                .local_addr()
                .map_err(|e| format!("failed_to_get_local_port: {}", e))?
                .port();
            ipv6_listener = Some(l6);

            match TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                Ok(l4) => ipv4_listener = Some(l4),
                Err(e) => {
                    crate::modules::logger::log_warn(&format!(
                        "failed_to_bind_ipv4_callback_port_127_0_0_1:{} (will only listen on IPv6): {}",
                        port, e
                    ));
                }
            }
        }
        Err(_) => {
            let l4 = TcpListener::bind("127.0.0.1:0")
                .await
                .map_err(|e| format!("failed_to_bind_local_port: {}", e))?;
            port = l4
                .local_addr()
                .map_err(|e| format!("failed_to_get_local_port: {}", e))?
                .port();
            ipv4_listener = Some(l4);

            match TcpListener::bind(format!("[::1]:{}", port)).await {
                Ok(l6) => ipv6_listener = Some(l6),
                Err(e) => {
                    crate::modules::logger::log_warn(&format!(
                        "failed_to_bind_ipv6_callback_port_::1:{} (will only listen on IPv4): {}",
                        port, e
                    ));
                }
            }
        }
    }

    let has_ipv4 = ipv4_listener.is_some();
    let has_ipv6 = ipv6_listener.is_some();

    let redirect_uri = if has_ipv4 && has_ipv6 {
        format!("http://localhost:{}/oauth-callback", port)
    } else if has_ipv4 {
        format!("http://127.0.0.1:{}/oauth-callback", port)
    } else {
        format!("http://[::1]:{}/oauth-callback", port)
    };

    let state_str = uuid::Uuid::new_v4().to_string();
    let (pkce_verifier, code_challenge) = if provider == Some("codex") {
        let (v, c) = oauth_codex::generate_pkce_pair();
        (Some(v), Some(c))
    } else {
        (None, None)
    };

    let (auth_url, resolved_client_key) = match provider {
        Some("codex") => {
            let url = oauth_codex::get_codex_auth_url(
                &redirect_uri,
                &state_str,
                code_challenge.as_deref().unwrap_or(""),
            )?;
            (url, "codex".to_string())
        }
        _ => oauth::get_auth_url_with_client(
            &redirect_uri,
            &state_str,
            requested_client_key.as_deref(),
        )?,
    };

    // Cancellation signal (supports multiple consumers)
    let (cancel_tx, cancel_rx) = watch::channel(false);
    // Use mpsc instead of oneshot to allow multiple senders (listener OR manual input)
    let (code_tx, code_rx) = mpsc::channel::<Result<String, String>>(1);

    // Start listeners immediately: even if the user authorizes before clicking "Start OAuth",
    // the browser can still hit our callback and finish the flow.
    let app_handle_for_tasks = app_handle.clone();

    if let Some(l4) = ipv4_listener {
        let tx = code_tx.clone();
        let mut rx = cancel_rx.clone();
        let app_handle = app_handle_for_tasks.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = tokio::select! {
                res = l4.accept() => res.map_err(|e| format!("failed_to_accept_connection: {}", e)),
                _ = rx.changed() => Err("OAuth cancelled".to_string()),
            } {
                // Reuse the existing parsing/response code by constructing a temporary listener task
                // that sends into the shared mpsc channel.
                let mut buffer = [0u8; 4096];
                let bytes_read = stream.read(&mut buffer).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buffer[..bytes_read]);

                // [FIX #931/850/778] More robust parsing and detailed logging
                let query_params = request
                    .lines()
                    .next()
                    .and_then(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            Some(parts[1])
                        } else {
                            None
                        }
                    })
                    .and_then(|path| {
                        // Use a dummy base for parsing; redirect_uri is already set to localhost
                        Url::parse(&format!("http://localhost{}", path)).ok()
                    })
                    .map(|url| {
                        let mut code = None;
                        let mut state = None;
                        for (k, v) in url.query_pairs() {
                            if k == "code" {
                                code = Some(v.to_string());
                            } else if k == "state" {
                                state = Some(v.to_string());
                            }
                        }
                        (code, state)
                    });

                let (code, received_state) = match query_params {
                    Some((c, s)) => (c, s),
                    None => (None, None),
                };

                if code.is_none() && bytes_read > 0 {
                    crate::modules::logger::log_error(&format!(
                        "OAuth callback failed to parse code. Raw request (first 512 bytes): {}",
                        &request.chars().take(512).collect::<String>()
                    ));
                }

                // Verify state
                let state_valid = {
                    if let Ok(lock) = get_oauth_flow_state().lock() {
                        if let Some(s) = lock.as_ref() {
                            received_state.as_ref() == Some(&s.state)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                let (result, response_html) = match (code, state_valid) {
                    (Some(code), true) => {
                        crate::modules::logger::log_info(
                            "Successfully captured OAuth code from IPv4 listener",
                        );
                        (Ok(code), oauth_success_html())
                    }
                    (Some(_), false) => {
                        crate::modules::logger::log_error(
                            "OAuth callback state mismatch (CSRF protection)",
                        );
                        (Err("OAuth state mismatch".to_string()), oauth_fail_html())
                    }
                    (None, _) => (
                        Err("Failed to get Authorization Code in callback".to_string()),
                        oauth_fail_html(),
                    ),
                };

                let _ = stream.write_all(response_html.as_bytes()).await;
                let _ = stream.flush().await;

                if let Some(h) = app_handle {
                    use tauri::Emitter;
                    let _ = h.emit("oauth-callback-received", ());
                }
                let _ = tx.send(result).await;
            }
        });
    }

    if let Some(l6) = ipv6_listener {
        let tx = code_tx.clone();
        let mut rx = cancel_rx;
        let app_handle = app_handle_for_tasks;
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = tokio::select! {
                res = l6.accept() => res.map_err(|e| format!("failed_to_accept_connection: {}", e)),
                _ = rx.changed() => Err("OAuth cancelled".to_string()),
            } {
                let mut buffer = [0u8; 4096];
                let bytes_read = stream.read(&mut buffer).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buffer[..bytes_read]);

                let query_params = request
                    .lines()
                    .next()
                    .and_then(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            Some(parts[1])
                        } else {
                            None
                        }
                    })
                    .and_then(|path| Url::parse(&format!("http://localhost{}", path)).ok())
                    .map(|url| {
                        let mut code = None;
                        let mut state = None;
                        for (k, v) in url.query_pairs() {
                            if k == "code" {
                                code = Some(v.to_string());
                            } else if k == "state" {
                                state = Some(v.to_string());
                            }
                        }
                        (code, state)
                    });

                let (code, received_state) = match query_params {
                    Some((c, s)) => (c, s),
                    None => (None, None),
                };

                if code.is_none() && bytes_read > 0 {
                    crate::modules::logger::log_error(&format!(
                        "OAuth callback failed to parse code (IPv6). Raw request: {}",
                        &request.chars().take(512).collect::<String>()
                    ));
                }

                // Verify state
                let state_valid = {
                    if let Ok(lock) = get_oauth_flow_state().lock() {
                        if let Some(s) = lock.as_ref() {
                            received_state.as_ref() == Some(&s.state)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                let (result, response_html) = match (code, state_valid) {
                    (Some(code), true) => {
                        crate::modules::logger::log_info(
                            "Successfully captured OAuth code from IPv6 listener",
                        );
                        (Ok(code), oauth_success_html())
                    }
                    (Some(_), false) => {
                        crate::modules::logger::log_error(
                            "OAuth callback state mismatch (IPv6 CSRF protection)",
                        );
                        (Err("OAuth state mismatch".to_string()), oauth_fail_html())
                    }
                    (None, _) => (
                        Err("Failed to get Authorization Code in callback".to_string()),
                        oauth_fail_html(),
                    ),
                };

                let _ = stream.write_all(response_html.as_bytes()).await;
                let _ = stream.flush().await;

                if let Some(h) = app_handle {
                    use tauri::Emitter;
                    let _ = h.emit("oauth-callback-received", ());
                }
                let _ = tx.send(result).await;
            }
        });
    }

    // Save state
    if let Ok(mut state) = get_oauth_flow_state().lock() {
        *state = Some(OAuthFlowState {
            auth_url: auth_url.clone(),
            redirect_uri,
            state: state_str,
            client_key: resolved_client_key,
            provider: provider.map(|s| s.to_string()),
            pkce_verifier,
            cancel_tx,
            code_tx,
            code_rx: Some(code_rx),
        });
    }

    // Send event to frontend (for display/copying link)
    if let Some(h) = app_handle {
        use tauri::Emitter;
        let payload = oauth_url_generated_payload(auth_url.clone(), provider);
        let _ = h.emit("oauth-url-generated", payload);
    }

    Ok(auth_url)
}

/// Pre-generate OAuth URL (does not open browser, does not block waiting for callback)
pub async fn prepare_oauth_url(
    app_handle: Option<tauri::AppHandle>,
    oauth_client_key: Option<String>,
    provider: Option<String>,
) -> Result<String, String> {
    ensure_oauth_flow_prepared(app_handle, oauth_client_key, provider.as_deref()).await
}

/// Cancel current OAuth flow
pub fn cancel_oauth_flow() {
    if let Ok(mut state) = get_oauth_flow_state().lock() {
        if let Some(s) = state.take() {
            let _ = s.cancel_tx.send(true);
            crate::modules::logger::log_info("Sent OAuth cancellation signal");
        }
    }
}

/// Start OAuth flow and wait for callback, then exchange token
pub async fn start_oauth_flow(
    app_handle: Option<tauri::AppHandle>,
    oauth_client_key: Option<String>,
    provider: Option<String>,
) -> Result<oauth::TokenResponse, String> {
    // Ensure URL + listener are ready (this way if the user authorizes first, it won't get stuck)
    let auth_url =
        ensure_oauth_flow_prepared(app_handle.clone(), oauth_client_key, provider.as_deref())
            .await?;

    if let Some(h) = app_handle {
        // Open default browser
        use tauri_plugin_opener::OpenerExt;
        h.opener()
            .open_url(&auth_url, None::<String>)
            .map_err(|e| format!("failed_to_open_browser: {}", e))?;
    }

    // Take code_rx to wait for it
    let (mut code_rx, redirect_uri, client_key, codex_verifier) = {
        let mut lock = get_oauth_flow_state()
            .lock()
            .map_err(|_| "OAuth state lock corrupted".to_string())?;
        let Some(state) = lock.as_mut() else {
            return Err("OAuth state does not exist".to_string());
        };
        let rx = state
            .code_rx
            .take()
            .ok_or_else(|| "OAuth authorization already in progress".to_string())?;
        (
            rx,
            state.redirect_uri.clone(),
            state.client_key.clone(),
            state.pkce_verifier.clone(),
        )
    };

    // Wait for code (if user has already authorized, this returns immediately)
    let code = match code_rx.recv().await {
        Some(Ok(code)) => code,
        Some(Err(e)) => return Err(e),
        None => return Err("OAuth flow channel closed unexpectedly".to_string()),
    };

    // Clean up flow state (release cancel_tx, etc.)
    if let Ok(mut lock) = get_oauth_flow_state().lock() {
        *lock = None;
    }

    if provider.as_deref() == Some("codex") {
        let verifier = codex_verifier.as_deref().unwrap_or("");
        let codex_res = oauth_codex::exchange_codex_code(&code, &redirect_uri, verifier).await?;
        Ok(oauth::TokenResponse {
            access_token: codex_res.access_token,
            expires_in: codex_res.expires_in,
            token_type: codex_res.token_type,
            refresh_token: codex_res.refresh_token,
            oauth_client_key: Some("codex".to_string()),
        })
    } else {
        oauth::exchange_code_with_client(&code, &redirect_uri, Some(&client_key)).await
    }
}

/// Завершить OAuth flow без открытия браузера.
/// Предполагается, что пользователь открыл ссылку вручную (или ранее была открыта),
/// а мы только ждём callback и обмениваем code на token.
pub async fn complete_oauth_flow(
    app_handle: Option<tauri::AppHandle>,
    provider: Option<String>,
) -> Result<oauth::TokenResponse, String> {
    // Ensure URL + listeners exist
    let _ = ensure_oauth_flow_prepared(app_handle, None, provider.as_deref()).await?;

    // Take receiver to wait for code
    let (mut code_rx, redirect_uri, client_key, provider, codex_verifier) = {
        let mut lock = get_oauth_flow_state()
            .lock()
            .map_err(|_| "OAuth state lock corrupted".to_string())?;
        let Some(state) = lock.as_mut() else {
            return Err("OAuth state does not exist".to_string());
        };
        let rx = state
            .code_rx
            .take()
            .ok_or_else(|| "OAuth authorization already in progress".to_string())?;
        (
            rx,
            state.redirect_uri.clone(),
            state.client_key.clone(),
            state.provider.clone(),
            state.pkce_verifier.clone(),
        )
    };

    let code = match code_rx.recv().await {
        Some(Ok(code)) => code,
        Some(Err(e)) => return Err(e),
        None => return Err("OAuth flow channel closed unexpectedly".to_string()),
    };

    if let Ok(mut lock) = get_oauth_flow_state().lock() {
        *lock = None;
    }

    if provider.as_deref() == Some("codex") {
        let verifier = codex_verifier.as_deref().unwrap_or("");
        let codex_res = oauth_codex::exchange_codex_code(&code, &redirect_uri, verifier).await?;
        Ok(oauth::TokenResponse {
            access_token: codex_res.access_token,
            expires_in: codex_res.expires_in,
            token_type: codex_res.token_type,
            refresh_token: codex_res.refresh_token,
            oauth_client_key: Some("codex".to_string()),
        })
    } else {
        oauth::exchange_code_with_client(&code, &redirect_uri, Some(&client_key)).await
    }
}

/// Manually submit an OAuth code to complete the flow.
/// This is used when the user manually copies the code/URL from the browser
/// because the localhost callback couldn't be reached (e.g. in Docker/remote).
pub async fn submit_oauth_code(
    code_input: String,
    state_input: Option<String>,
) -> Result<(), String> {
    let tx = {
        let lock = get_oauth_flow_state().lock().map_err(|e| e.to_string())?;
        if let Some(state) = lock.as_ref() {
            // Verify state if provided
            if let Some(provided_state) = state_input {
                if provided_state != state.state {
                    return Err("OAuth state mismatch (CSRF protection)".to_string());
                }
            }
            state.code_tx.clone()
        } else {
            return Err("No active OAuth flow found".to_string());
        }
    };

    // Extract code if it's a URL
    let code = if code_input.starts_with("http") {
        if let Ok(url) = Url::parse(&code_input) {
            url.query_pairs()
                .find(|(k, _)| k == "code")
                .map(|(_, v)| v.to_string())
                .unwrap_or(code_input)
        } else {
            code_input
        }
    } else {
        code_input
    };

    crate::modules::logger::log_info("Received manual OAuth code submission");

    // Send to the channel
    tx.send(Ok(code))
        .await
        .map_err(|_| "Failed to send code to OAuth flow (receiver dropped)".to_string())?;

    Ok(())
}
/// Manually prepare an OAuth flow without starting listeners.
/// Useful for Web/Docker environments where we only need manual code submission.
pub fn prepare_oauth_flow_manually(
    redirect_uri: String,
    state_str: String,
    oauth_client_key: Option<String>,
    provider: Option<&str>,
) -> Result<(String, mpsc::Receiver<Result<String, String>>), String> {
    let (pkce_verifier, code_challenge) = if provider == Some("codex") {
        let (v, c) = oauth_codex::generate_pkce_pair();
        (Some(v), Some(c))
    } else {
        (None, None)
    };

    let (auth_url, resolved_client_key) = match provider {
        Some("codex") => {
            let url = oauth_codex::get_codex_auth_url(
                &redirect_uri,
                &state_str,
                code_challenge.as_deref().unwrap_or(""),
            )?;
            (url, "codex".to_string())
        }
        _ => {
            oauth::get_auth_url_with_client(&redirect_uri, &state_str, oauth_client_key.as_deref())?
        }
    };

    // Check if we can reuse existing state
    if let Ok(mut lock) = get_oauth_flow_state().lock() {
        if let Some(s) = lock.as_mut() {
            let _ = s.cancel_tx.send(true);
            *lock = None;
        }
    }

    let (cancel_tx, _cancel_rx) = watch::channel(false);
    let (code_tx, code_rx) = mpsc::channel(1);

    if let Ok(mut state) = get_oauth_flow_state().lock() {
        *state = Some(OAuthFlowState {
            auth_url: auth_url.clone(),
            redirect_uri: redirect_uri.clone(),
            state: state_str,
            client_key: resolved_client_key,
            provider: provider.map(|s| s.to_string()),
            pkce_verifier,
            cancel_tx,
            code_tx,
            code_rx: None,
        });
    }

    Ok((auth_url, code_rx))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_flow(provider: Option<&str>, client_key: &str) -> OAuthFlowState {
        let (cancel_tx, _) = watch::channel(false);
        let (code_tx, code_rx) = mpsc::channel(1);
        OAuthFlowState {
            auth_url: "https://example.test/auth".to_string(),
            redirect_uri: "http://localhost/oauth-callback".to_string(),
            state: "state".to_string(),
            client_key: client_key.to_string(),
            provider: provider.map(|value| value.to_string()),
            pkce_verifier: None,
            cancel_tx,
            code_tx,
            code_rx: Some(code_rx),
        }
    }

    #[test]
    fn existing_gemini_flow_does_not_match_codex_request() {
        let flow = fake_flow(None, "gemini-client");

        assert!(!existing_flow_matches_request(&flow, None, Some("codex")));
    }

    #[test]
    fn existing_codex_flow_does_not_match_gemini_request() {
        let requested_key = "gemini-client".to_string();
        let flow = fake_flow(Some("codex"), "codex");

        assert!(!existing_flow_matches_request(
            &flow,
            Some(&requested_key),
            None
        ));
    }

    #[test]
    fn existing_flow_matches_same_provider_and_client_key() {
        let requested_key = "gemini-client".to_string();
        let flow = fake_flow(None, "gemini-client");

        assert!(existing_flow_matches_request(
            &flow,
            Some(&requested_key),
            None
        ));
    }

    #[test]
    fn oauth_url_generated_payload_defaults_to_gemini_provider() {
        let payload = oauth_url_generated_payload("https://example.test/gemini".to_string(), None);

        assert_eq!(payload.url, "https://example.test/gemini");
        assert_eq!(payload.provider, "gemini");
    }

    #[test]
    fn oauth_url_generated_payload_preserves_codex_provider() {
        let payload =
            oauth_url_generated_payload("https://example.test/codex".to_string(), Some("codex"));

        assert_eq!(payload.url, "https://example.test/codex");
        assert_eq!(payload.provider, "codex");
    }

    struct OAuthEnvGuard;
    impl Drop for OAuthEnvGuard {
        fn drop(&mut self) {
            std::env::remove_var("ANTIGRAVITY_OAUTH_CLIENTS");
            std::env::remove_var("ANTIGRAVITY_GOOGLE_OAUTH_CLIENT_ID");
            std::env::remove_var("ANTIGRAVITY_GOOGLE_OAUTH_CLIENT_SECRET");
        }
    }

    #[tokio::test]
    async fn ensure_oauth_flow_prepared_replaces_existing_provider_flow() {
        std::env::remove_var("ANTIGRAVITY_OAUTH_CLIENTS");
        std::env::remove_var("ANTIGRAVITY_GOOGLE_OAUTH_CLIENT_ID");
        std::env::remove_var("ANTIGRAVITY_GOOGLE_OAUTH_CLIENT_SECRET");
        std::env::set_var(
            "ANTIGRAVITY_OAUTH_CLIENTS",
            "test|client-id|client-secret|Test",
        );
        let _env_guard = OAuthEnvGuard;
        crate::modules::oauth::reload_oauth_registry_for_tests();
        cancel_oauth_flow();

        let gemini_url = ensure_oauth_flow_prepared(None, None, None)
            .await
            .expect("Gemini flow should be prepared");
        assert!(gemini_url.contains("state="));

        let codex_url = ensure_oauth_flow_prepared(None, None, Some("codex"))
            .await
            .expect("Codex flow should replace Gemini flow");
        assert_ne!(gemini_url, codex_url);

        let lock = get_oauth_flow_state()
            .lock()
            .expect("OAuth flow state should lock");
        let state = lock.as_ref().expect("OAuth flow state should exist");
        assert_eq!(state.provider.as_deref(), Some("codex"));
        assert!(state.code_rx.is_some());
        drop(lock);

        cancel_oauth_flow();
    }
}
