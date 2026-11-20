#!/usr/bin/env bash
set -euo pipefail

PLUGIN_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$PLUGIN_ROOT/.." && pwd)"
INSTALL_BIN_DIR="${HOME}/.local/bin"
SKILLS_DEST="${HOME}/.gemini/antigravity/skills"

echo "[Lutetian Plugin Setup] Installing skills and MCP server..."

MCP_BIN=""
for cand in \
  "$PLUGIN_ROOT/lutetian-mcp" \
  "$INSTALL_BIN_DIR/lutetian-mcp" \
  "$REPO_ROOT/target/release/lutetian-mcp" \
  "$REPO_ROOT/target/debug/lutetian-mcp"; do
  if [[ -x "$cand" ]]; then
    MCP_BIN="$cand"
    break
  fi
done

if [[ -z "$MCP_BIN" && -f "$REPO_ROOT/Cargo.toml" ]]; then
  echo "Building lutetian-mcp in release mode..."
  (cd "$REPO_ROOT" && cargo build --release -p lutetian-mcp)
  MCP_BIN="$REPO_ROOT/target/release/lutetian-mcp"
fi

if [[ -z "$MCP_BIN" || ! -x "$MCP_BIN" ]]; then
  echo "Error: lutetian-mcp binary not found." >&2
  exit 1
fi

mkdir -p "$INSTALL_BIN_DIR" "$SKILLS_DEST"
cp -f "$MCP_BIN" "$INSTALL_BIN_DIR/lutetian-mcp"
chmod +x "$INSTALL_BIN_DIR/lutetian-mcp"

for dir in "$PLUGIN_ROOT/skills/"*; do
  if [[ -d "$dir" ]]; then
    name="$(basename "$dir")"
    mkdir -p "$SKILLS_DEST/$name"
    cp -rf "$dir/"* "$SKILLS_DEST/$name/"
    echo "  + Installed skill: $name"
  fi
done

cat > "$PLUGIN_ROOT/mcp_config.resolved.json" <<EOF
{
  "mcpServers": {
    "lutetian": {
      "command": "$INSTALL_BIN_DIR/lutetian-mcp",
      "args": []
    }
  }
}
EOF

echo ""
echo "Setup complete! Config written to $PLUGIN_ROOT/mcp_config.resolved.json"
