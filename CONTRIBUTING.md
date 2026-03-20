# Contributing to OpenClaw Desktop

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

### Prerequisites
- Rust stable + cargo — https://rustup.rs
- Node.js 18+ — https://nodejs.org
- Tauri v2 system deps — https://tauri.app/start/prerequisites/

### Running locally
```bash
git clone https://github.com/monchoz/openclaw-desktop.git
cd openclaw-desktop
npm install
npm run tauri dev
```

### Running tests
```bash
# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Rust lints
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Frontend lint
npm run lint

# Frontend tests
npm test
```

## Pull Requests

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Ensure all tests pass and lints are clean
4. Write a clear PR description explaining what and why
5. Submit your PR

### Code Style

- **Rust**: Follow standard Rust conventions. No `unwrap()` in production code — use `?` with typed errors.
- **TypeScript**: `strict: true`, no implicit `any`. Use existing Tailwind utility classes.
- **Architecture**: All OpenClaw communication goes through the Rust backend. The frontend never makes direct WebSocket/HTTP calls.

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full system design. Key constraint: the Rust backend is the security boundary between the WebView and the OpenClaw Gateway.

## Reporting Issues

- Use GitHub Issues
- Include your OS, Rust version, and Node.js version
- Steps to reproduce are always helpful

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
