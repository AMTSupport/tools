# interfacer

API integration tool for MSP platforms.

## ENDPOINTS

| Endpoint | Directory              | Status |
| -------- | ---------------------- | ------ |
| Hudu     | `src/endpoints/hudu/`  | WIP    |
| N-able   | `src/endpoints/nable/` | WIP    |

## STRUCTURE

```
interfacer/
├── src/
│   ├── endpoints/
│   │   ├── endpoint.rs  # Endpoint trait
│   │   ├── hudu/        # Hudu API
│   │   └── nable/       # N-able API
│   ├── manager/
│   │   ├── mod.rs
│   │   └── structs.rs   # Shared types
│   └── main.rs
```

## ADDING ENDPOINTS

1. Create `src/endpoints/{name}/`
2. Implement `Endpoint` trait from `endpoint.rs`
3. Add structs in endpoint directory
4. Register in `src/endpoints/mod.rs`

## NOTE

Currently has compilation issues - WIP crate. Check `lsp_diagnostics` before working here.
