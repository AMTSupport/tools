# PROJECT KNOWLEDGE BASE

## OVERVIEW

Rust monorepo of CLI tools for system administration and automation.
Nix-first build system with strict linting.

## STRUCTURE

```
tools/
├── crates/
│   ├── amt-lib/       # Core library: UI, CLI, traits
│   ├── amt-macros/    # Proc macros for derives
│   ├── obj-builder/   # Builder pattern utilities
│   ├── backup/        # Backup automation
│   ├── country-ip/    # IP geolocation lookup
│   ├── interfacer/    # API integration
│   ├── memorable-pass/# Passphrase generator
│   ├── popup/         # Simple popup notifications
│   ├── rebooter/      # Scheduled reboot manager
│   └── sys-cleaner/   # System cleanup automation
├── scripts/           # Build/release scripts
└── .github/workflows/ # CI: Nix builds, matrix per crate
```

## WHERE TO LOOK

| Task                  | Location                   | Notes                                    |
| --------------------- | -------------------------- | ---------------------------------------- |
| Add new CLI app       | `crates/*/src/main.rs`     | Follow existing structure, use `amt-lib` |
| CLI argument handling | `crates/*/src/ui/cli/`     | Uses clap with CommonFlags               |
| Shared traits         | `crates/amt-lib/src/`      | Named, Pathed, handler! macro            |
| Derive macros         | `crates/amt-macros/src/`   | EnumVariants, EnumNames, Regex           |
| Error handling        | App's `src/lib.rs`         | anyhow + thiserror pattern               |
| Tests                 | `crates/*/tests/`          | Integration tests, use test-log          |
| CI config             | `.github/workflows/ci.yml` | Nix-based, crate-level detection         |
| Add dependency        | Root `Cargo.toml`          | Add to `[workspace.dependencies]`        |

## CONVENTIONS

### CRITICAL - Will fail CI

- **NO unsafe code** - Denied by Cranky.toml
- **NO unwrap/expect/panic** - Use `?` or proper error handling
- **NO unused imports/features** - Denied
- Format with `rustfmt` (120 char width) before commit

### Code Style

- Edition 2021, resolver v2
- All crates depend on `amt-lib` for shared functionality
- Async via `tokio`, errors via `anyhow`+`thiserror`
- File operations via `fs-err` (better error messages)
- Use `tracing` for logging, not `println!`

### CLI Pattern

```rust
use amt_lib::ui::cli::{CommonFlags, handler};

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    common: CommonFlags,  // verbose, dry_run, quiet, format, update
    #[command(subcommand)]
    command: Commands,
}
```

### Commit Messages

Conventional commits enforced (cog.toml):

- `feat(crate):` - Minor version bump
- `fix(crate):` - Patch version bump
- `refactor:`, `doc:`, `test:`, `ci:`, `chore:` - No bump

## ANTI-PATTERNS

- `as any` type casting
- Empty catch blocks
- Suppressing clippy with `#[allow(...)]` without justification
- Direct `std::fs` - use `fs-err` instead
- `println!` for output - use `tracing` macros

## BUILD & TEST

```bash
# Development (via Nix)
nix develop              # Enter dev shell
cargo build              # Build all
cargo nextest run        # Run tests

# Single crate
cargo build -p backup
cargo nextest run -p backup

# Pre-commit (runs automatically)
pre-commit run --all-files

# Release
cog bump --auto          # Version bump based on commits
```

## CI NOTES

- Builds via Nix flake, not raw cargo
- Only changed crates are built
- Matrix: 6 platforms (Linux/Windows/macOS × x86_64/aarch64)
- Pre-commit hooks must pass locally first

## CRATE DEPENDENCY GRAPH

```
amt-lib ←── all apps
amt-macros ←── amt-lib
obj-builder ←── several apps
```
