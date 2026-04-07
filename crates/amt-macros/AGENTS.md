# amt-macros

Procedural derive macros for amt-lib.

## AVAILABLE DERIVES

| Derive         | Purpose                               | Applies To               |
| -------------- | ------------------------------------- | ------------------------ |
| `EnumVariants` | Generate `VARIANTS` const array       | Enums                    |
| `EnumNames`    | Generate `NAMES` const array          | Enums                    |
| `EnumRegex`    | Generate regex pattern matching       | Enums                    |
| `CommonFields` | Extract shared fields across variants | Enums                    |
| `Delegation`   | Auto-delegate trait methods           | Enums with `#[delegate]` |
| `Builder`      | Builder pattern generation            | Structs                  |

## WHERE TO LOOK

| Task                 | File                                    |
| -------------------- | --------------------------------------- |
| Add new enum derive  | `src/enums/` + register in `src/lib.rs` |
| Modify builder logic | `src/builder.rs`                        |
| Error handling       | `error()` helper in `src/lib.rs`        |

## ADDING NEW DERIVES

1. Create handler in `src/enums/` or `src/`
2. Register in `src/lib.rs` with `#[proc_macro_derive(Name)]`
3. Add tests in `tests/`

## DELEGATION EXAMPLE

```rust
#[derive(Delegation)]
#[delegate(trait = dyn MyTrait)]
enum Sources {
    #[delegate(path = crate::sources::bitwarden::Bitwarden)]
    Bitwarden,
}
```
