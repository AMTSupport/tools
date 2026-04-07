# country-ip

IP geolocation lookup using DB-IP database.

## WHERE TO LOOK

| Task                | Location          |
| ------------------- | ----------------- |
| Database handling   | `src/db_ip.rs`    |
| IP record structure | `src/record.rs`   |
| Registry logic      | `src/registry.rs` |
| CLI interface       | `src/ui/cli/`     |
| GUI interface       | `src/ui/gui/`     |

## DATA FLOW

1. Load/download DB-IP database
2. Parse IP address input
3. Query database for country info
4. Display via CLI or GUI

## DUAL UI

Has both CLI and GUI implementations:

- CLI: `src/ui/cli/`
- GUI: `src/ui/gui/`

Both share core logic in `src/lib.rs`.
