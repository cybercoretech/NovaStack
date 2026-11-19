#!/usr/bin/env python3
"""Builds the website's data from the real `novastack` binary.

    site/tools/build-data.py <path-to-novastack> [<out-dir>]

Everything the site shows about templates, plugins, generators and verification comes
from here: the binary's own catalogue (`novastack list --json`), real scaffolds of every
template, before/after diffs for every plugin and generator, the compile-matrix tests in
tests/generated_compile.rs and the checks in scripts/docker-smoke.sh. Nothing is typed
into the HTML by hand, so the site cannot drift from the code.
"""
import difflib
import json
import os
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ANSI = re.compile(r"\x1b\[[0-9;]*m")
LANGS = {
    ".rs": "rust", ".toml": "toml", ".sql": "sql", ".html": "html", ".proto": "proto",
    ".yml": "yaml", ".yaml": "yaml", ".md": "markdown", ".sh": "shell",
}
SPECIAL = {"Dockerfile": "docker", "Justfile": "shell", ".env.example": "env",
           ".gitignore": "ignore", ".dockerignore": "ignore", "novastack.toml": "toml"}


def run(binary, *args, cwd=None):
    out = subprocess.run([binary, *args], cwd=cwd, capture_output=True, text=True, check=True)
    return ANSI.sub("", out.stdout)


def scaffold(binary, workdir, name, *args):
    run(binary, "new", name, "--no-interactive", "--skip-hooks", "--out", str(workdir), *args)
    return workdir / name


def read_tree(project):
    files = {}
    for path in sorted(project.rglob("*")):
        if path.is_file():
            rel = path.relative_to(project).as_posix()
            files[rel] = path.read_text(encoding="utf-8")
    return files


def lang(path):
    name = path.rsplit("/", 1)[-1]
    return SPECIAL.get(name) or LANGS.get(Path(name).suffix, "text")


def stable(text):
    # novastack.toml carries a timestamp/version; keep previews deterministic.
    return text


def diff(before, after, path):
    return "".join(difflib.unified_diff(
        before.splitlines(keepends=True), after.splitlines(keepends=True),
        fromfile=f"a/{path}", tofile=f"b/{path}", n=2))


def added_lines(before, after):
    old = set(before.splitlines())
    return [line for line in after.splitlines() if line.strip() and line not in old
            and not line.lstrip().startswith("#")]


def changes(before, after):
    """Files created and patched between two project trees."""
    created = [{"path": p, "lang": lang(p), "content": after[p]}
               for p in sorted(after) if p not in before and p != "novastack.toml"]
    patched = [{"path": p, "diff": diff(before[p], after[p], p)}
               for p in sorted(after) if p in before and before[p] != after[p]
               and p not in ("novastack.toml", "README.md")]
    return created, patched


def matrix_tests():
    source = (ROOT / "tests/generated_compile.rs").read_text()
    tests = []
    for match in re.finditer(r"((?:\s*///[^\n]*\n)*)\s*#\[test\]\s*#\[ignore[^\]]*\]\s*fn (\w+)", source):
        doc = " ".join(l.strip()[3:].strip() for l in match.group(1).splitlines() if l.strip())
        tests.append({"name": match.group(2), "doc": doc})
    return tests


def ci_jobs():
    """CI job names, with matrix expressions expanded into the real job instances."""
    text = (ROOT / ".github/workflows/ci.yml").read_text()
    body = text.split("\njobs:\n", 1)[1]
    jobs = []
    for block in re.split(r"\n(?=  [\w-]+:\n)", body):
        m = re.search(r"^    name: (.+)$", block, re.M)
        if not m:
            continue
        name = m.group(1).strip()
        keys = re.findall(r"\$\{\{ matrix\.(\w+) \}\}", name)
        if not keys:
            jobs.append(name)
            continue
        include = re.findall(r"- shard: (.+)\n\s+toolchain: \"?([^\"\n]+)\"?", block)
        if include:  # include-style matrix (shard + toolchain)
            for shard, toolchain in include:
                jobs.append(name.replace("${{ matrix.shard }}", shard).replace("${{ matrix.toolchain }}", toolchain))
            continue
        lists = {k: re.search(rf"^\s+{k}: \[(.+)\]$", block, re.M) for k in keys}
        values = [v.strip() for v in lists[keys[0]].group(1).split(",")] if lists[keys[0]] else []
        for v in values:
            jobs.append(name.replace(f"${{{{ matrix.{keys[0]} }}}}", v))
    return jobs


def smoke_checks():
    """Docker smoke scenarios (from the script's header) and the checks each one runs."""
    text = (ROOT / "scripts/docker-smoke.sh").read_text()
    titles = dict(re.findall(r"^#   (\w+)\s{2,}(.+)$", text, re.M))
    after_restart = dict(re.findall(r"^after_restart_(\w+)\(\) \{\n(.*?)^\}", text, re.M | re.S))
    sections = []
    for name, fn in re.findall(r"^checks_(\w+)\(\) \{\n(.*?)^\}", text, re.M | re.S):
        checks = re.findall(r'expect "([^"]+)"', fn)
        # Checks run after `docker compose restart app` (data must persist)
        checks += re.findall(r'expect "([^"]+)"', after_restart.get(name, ""))
        checks += ["container healthcheck"]
        sections.append({"title": titles.get(name, name), "checks": checks})
    return sections


def rust_version():
    m = re.search(r'^rust-version = "([^"]+)"', (ROOT / "Cargo.toml").read_text(), re.M)
    return m.group(1) if m else ""


def commit():
    sha = os.environ.get("GITHUB_SHA")
    if sha:
        return sha
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True,
                              text=True, check=True).stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        return ""


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    binary = str(Path(sys.argv[1]).resolve())
    out = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "site/data/generated"
    (out / "templates").mkdir(parents=True, exist_ok=True)

    catalog = json.loads(run(binary, "list", "--json"))
    data = {
        "catalog": catalog,
        "built": {"commit": commit(), "at": datetime.now(timezone.utc).strftime("%Y-%m-%d")},
        "novastack_rust_version": rust_version(),
        "templates": [],
        "plugins": [],
        "generate": [],
        "verification": {
            "matrix": matrix_tests(),
            "ci_jobs": ci_jobs(),
            "docker_smoke": smoke_checks(),
        },
    }

    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)

        # Hero: a real session, trimmed to the interesting lines
        hero = run(binary, "new", "my-api", "-t", "rest", "--db", "postgres",
                   "--plugins", "auth,otel", "--no-interactive", "--skip-hooks", "--dry-run")
        data["hero"] = {
            "command": "novastack new my-api -t rest --db postgres --plugins auth,otel",
            "files": [l.strip()[2:] for l in hero.splitlines() if l.strip().startswith("+ ")],
        }

        # Templates: complete generated trees (no database, no plugins)
        for t in catalog["templates"]:
            name = t["name"]
            project = scaffold(binary, tmp / "templates", f"my-{name}", "-t", name, "--db", "none")
            files = read_tree(project)
            files.pop("novastack.toml", None)
            entry = {"name": name, "summary": t["summary"],
                     "command": f"novastack new my-{name} -t {name}",
                     "files": [{"path": p, "lang": lang(p), "lines": c.count("\n"),
                                "content": c} for p, c in files.items()]}
            (out / "templates" / f"{name}.json").write_text(json.dumps(entry))
            data["templates"].append({"name": name, "summary": t["summary"],
                                      "default": t["default"], "command": entry["command"],
                                      "file_count": len(files),
                                      "test_files": [p for p in files if p.startswith("tests/")]})

        # Plugins: what installing each one does to a fresh project
        base = read_tree(scaffold(binary, tmp / "plugins", "base", "--db", "none"))
        for p in catalog["plugins"]:
            after = read_tree(scaffold(binary, tmp / "plugins", f"with-{p['name']}",
                                       "--db", "none", "--plugins", p["name"]))
            # Project names differ (base vs with-x); compare with the name normalised
            after = {k: v.replace(f"with-{p['name']}", "base").replace(
                f"with_{p['name']}", "base") for k, v in after.items()}
            created, patched = changes(base, after)
            data["plugins"].append({
                **p,
                "command": f"novastack add {p['name']}",
                "created": created,
                "patched": patched,
                "cargo_added": added_lines(base["Cargo.toml"], after["Cargo.toml"]),
                "env_added": added_lines(base.get(".env.example", ""), after.get(".env.example", "")),
            })

        # Generators: one component of each kind in a fresh project
        for g in catalog["generate"]:
            project = scaffold(binary, tmp / "generate", f"gen-{g['kind']}", "--db", "none")
            before = read_tree(project)
            run(binary, "generate", g["kind"], "orders", cwd=project)
            created, patched = changes(before, read_tree(project))
            data["generate"].append({**g, "command": f"novastack generate {g['kind']} orders",
                                     "created": created, "patched": patched})

    (out / "site-data.json").write_text(json.dumps(data, indent=1))
    print(f"wrote {out}/site-data.json and {len(data['templates'])} template files")


if __name__ == "__main__":
    main()
