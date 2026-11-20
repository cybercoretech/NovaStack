#!/usr/bin/env python3
"""Runs the reset unit's SQL against a throwaway Postgres container, the way systemd would.

    demo/tools/check-reset.py [docker|podman]

systemd unescapes `$$` to `$` in ExecStart and splits words on quotes (no shell expansion),
so this does the same, then swaps `podman exec demo-postgres` for the chosen engine and a
temporary container. It checks the reset twice: on a fresh database (tables not created yet)
and after inserting rows (they must be gone).
"""
import shlex
import subprocess
import sys
import time
from pathlib import Path

ENGINE = sys.argv[1] if len(sys.argv) > 1 else "docker"
UNIT = Path(__file__).resolve().parent.parent / "systemd" / "novastack-demo-reset.service"
NAME = f"reset-check-{int(time.time())}"
PREFIX = ["/usr/bin/podman", "exec", "demo-postgres"]


def run(*args, check=True):
    return subprocess.run(args, check=check, capture_output=True, text=True)


def psql(db, sql):
    return run(ENGINE, "exec", NAME, "psql", "-U", "novastack", "-d", db, "-tAc", sql).stdout.strip()


def reset_commands():
    commands = []
    for line in UNIT.read_text().splitlines():
        if line.startswith("ExecStart="):
            argv = shlex.split(line[len("ExecStart="):].replace("$$", "$"))
            assert argv[:3] == PREFIX, f"unexpected ExecStart: {argv[:3]}"
            commands.append([ENGINE, "exec", NAME, *argv[3:]])
    assert len(commands) == 2, f"expected 2 ExecStart lines, found {len(commands)}"
    return commands


def reset():
    for cmd in reset_commands():
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            sys.exit(f"reset failed: {result.stderr.strip()}")


def main():
    init = Path(__file__).resolve().parent.parent / "postgres" / "init.sql"
    run(ENGINE, "run", "-d", "--name", NAME,
        "-e", "POSTGRES_USER=novastack", "-e", "POSTGRES_PASSWORD=check", "-e", "POSTGRES_DB=rest",
        "-v", f"{init}:/docker-entrypoint-initdb.d/init.sql:ro,Z",
        "docker.io/library/postgres:17.10-alpine")
    try:
        for _ in range(60):
            if run(ENGINE, "exec", NAME, "psql", "-U", "novastack", "-d", "htmx", "-tAc", "select 1",
                   check=False).returncode == 0:
                break
            time.sleep(1)
        else:
            sys.exit("postgres did not come up")

        reset()
        print("ok   reset on a fresh database (no tables yet)")

        psql("rest", "create table items(id bigserial primary key, name text);"
                     "create table users(id bigserial primary key, username text);"
                     "insert into items(name) values ('a'),('b'); insert into users(username) values ('u');")
        psql("htmx", "create table todos(id bigserial primary key, title text);"
                     "create table users(id bigserial primary key, username text);"
                     "insert into todos(title) values ('t'); insert into users(username) values ('u');")
        reset()
        counts = [psql("rest", "select count(*) from items"), psql("rest", "select count(*) from users"),
                  psql("htmx", "select count(*) from todos"), psql("htmx", "select count(*) from users")]
        assert counts == ["0"] * 4, f"rows left after reset: {counts}"
        psql("rest", "insert into items(name) values ('c')")
        assert psql("rest", "select max(id) from items") == "1", "ids did not restart at 1"
        print("ok   reset empties items, todos and users and restarts ids")
    finally:
        run(ENGINE, "rm", "-f", NAME, check=False)


if __name__ == "__main__":
    main()
