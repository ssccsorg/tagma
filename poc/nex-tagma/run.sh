#!/usr/bin/env bash
set -euo pipefail
# nex-tagma: standalone CI runner
# Runs all coverage: build, tests, bench, and optional nexus subtree sync.
# Usage: ./run.sh

cd "$(dirname "$0")"

# ── Subtree management ─────────────────────────────────────────────────────
# nex-tagma references the nexus repository as a git subtree for integration
# testing. If the subtree is not present, it is cloned automatically.

NEXUS_SUBTREE_DIR="nexus"
NEXUS_REMOTE="git@github.com:ssccsorg/ssccs-nexus.git"
NEXUS_BRANCH="main"

check_subtree() {
    if [ -d "$NEXUS_SUBTREE_DIR" ] && [ -f "$NEXUS_SUBTREE_DIR/Cargo.toml" ]; then
        echo "nexus subtree: found at $NEXUS_SUBTREE_DIR"
        return 0
    else
        echo "nexus subtree: not found — cloning..."
        if git remote get-url origin &>/dev/null; then
            # Running inside a git repo: add as subtree
            git subtree add --prefix="$NEXUS_SUBTREE_DIR" "$NEXUS_REMOTE" "$NEXUS_BRANCH" --squash 2>&1 || {
                # If subtree add fails (e.g. already exists or network issue), try shallow clone
                echo "nexus subtree: git subtree add failed, trying shallow clone..."
                git clone --depth 1 --branch "$NEXUS_BRANCH" "$NEXUS_REMOTE" "$NEXUS_SUBTREE_DIR" 2>&1 || {
                    echo "nexus subtree: clone FAILED — continuing without nexus"
                    return 1
                }
            }
        else
            # Not a git repo: shallow clone
            git clone --depth 1 --branch "$NEXUS_BRANCH" "$NEXUS_REMOTE" "$NEXUS_SUBTREE_DIR" 2>&1 || {
                echo "nexus subtree: clone FAILED — continuing without nexus"
                return 1
            }
        fi
        echo "nexus subtree: ready at $NEXUS_SUBTREE_DIR"
        return 0
    fi
}

# ── Main ───────────────────────────────────────────────────────────────────

echo "=== nex-tagma ==="
cargo build 2>&1 || { echo "nex-tagma: build FAILED"; exit 1; }

echo "--- tests ---"
cargo test --tests 2>&1 || { echo "nex-tagma: tests FAILED"; exit 1; }

echo "--- bench ---"
cargo run -- bench 2>&1 || { echo "nex-tagma: bench FAILED"; exit 1; }

echo "--- subtree ---"
check_subtree

echo "nex-tagma: passed"
