// NovaStack site. Every number, file and code sample on the page comes from
// data/generated/, which site/tools/build-data.py produces from the real `novastack`
// binary at deploy time.

const DATA = "data/generated/";
const REPO = "https://github.com/cybercoretech/NovaStack";
const $ = (sel, root = document) => root.querySelector(sel);
const reduceMotion = matchMedia("(prefers-reduced-motion: reduce)").matches;

const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

/* ---------- Plugin guides (behaviour, not counts) ----------
   Hand-written usage notes. Every Rust snippet and shell command here was compiled /
   run against a real generated project; files, diffs and env vars come from the data. */
const ENV_HELP = {
  JWT_SECRET: "Key used to sign tokens. Required: the app refuses to start without it. Use a long random value (openssl rand -hex 32).",
  JWT_EXPIRATION_HOURS: "How long a token stays valid, in hours (default 24).",
  REDIS_URL: "Where Redis runs. Under docker compose the app uses redis://redis:6379 automatically.",
};
const PLUGIN_GUIDES = {
  auth: {
    tagline: "User accounts: register, log in with JWTs, protect routes.",
    gets: [
      "<code>POST /auth/register</code> creates an account (password stored as an Argon2id hash).",
      "<code>POST /auth/login</code> checks the password and returns a signed JWT.",
      "<code>GET /auth/me</code>, an example route that requires a valid token.",
      "An <code>AuthUser</code> extractor: add it to any handler's arguments to require login.",
      "A <code>users</code> table and migration with <code>--db postgres</code> or <code>sqlite</code>; with <code>--db none</code>, accounts live in memory.",
    ],
    setup: { text: "Set a real signing secret in <code>.env</code>, then start the app:", cmd: `cp .env.example .env    # if you haven't yet
sed -i "s/^JWT_SECRET=.*/JWT_SECRET=$(openssl rand -hex 32)/" .env
cargo run` },
    tryIt: `# create an account (username 3-64 chars, password 8+ chars)
curl -s -X POST localhost:8080/auth/register \\
  -H 'content-type: application/json' \\
  -d '{"username":"ada","password":"correct horse"}'
# {"id":1,"username":"ada"}

# log in and keep the token
TOKEN=$(curl -s -X POST localhost:8080/auth/login \\
  -H 'content-type: application/json' \\
  -d '{"username":"ada","password":"correct horse"}' | sed -E 's/.*"token":"([^"]+)".*/\\1/')

curl localhost:8080/auth/me -H "Authorization: Bearer $TOKEN"
# {"id":1,"username":"ada"}

curl localhost:8080/auth/me
# {"error":{"code":401,"message":"Missing bearer token"}}`,
    code: { title: "protect a route", lang: "rust", src: `use crate::auth::AuthUser;
use axum::Json;
use serde_json::{json, Value};

/// Only reachable with a valid \`Authorization: Bearer <token>\` header.
/// \`user.user_id\` is the account id from the users table.
pub async fn my_orders(user: AuthUser) -> Json<Value> {
    Json(json!({ "orders_for": user.user_id }))
}

// in router.rs: .route("/my-orders", get(my_orders))` },
    limits: "Wrong passwords and unknown usernames get the same 401, with the same Argon2 cost. There is no rate limiting, email verification or password reset yet; add them before going public. Auth guards HTTP routes only (not gRPC or WebSockets).",
  },
  otel: {
    tagline: "Prometheus metrics for every route.",
    gets: [
      "<code>GET /metrics</code> in Prometheus text format, ready to scrape.",
      "<code>http_requests_total</code> and <code>http_request_duration_seconds</code> for every request, by method, route and status.",
      "The <code>metrics</code> crate set up, so your own counters and histograms show up too.",
    ],
    setup: { text: "Nothing to configure: the recorder starts with the app.", cmd: "cargo run" },
    tryIt: `curl localhost:8080/health > /dev/null
curl localhost:8080/metrics
# http_requests_total{method="GET",path="/health",status="200"} 1
# ...`,
    code: { title: "record your own metrics", lang: "rust", src: `// anywhere in your code:
metrics::counter!("orders_created_total").increment(1);
metrics::histogram!("order_value_usd").record(42.0);` },
    limits: "Metrics only for now: OpenTelemetry trace export (OTLP) is planned but not implemented.",
  },
  redis: {
    tagline: "A Redis connection pool and cache helpers.",
    gets: [
      "A <code>deadpool-redis</code> pool in <code>AppState</code> as <code>state.redis</code>.",
      "<code>cache::get</code> and <code>cache::set_ex</code> helpers.",
      "<code>REDIS_URL</code> in the config, and a <code>redis</code> service in <code>docker-compose.yml</code> for new projects.",
    ],
    setup: { text: "Start Redis. New projects created with <code>--plugins redis</code> already have the Compose service. After <code>novastack add redis</code>, add this under <code>services:</code> in <code>docker-compose.yml</code> first:", cmd: `  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"`, after: "docker compose up -d redis\ncargo run" },
    tryIt: `docker compose exec redis redis-cli ping
# PONG`,
    code: { title: "cache a value (cache-aside)", lang: "rust", src: `use crate::{cache, error::AppError, state::AppState};
use axum::extract::{Path, State};

pub async fn cached_greeting(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<String, AppError> {
    let key = format!("greeting:{name}");
    if let Some(hit) = cache::get(&state.redis, &key).await? {
        return Ok(hit);
    }
    let value = format!("Hello, {name}!");
    cache::set_ex(&state.redis, &key, &value, 60).await?;
    Ok(value)
}

// in router.rs: .route("/greeting/{name}", get(cached_greeting))` },
    limits: "The pool connects lazily, so the app starts even if Redis is down; the first cache call then returns an error.",
  },
  workers: {
    tagline: "Run work in the background.",
    gets: [
      "A worker supervisor started at boot (<code>workers::start_workers</code>).",
      "A <code>Task</code> trait: implement it for each kind of background job.",
    ],
    setup: { text: "Nothing to configure. Start the app and look for the supervisor in the logs:", cmd: `cargo run
# ... "message":"Background worker supervisor started." ...` },
    tryIt: null,
    code: { title: "define and run a task", lang: "rust", src: `use crate::workers::tasks::Task;
use async_trait::async_trait;

pub struct SendWelcomeEmail {
    pub to: String,
}

#[async_trait]
impl Task for SendWelcomeEmail {
    async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(to = %self.to, "sending welcome email");
        Ok(())
    }
}

// fire it off from a handler (or from start_workers):
tokio::spawn(async move {
    let task = SendWelcomeEmail { to: "ada@example.com".into() };
    if let Err(err) = task.execute().await {
        tracing::error!(error = %err, "task failed");
    }
});` },
    limits: "Tasks run in-process with tokio::spawn. For durable queues (retries across restarts), back them with Redis or Postgres; the Task trait is the extension point.",
  },
};
const NAV = [
  ["top", "start"], ["templates", "templates"], ["plugins", "plugins"], ["generate", "generate"],
  ["verified", "verified"], ["install", "install"], ["roadmap", "status"],
];

/* ---------- Tiny syntax highlighter ---------- */
const KEYWORDS = {
  rust: "as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while",
  toml: "true false",
  proto: "syntax package service rpc returns message string int32 int64 bool bytes repeated",
  sql: "SELECT FROM WHERE CREATE TABLE INSERT INTO VALUES PRIMARY KEY NOT NULL",
  shell: "cargo docker compose run set if then fi",
  docker: "FROM RUN COPY WORKDIR ENV USER EXPOSE HEALTHCHECK ENTRYPOINT CMD AS",
  yaml: "true false",
};
const COMMENT = { rust: "//", proto: "//", toml: "#", yaml: "#", shell: "#", docker: "#", env: "#", ignore: "#", sql: "--" };

function highlight(line, lang) {
  if (lang === "markdown" || lang === "text") return esc(line);
  if (lang === "html") {
    return esc(line)
      .replace(/(&lt;\/?)([\w-]+)/g, '$1<span class="t-k">$2</span>')
      .replace(/([\w:-]+)=(&quot;.*?&quot;)/g, '<span class="t-t">$1</span>=<span class="t-s">$2</span>')
      .replace(/(\{[{%].*?[%}]\})/g, '<span class="t-m">$1</span>');
  }
  const words = new Set((KEYWORDS[lang] || "").split(" "));
  const marker = COMMENT[lang];
  let out = "";
  let i = 0;
  while (i < line.length) {
    const rest = line.slice(i);
    if (marker && rest.startsWith(marker)) { out += `<span class="t-c">${esc(rest)}</span>`; break; }
    let m;
    if ((m = rest.match(/^"(?:[^"\\]|\\.)*"?/))) { out += `<span class="t-s">${esc(m[0])}</span>`; i += m[0].length; continue; }
    if (lang === "rust" && (m = rest.match(/^#!?\[[^\]]*\]?/))) { out += `<span class="t-a">${esc(m[0])}</span>`; i += m[0].length; continue; }
    if ((m = rest.match(/^\d[\d_.]*/))) { out += `<span class="t-n">${esc(m[0])}</span>`; i += m[0].length; continue; }
    if ((m = rest.match(/^[A-Za-z_][\w-]*!?/))) {
      const w = m[0];
      if (lang === "rust" && w.endsWith("!")) out += `<span class="t-m">${esc(w)}</span>`;
      else if (words.has(w)) out += `<span class="t-k">${esc(w)}</span>`;
      else if (lang === "rust" && /^[A-Z]/.test(w)) out += `<span class="t-t">${esc(w)}</span>`;
      else if ((lang === "toml" || lang === "env") && /^\s*$/.test(line.slice(0, i)) && /^\s*=/.test(line.slice(i + w.length))) out += `<span class="t-t">${esc(w)}</span>`;
      else out += esc(w);
      i += w.length;
      continue;
    }
    out += esc(line[i]);
    i += 1;
  }
  return out;
}

function codeLines(content, lang, numbered = true) {
  const lines = content.replace(/\n$/, "").split("\n");
  return lines.map((l, n) => `<span class="line${numbered ? "" : " bare"}">${numbered ? `<span class="ln">${n + 1}</span>` : ""}${highlight(l, lang)}</span>`).join("");
}

function diffLines(diff) {
  return diff.replace(/\n$/, "").split("\n").map((l) => {
    const cls = l.startsWith("+++") || l.startsWith("---") ? "d-file" : l.startsWith("@@") ? "d-hunk" : l.startsWith("+") ? "d-add" : l.startsWith("-") ? "d-del" : "";
    return `<span class="line ${cls}"><span class="ln"></span><span class="tx">${esc(l)}</span></span>`;
  }).join("");
}

let blockId = 0;
function cmdBlock(title, text, lang = "shell") {
  const id = `blk-${++blockId}`;
  return `<div class="code"><div class="code-bar"><span>${esc(title)}</span><button class="copy btn btn-ghost btn-sm" type="button" data-copy-from="${id}">copy</button></div><pre id="${id}"><code>${codeLines(text, lang, lang !== "shell")}</code></pre></div>`;
}

function fileBlock(title, label, body) {
  return `<div class="code"><div class="code-bar"><span>${esc(title)}</span>${label ? `<span class="label ${label}">${label}</span>` : ""}</div><pre><code>${body}</code></pre></div>`;
}

/* ---------- Drawer ---------- */
function setupDrawer() {
  const drawer = $("#drawer"), scrim = $(".scrim"), burger = $(".burger"), body = $("#drawer-body");
  body.innerHTML = NAV.map(([id, label], i) => `<li><a href="#${id}"><span class="n">${String(i).padStart(2, "0")}</span>${label}</a></li>`).join("")
    + `<li><a href="${REPO}" target="_blank" rel="noopener"><span class="n">↗</span>github<span class="sr"> (opens in a new tab)</span></a></li>`;
  const open = (on) => {
    burger.setAttribute("aria-expanded", String(on));
    if (on) { drawer.hidden = false; scrim.hidden = false; requestAnimationFrame(() => drawer.classList.add("open")); $("a", body).focus(); }
    else { drawer.classList.remove("open"); scrim.hidden = true; setTimeout(() => { drawer.hidden = true; }, 220); burger.focus(); }
  };
  burger.addEventListener("click", () => open(burger.getAttribute("aria-expanded") !== "true"));
  $(".drawer-close").addEventListener("click", () => open(false));
  scrim.addEventListener("click", () => open(false));
  body.addEventListener("click", (e) => { if (e.target.closest("a[href^='#']")) open(false); });
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && !drawer.hidden) open(false); });
}

/* ---------- Copy buttons ---------- */
function setupCopy() {
  document.addEventListener("click", async (e) => {
    const btn = e.target.closest("[data-copy-from]");
    if (!btn) return;
    // Copy the code only: drop line-number gutters before reading the text.
    const copy = document.getElementById(btn.dataset.copyFrom).cloneNode(true);
    copy.querySelectorAll(".ln").forEach((n) => n.remove());
    // Highlighted blocks render each line as its own block <span>; join them with newlines.
    const lines = copy.querySelectorAll(".line");
    const text = (lines.length ? [...lines].map((l) => l.textContent).join("\n") : copy.textContent).trim();
    try { await navigator.clipboard.writeText(text); btn.textContent = "copied"; }
    catch { btn.textContent = "select + copy"; }
    setTimeout(() => { btn.textContent = "copy"; }, 1600);
  });
}

/* ---------- Accessible tabs ---------- */
function tabs(container, items, onSelect) {
  container.innerHTML = items.map((it, i) => `<button role="tab" type="button" id="${container.id}-${i}" aria-selected="${i === 0}" tabindex="${i === 0 ? 0 : -1}">${it.label}</button>`).join("");
  const buttons = [...container.querySelectorAll("button")];
  const select = (i, focus) => {
    buttons.forEach((b, j) => { b.setAttribute("aria-selected", String(i === j)); b.tabIndex = i === j ? 0 : -1; });
    if (focus) buttons[i].focus();
    onSelect(items[i], i);
  };
  buttons.forEach((b, i) => b.addEventListener("click", () => select(i)));
  container.addEventListener("keydown", (e) => {
    const i = buttons.indexOf(document.activeElement);
    if (i < 0) return;
    const next = { ArrowRight: i + 1, ArrowLeft: i - 1, Home: 0, End: buttons.length - 1 }[e.key];
    if (next === undefined) return;
    e.preventDefault();
    select((next + buttons.length) % buttons.length, true);
  });
  select(0);
}

/* ---------- Hero terminal (replays a real dry run) ---------- */
function hero(data) {
  const term = $("#term");
  const files = data.hero.files;
  const shown = files.slice(0, 9);
  const lines = [
    `<span class="tl"><b class="p">❯</b> ${esc(data.hero.command)}</span>`,
    `<span class="tl"><b class="ok">✓</b> template <s>rest · ${data.catalog.plugins.length} plugins available</s></span>`,
    ...shown.map((f) => `<span class="tl">  <b class="ok">+</b> <span class="f">${esc(f)}</span></span>`),
    `<span class="tl">  <s>… ${files.length - shown.length} more files</s></span>`,
    `<span class="tl"><b class="ok">✓</b> ${files.length} files <s>· compiles · cargo run</s></span>`,
    `<span class="tl"><b class="p">❯</b> <span class="caret"></span></span>`,
  ];
  if (reduceMotion) { term.innerHTML = lines.join("\n"); return; }
  term.innerHTML = "";
  let i = 0;
  const step = () => {
    term.innerHTML = lines.slice(0, i + 1).join("\n");
    i += 1;
    if (i < lines.length) setTimeout(step, i === 1 ? 700 : 110);
  };
  step();
}

/* ---------- Template explorer ---------- */
const templateCache = new Map();

async function loadTemplate(name) {
  if (!templateCache.has(name)) {
    templateCache.set(name, fetch(`${DATA}templates/${name}.json`).then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); }));
  }
  return templateCache.get(name);
}

function buildTree(files) {
  const root = { dirs: new Map(), files: [] };
  for (const f of files) {
    const parts = f.path.split("/");
    let node = root;
    for (const dir of parts.slice(0, -1)) {
      if (!node.dirs.has(dir)) node.dirs.set(dir, { dirs: new Map(), files: [] });
      node = node.dirs.get(dir);
    }
    node.files.push({ ...f, name: parts.at(-1) });
  }
  return root;
}

function renderTree(node, depth = 0, prefix = "") {
  let html = "";
  for (const [name, child] of [...node.dirs].sort()) {
    html += `<div class="tree-item dir" role="treeitem" aria-expanded="true" style="padding-left:${12 + depth * 14}px"><span class="ic" aria-hidden="true">▾</span><span class="name">${esc(name)}/</span></div>`;
    html += `<div role="group">${renderTree(child, depth + 1, `${prefix}${name}/`)}</div>`;
  }
  for (const f of node.files.sort((a, b) => a.name.localeCompare(b.name))) {
    const test = f.path.startsWith("tests/") ? " test" : "";
    html += `<button type="button" class="tree-item file${test}" role="treeitem" aria-selected="false" data-path="${esc(f.path)}" style="padding-left:${12 + depth * 14}px"><span class="ic" aria-hidden="true">·</span><span class="name">${esc(f.name)}</span><span class="meta">${f.lines}</span></button>`;
  }
  return html;
}

function showFile(file) {
  $("#template-code").innerHTML = `<div class="code-bar"><span>${esc(file.path)}</span><span class="comment">${esc(file.lang)} · ${file.lines} lines</span></div><pre tabindex="0" aria-label="${esc(file.path)}"><code>${codeLines(file.content, file.lang)}</code></pre>`;
  document.querySelectorAll("#template-tree .file").forEach((b) => b.setAttribute("aria-selected", String(b.dataset.path === file.path)));
}

const START_FILE = { rest: "src/api/items.rs", htmx: "src/web/mod.rs", grpc: "src/grpc/mod.rs", realtime: "src/realtime/mod.rs", minimal: "src/main.rs" };

async function showTemplate(meta) {
  $("#template-summary").textContent = meta.summary;
  $("#template-cmd").textContent = meta.command;
  const tree = $("#template-tree"), code = $("#template-code");
  tree.innerHTML = `<p class="comment" style="padding:0 12px">loading…</p>`;
  code.innerHTML = "";
  try {
    const t = await loadTemplate(meta.name);
    tree.innerHTML = renderTree(buildTree(t.files));
    const byPath = new Map(t.files.map((f) => [f.path, f]));
    tree.onclick = (e) => { const b = e.target.closest(".file"); if (b) showFile(byPath.get(b.dataset.path)); };
    showFile(byPath.get(START_FILE[meta.name]) || t.files[0]);
  } catch {
    tree.innerHTML = `<p class="comment" style="padding:0 12px">couldn't load the file list</p>`;
  }
}

function templates(data) {
  tabs($("#template-tabs"), data.templates.map((t) => ({ ...t, label: `${esc(t.name)}${t.default ? '<span class="def">default</span>' : ""}` })), showTemplate);
}

/* ---------- Plugins ---------- */
function pluginSheet(p) {
  const g = PLUGIN_GUIDES[p.name] || {};
  const env = p.env_added.map((line) => {
    const name = line.split("=")[0];
    return `<li><code>${esc(name)}</code>${ENV_HELP[name] ? ` · ${esc(ENV_HELP[name])}` : ""}</li>`;
  }).join("");
  const step = (n, title, body) => `<section class="guide-step"><h3><span class="step-n">${n}</span>${title}</h3>${body}</section>`;
  let n = 0;
  return `
    <p class="guide-lede">${esc(g.tagline || p.description)}</p>
    ${g.gets ? step(++n, "What you get", `<ul class="gets">${g.gets.map((x) => `<li>${x}</li>`).join("")}</ul>`) : ""}
    ${step(++n, "Install it", `
      <p>In a <b>new project</b>:</p>${cmdBlock("terminal", `novastack new my-api --plugins ${p.name}`)}
      <p>In an <b>existing project</b> (run inside its directory):</p>${cmdBlock("terminal", `novastack add ${p.name}`)}
      <p class="hint">Both give the same code. <code>add</code> edits ${p.patched.map((f) => `<code>${esc(f.path)}</code>`).join(", ")} and creates ${p.created.length} file${p.created.length === 1 ? "" : "s"}; review the changes and run <code>cargo check</code>.</p>`)}
    ${g.setup ? step(++n, "Set it up", `
      <p>${g.setup.text}</p>${cmdBlock(g.setup.cmd.includes("image:") ? "docker-compose.yml" : "terminal", g.setup.cmd, g.setup.cmd.includes("image:") ? "yaml" : "shell")}
      ${g.setup.after ? cmdBlock("terminal", g.setup.after) : ""}
      ${env ? `<p>Environment variables it adds to <code>.env.example</code>:</p><ul class="envs">${env}</ul>` : ""}`) : ""}
    ${g.tryIt ? step(++n, "Try it", cmdBlock("terminal", g.tryIt)) : ""}
    ${g.code ? step(++n, "Use it in your code", cmdBlock(g.code.title, g.code.src, g.code.lang)) : ""}
    ${g.limits ? `<p class="note"><b>Limits.</b> ${esc(g.limits)}</p>` : ""}
    <details class="changes">
      <summary>What it changes in your project · ${p.created.length} new, ${p.patched.length} patched</summary>
      <div class="changes-body">
        <div><h3>Cargo.toml adds</h3>${fileBlock("Cargo.toml", "", codeLines(p.cargo_added.join("\n"), "toml"))}</div>
        <div><h3>files created</h3><div class="gen-files">${p.created.map((f) => fileBlock(f.path, "new", codeLines(f.content, f.lang))).join("")}</div></div>
        <div><h3>files patched</h3><div class="gen-files">${p.patched.map((f) => `<div class="diff">${fileBlock(f.path, "patched", diffLines(f.diff))}</div>`).join("")}</div></div>
      </div>
    </details>`;
}

function plugins(data) {
  const sheet = $("#sheet");
  $("#plugin-cards").innerHTML = data.plugins.map((p, i) => {
    const g = PLUGIN_GUIDES[p.name] || {};
    return `
    <button type="button" class="card" data-i="${i}" aria-haspopup="dialog">
      <span class="card-file">--plugins ${esc(p.name)}</span>
      <h3>${esc(p.name)}</h3>
      <p class="card-tag">${esc(g.tagline || "")}</p>
      <p>${esc(p.description)}</p>
      <code class="card-cmd">novastack add ${esc(p.name)}</code>
      <span class="chips">${p.dependencies.map((d) => `<span class="chip c-dep">${esc(d)}</span>`).join("")}</span>
      <span class="open">open the guide →</span>
    </button>`;
  }).join("");
  $("#plugin-cards").addEventListener("click", (e) => {
    const card = e.target.closest(".card");
    if (!card) return;
    const p = data.plugins[Number(card.dataset.i)];
    $("#sheet-title").textContent = `plugin · ${p.name}`;
    $("#sheet-body").innerHTML = pluginSheet(p);
    sheet.showModal();
    $("#sheet-body").scrollTop = 0;
  });
  $(".sheet-close").addEventListener("click", () => sheet.close());
  sheet.addEventListener("click", (e) => { if (e.target === sheet) sheet.close(); });
}

/* ---------- Generate ---------- */
function generate(data) {
  tabs($("#generate-tabs"), data.generate.map((g) => ({ ...g, label: esc(g.kind) })), (g) => {
    $("#generate-panel").innerHTML = `
      <p>${esc(g.description)}</p>
      <div class="cmd"><code id="gen-cmd">${esc(g.command)}</code><button class="btn btn-ghost btn-sm copy" type="button" data-copy-from="gen-cmd">copy</button></div>
      <div class="gen-files">
        ${g.created.map((f) => fileBlock(f.path, "new", codeLines(f.content, f.lang))).join("")}
        ${g.patched.map((f) => `<div class="diff">${fileBlock(f.path, "patched", diffLines(f.diff))}</div>`).join("")}
      </div>`;
  });
}

/* ---------- Verification ---------- */
const testLabel = (name) => name.replace(/^gen_/, "").replaceAll("_", " ");

function verification(data) {
  const v = data.verification;
  $("#matrix-list").innerHTML = v.matrix.map((t) => `<li><div>${esc(testLabel(t.name))}${t.doc ? `<br><span>${esc(t.doc)}</span>` : ""}</div></li>`).join("");
  $("#smoke-list").innerHTML = v.docker_smoke.map((s) => `<div class="smoke-group"><p>${esc(s.title)}</p><ul class="checks">${s.checks.map((c) => `<li>${esc(c)}</li>`).join("")}</ul></div>`).join("");
  $("#job-list").innerHTML = v.ci_jobs.map((j) => `<li>${esc(j)}</li>`).join("");
}

/* ---------- Bound values ---------- */
function bind(data) {
  const values = {
    version: `v${data.catalog.novastack_version}`,
    "template-count": data.templates.length,
    "plugin-count": data.plugins.length,
    "matrix-count": data.verification.matrix.length,
    rust: data.novastack_rust_version,
  };
  document.querySelectorAll("[data-bind]").forEach((el) => { if (el.dataset.bind in values) el.textContent = values[el.dataset.bind]; });
  if (data.built.commit) {
    const a = $("#built-commit");
    a.href = `${REPO}/commit/${data.built.commit}`;
    a.textContent = data.built.commit.slice(0, 7);
  }
  const names = data.templates.map((t) => t.name);
  const flag = $("#flag-templates");
  if (flag) flag.innerHTML = `the template: ${names.map((n) => `<code>${esc(n)}</code>`).join(", ")}`;
  $("#built-at").textContent = data.built.at ? `on ${data.built.at}` : "";
}

/* ---------- Boot ---------- */
setupDrawer();
setupCopy();
fetch(`${DATA}site-data.json`)
  .then((r) => { if (!r.ok) throw new Error(r.status); return r.json(); })
  .then((data) => {
    bind(data);
    hero(data);
    templates(data);
    plugins(data);
    generate(data);
    verification(data);
  })
  .catch((err) => {
    console.error("NovaStack site data failed to load", err);
    $("#template-summary").textContent = "Site data failed to load. The repository README has everything this page shows.";
  });
