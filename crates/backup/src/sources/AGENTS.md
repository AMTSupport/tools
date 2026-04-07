# backup/sources

Backup source providers and utilities.

## PROVIDERS

| Provider  | Feature Flag        | Directory    |
| --------- | ------------------- | ------------ |
| Bitwarden | `sources-bitwarden` | `bitwarden/` |
| 1Password | `sources-1password` | `op/`        |
| S3        | `sources-s3`        | `s3/`        |

## SHARED UTILITIES

| File            | Purpose                        |
| --------------- | ------------------------------ |
| `auto_prune.rs` | Automatic old backup cleanup   |
| `downloader.rs` | Generic download with progress |
| `exporter.rs`   | Export format handling         |
| `getter.rs`     | Internal helper                |

## ADDING NEW PROVIDER

1. Create feature flag in `Cargo.toml`: `sources-{name} = []`
2. Create directory: `src/sources/{name}/`
3. Add to `mod.rs`:
   ```rust
   #[cfg(feature = "sources-{name}")]
   pub mod {name};
   ```
4. Implement required traits (see existing providers)
5. Wire up in CLI: `src/ui/cli/`

## DOWNLOAD HELPER

```rust
// Use shared download utilities
download_to(size, stream, &path, &progress).await?;
```
