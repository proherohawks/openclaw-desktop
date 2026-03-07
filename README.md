# OpenClaw Desktop

Local Tauri v2 desktop app to chat with your running OpenClaw agents via WebSocket.

## Prerequisites
- Rust stable + `cargo` — https://rustup.rs
- Node.js 18+ — https://nodejs.org
- Tauri v2 system deps — https://tauri.app/start/prerequisites/
- tmux (recommended for agent team split panes) — `sudo apt install tmux`

## Quickstart
```bash
npm install
npm run tauri dev
```

## Build Release Binary
```bash
npm run tauri build
# Binary in: src-tauri/target/release/openclaw-desktop
```

## Using with OpenClaw
1. Start your OpenClaw instance (default gateway port 18789)
2. Launch openclaw-desktop
3. Enter your OpenClaw WebSocket endpoint (e.g. `ws://127.0.0.1:18789`)
4. Enter your auth token
5. Click Connect — your agents will appear in the sidebar

### Configuring the endpoint
The default endpoint is `ws://127.0.0.1:18789`. To connect to OpenClaw on a different port or host, enter the full WebSocket URL in the connection screen (e.g. `ws://192.168.1.100:18789`). The endpoint and auth token are persisted in localStorage across sessions.

## Building with Claude Code Agent Teams

Enable agent teams in `.claude/settings.json` (already done).

### Start the build team
```bash
# In your project directory
claude

# Then paste the spawn prompt from CLAUDE.md
# Or use the slash command:
/build
```

### Other slash commands
```
/review [files]     — 3-perspective code review
/debug [problem]    — adversarial hypothesis debugging
```

### tmux split panes
```bash
# Start tmux first for per-agent panels
tmux new-session -s openclaw
claude
# Teammates will open in split panes automatically
```

## Project Structure
See `ARCHITECTURE.md` for full system design.
See `AGENTS.md` for navigation map.
See `docs/exec-plans/active/v1-mvp.md` for the ordered task list.
