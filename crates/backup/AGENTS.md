# backup

Backup automation tool for credential managers (Bitwarden, 1Password).

## STRUCTURE

```
backup/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Re-exports modules
│   ├── config/
│   │   ├── backend.rs   # Storage backend config
│   │   ├── config.rs    # Main config struct
│   │   ├── rules/       # Backup scheduling rules
│   │   └── runtime.rs   # Runtime state
│   ├── sources/         # See sources/AGENTS.md
│   │   ├── bitwarden/   # Bitwarden provider
│   │   ├── op/          # 1Password provider
│   │   └── s3/          # S3 storage
│   └── ui/cli/          # CLI commands
└── tests/
```

## WHERE TO LOOK

| Task                | Location                               |
| ------------------- | -------------------------------------- |
| Add backup source   | `src/sources/` - see sources/AGENTS.md |
| Modify backup rules | `src/config/rules/`                    |
| Change CLI commands | `src/ui/cli/`                          |
| Backend storage     | `src/config/backend.rs`                |

## FEATURE FLAGS

- `sources-bitwarden` - Bitwarden support
- `sources-1password` - 1Password support
- `sources-s3` - S3 storage backend

## CONFIG SYSTEM

Config lives in `src/config/`. Uses serde for serialization. Runtime config (`runtime.rs`) holds transient state during execution.

## NOTES

- Largest crate in workspace
- Heavy async with tokio
- Uses indicatif for progress bars
- Feature-gated source providers
