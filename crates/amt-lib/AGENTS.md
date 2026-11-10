# amt-lib

Core shared library for all AMT tools.

## KEY EXPORTS

| Module    | Provides                                 | Used For                                 |
| --------- | ---------------------------------------- | ---------------------------------------- |
| `ui::cli` | `CliUi`, `CommonFlags`, `handler!` macro | CLI argument parsing, oneshot/REPL modes |
| `fs`      | `create_parents`, `normalise_path`       | Safe filesystem operations               |
| `log`     | `init`                                   | Tracing setup per-crate                  |
| `named`   | `Named` trait                            | Type naming conventions                  |
| `pathed`  | `Pathed` trait                           | Path-based resource access               |
| `macros`  | `feature_trait!`, `populate!`            | Conditional trait definitions            |
| `updater` | Self-update functionality                | Optional via `updater` feature           |

## WHERE TO LOOK

| Task                        | File                              |
| --------------------------- | --------------------------------- |
| Add CLI flags               | `src/ui/cli/flags.rs`             |
| Modify CLI handler macro    | `src/ui/cli/mod.rs` -> `handler!` |
| Add oneshot command pattern | `src/ui/cli/oneshot.rs`           |
| REPL mode                   | `src/ui/cli/repl.rs`              |
| Inquire styling             | `src/ui/cli/inquire.rs`           |
| Progress bars               | `src/ui/cli/progress.rs`          |

## FEATURES

- `ui-cli` - CLI utilities (enabled by most apps)
- `ui-repl` - REPL mode support
- `updater` - Self-update capability

## HANDLER MACRO PATTERN

```rust
// Generates: OneshotHandler trait + OneshotParser struct
handler!(pub Oneshot);
```

## NIGHTLY FEATURES USED

Heavy use of nightly: `associated_type_defaults`. Expect breakage on toolchain updates.
