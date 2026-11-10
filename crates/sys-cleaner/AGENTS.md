# sys-cleaner

Modular system cleanup automation.

## STRUCTURE

```
sys-cleaner/
├── src/
│   ├── cleaners/
│   │   ├── impls/       # 8 cleaner implementations
│   │   ├── cleaner.rs   # Cleaner trait
│   │   ├── location.rs  # Path resolution
│   │   └── mod.rs       # CleanerError, helpers
│   ├── rule/
│   │   ├── age.rs       # Age-based rules
│   │   └── mod.rs
│   ├── config/          # CLI config, runtime
│   ├── application.rs   # App orchestration
│   └── main.rs
└── tests/cleaners/
```

## CLEANERS

Located in `src/cleaners/impls/`:
- `browsers.rs` - Browser cache cleanup
- `trash.rs` - Trash/recycle bin
- Others in `impls/` directory

## ADDING A CLEANER

1. Create `src/cleaners/impls/{name}.rs`
2. Implement `CleanerInternal` trait from `cleaner.rs`
3. Register in `src/cleaners/impls/mod.rs`
4. Add delegated variant to `Cleaner` enum in `src/cleaners/cleaner.rs` (enables CLI selection)

## RULES

Rules in `src/rule/` define cleanup conditions:
- `age.rs` - Delete files older than threshold

## ENV PATH HELPERS

```rust
// Get path from env var (with validation)
env_path("XDG_CACHE_HOME".to_owned())?;  // Returns PathBuf
env_dir("XDG_CACHE_HOME".to_owned())?;   // Validates is directory
```
