// NovaStack site. Every number, file and code sample on the page comes from
// data/generated/, which site/tools/build-data.py produces from the real `novastack`
// binary at deploy time.

const DATA = "data/generated/";
const REPO = "https://github.com/cybercoretech/NovaStack";
const $ = (sel, root = document) => root.querySelector(sel);
const reduceMotion = matchMedia("(prefers-reduced-motion: reduce)").matches;

const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));

/* ---------- Editorial notes (behaviour, not counts) ---------- */
const PLUGIN_NOTES = {
  auth: "The generated /auth/login is a stub: it issues a token for any non-empty credentials and is marked as such. Check credentials with auth::password::verify_password before shipping. JWT_SECRET is required at startup.",
  otel: "Serves Prometheus metrics at /metrics with per-route request counts and latency. OpenTelemetry trace export (OTLP) is not implemented yet.",
  redis: "The pool connects lazily, so the app starts even if Redis is briefly down. `novastack add redis` on an existing project doesn't edit docker-compose.yml; it prints a reminder.",
  workers: "Spawns the worker supervisor at startup. Queue backends (Redis, Postgres) are up to you; the Task trait is the extension point.",
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

function codeLines(content, lang) {
  const lines = content.replace(/\n$/, "").split("\n");
  return lines.map((l, n) => `<span class="line"><span class="ln">${n + 1}</span>${highlight(l, lang)}</span>`).join("");
}

function diffLines(diff) {
  return diff.replace(/\n$/, "").split("\n").map((l) => {
    const cls = l.startsWith("+++") || l.startsWith("---") ? "d-file" : l.startsWith("@@") ? "d-hunk" : l.startsWith("+") ? "d-add" : l.startsWith("-") ? "d-del" : "";
    return `<span class="line ${cls}"><span class="ln"></span><span class="tx">${esc(l)}</span></span>`;
  }).join("");
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
    const text = document.getElementById(btn.dataset.copyFrom).innerText.trim();
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
function plugins(data) {
  const sheet = $("#sheet");
  $("#plugin-cards").innerHTML = data.plugins.map((p, i) => `
    <button type="button" class="card" data-i="${i}" aria-haspopup="dialog">
      <span class="card-file">plugins/${esc(p.name)}.rs</span>
      <h3>${esc(p.name)}</h3>
      <p>${esc(p.description)}</p>
      <span class="chips">${p.dependencies.map((d) => `<span class="chip c-dep">${esc(d)}</span>`).join("")}<span class="chip">+${p.created.length} files</span><span class="chip">~${p.patched.length} patched</span></span>
      <span class="open">open details →</span>
    </button>`).join("");
  $("#plugin-cards").addEventListener("click", (e) => {
    const card = e.target.closest(".card");
    if (!card) return;
    const p = data.plugins[Number(card.dataset.i)];
    $("#sheet-title").textContent = `${p.command}`;
    $("#sheet-body").innerHTML = `
      <p>${esc(p.description)}</p>
      ${PLUGIN_NOTES[p.name] ? `<p class="note">${esc(PLUGIN_NOTES[p.name])}</p>` : ""}
      <div><h3>Cargo.toml adds</h3>${fileBlock("Cargo.toml", "", codeLines(p.cargo_added.join("\n"), "toml"))}</div>
      ${p.env_added.length ? `<div><h3>.env.example adds</h3>${fileBlock(".env.example", "", codeLines(p.env_added.join("\n"), "env"))}</div>` : ""}
      <div><h3>files created (${p.created.length})</h3><div class="gen-files">${p.created.map((f) => fileBlock(f.path, "new", codeLines(f.content, f.lang))).join("")}</div></div>
      <div><h3>files patched (${p.patched.length})</h3><div class="gen-files">${p.patched.map((f) => `<div class="diff">${fileBlock(f.path, "patched", diffLines(f.diff))}</div>`).join("")}</div></div>`;
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
