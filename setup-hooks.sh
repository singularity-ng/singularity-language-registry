#!/usr/bin/env bash
set -euo pipefail

# Install repository git hooks to use the Nix devShell for heavy checks.
# This will create .githooks/pre-push and configure git to use .githooks as
# the hooks directory (via core.hooksPath). Running the pre-push hook will
# prefer `nix develop --command just pre-push` so native deps like OpenSSL
# are available when clippy and the build run.

HOOK_DIR=".githooks"
mkdir -p "$HOOK_DIR"

cat > "$HOOK_DIR/pre-push" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

# Pre-push hook that runs the project's pre-push task. If Nix is available
# we'll run the command inside the devShell so native libraries are provided.
if command -v nix >/dev/null 2>&1; then
  echo "🏗️  Running pre-push checks inside Nix devShell (nix develop)..."
  nix develop --command just pre-push
else
  echo "⚠️  Nix not found; running pre-push via just (may fail if native deps missing)..."
  just pre-push
fi
EOF

chmod +x "$HOOK_DIR/pre-push"

# Configure git to use the .githooks directory (safe to re-run)
git config core.hooksPath "$HOOK_DIR" || true

echo "✅ Installed git hooks to $HOOK_DIR and set core.hooksPath."
echo "Tip: run 'nix develop' or 'direnv allow' to enter the devShell before heavy builds."
