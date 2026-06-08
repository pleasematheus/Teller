# Teller

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## CI/CD

### Automatic verification

Every `push` and `pull_request` triggers the CI workflow (`.github/workflows/ci.yml`), which runs:

- TypeScript type check + frontend build (`bun run build`)
- Frontend tests (`bun run test`)
- Rust format check (`cargo fmt --check`)
- Rust lint (`cargo clippy -- -D warnings`)
- Rust tests (`cargo test`)

**Push and pull_request never trigger an installer build.**

### Manual build authorization

Building the Windows installers (NSIS + MSI) requires explicit authorization. The process has two gates:

1. The workflow must be dispatched manually -- it never runs automatically.
2. The `bundle` job references the `build-approval` GitHub Environment, which requires approval from a designated reviewer before the job executes.

To trigger a build:

```bash
# Via GitHub CLI
gh workflow run build.yml

# With release creation
gh workflow run build.yml -f create_release=true
```

Or go to **Actions > Build > Run workflow** in the GitHub UI.

After dispatch, the `verify` job runs first. If it passes, the `bundle` job waits for approval in the `build-approval` environment. A reviewer must approve via the GitHub Actions UI before the installers are built and uploaded as workflow artifacts.

Only repository maintainers configured as required reviewers in the `build-approval` environment can approve builds.
