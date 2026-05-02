import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const dialog = readFileSync('src/components/accounts/AddAccountDialog.tsx', 'utf8');
const commands = readFileSync('src-tauri/src/commands/mod.rs', 'utf8');
const oauthServer = readFileSync('src-tauri/src/modules/oauth_server.rs', 'utf8');

assert(
  !dialog.includes("setOauthUrl(event.payload as string);"),
  'oauth-url-generated 事件不能无条件写入 Gemini URL 状态'
);

assert(
  !oauthServer.includes('h.emit("oauth-url-generated", &auth_url)'),
  'oauth-url-generated 事件必须携带 provider，不能只发送 URL 字符串'
);

assert(
  !dialog.includes('const url = event.payload as string;'),
  '前端不能把 oauth-url-generated payload 当作无来源 URL 字符串处理'
);

assert(
  !dialog.includes("if (oauthProviderRef.current === 'codex')") ||
    !dialog.slice(dialog.indexOf("listen('oauth-url-generated'"), dialog.indexOf('// Listen for OAuth callback completion')).includes('oauthProviderRef.current'),
  'oauth-url-generated 监听器必须按事件 provider 写入状态，不能依赖当前选中 provider'
);

const oauthUrlListener = dialog.slice(
  dialog.indexOf("listen('oauth-url-generated'"),
  dialog.indexOf('// Listen for OAuth callback completion')
);

assert(
  dialog.includes("url: string;") && dialog.includes("provider: 'gemini' | 'codex';"),
  'oauth-url-generated 前端 payload 类型必须强制携带 url 和受限 provider，不能允许缺失来源'
);

assert(
  oauthUrlListener.includes("provider === 'codex'") && oauthUrlListener.includes("provider === 'gemini'"),
  'oauth-url-generated 监听器必须显式识别 codex 与 gemini provider'
);
assert(
  !oauthUrlListener.includes('} else {'),
  'oauth-url-generated 监听器不能用默认 else 写入 URL，未知 provider 必须被忽略'
);

const codexPrepareIndex = dialog.indexOf("prepare_codex_oauth_url");
assert(codexPrepareIndex >= 0, 'Codex OAuth URL 仍需在用户选择 Codex 时准备');
const codexGuard = dialog.lastIndexOf("oauthProviderRef.current !== 'codex'", codexPrepareIndex);
assert(
  codexGuard >= 0,
  'Codex OAuth URL 准备必须受当前 provider 约束，不能在打开 OAuth tab 时无条件执行'
);

const geminiPrepareIndex = dialog.indexOf("prepare_oauth_url");
assert(geminiPrepareIndex >= 0, 'Gemini OAuth URL 仍需在用户选择 Gemini 时准备');
const geminiGuard = dialog.lastIndexOf("oauthProviderRef.current !== 'gemini'", geminiPrepareIndex);
assert(
  geminiGuard >= 0,
  'Gemini OAuth URL 准备必须受当前 provider 约束，不能和 Codex 同时生成真实 flow'
);

assert(
  !commands.includes('let _ = crate::commands::proxy::reload_proxy_accounts'),
  'reload_proxy_accounts 的错误不能用 let _ = 静默丢弃'
);
