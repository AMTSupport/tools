# memorable-pass

Passphrase generator with configurable rules.

## STRUCTURE

```
memorable-pass/
├── assets/words.json    # Word dictionary by length
├── src/
│   ├── config/
│   │   └── asset.rs     # WORDS static (from build.rs)
│   ├── processor/
│   │   ├── processor.rs # Word transformation pipeline
│   │   └── word.rs      # Word abstraction
│   ├── rules/
│   │   ├── addition/    # Digit/separator rules
│   │   ├── transformation/ # Case rules
│   │   ├── rule.rs      # Rule trait
│   │   └── mod.rs       # Rules struct
│   └── ui/cli/
└── build.rs             # Embeds words.json at compile
```

## GENERATION PIPELINE

1. Select random words matching length criteria
2. Apply rules via `Processor`:
   - `addition_digits` - Add numbers
   - `addition_separator` - Add separators
   - `transformation_case` - Case transform
3. `processor.finish()` → final password

## ADDING RULES

1. Create in `src/rules/addition/` or `src/rules/transformation/`
2. Implement `Rule` trait from `rule.rs`
3. Add field to `Rules` struct in `src/rules/mod.rs`
4. Wire into `generate()` in `src/lib.rs`

## WORD DICTIONARY

`assets/words.json` keyed by word length. Embedded at compile time via `build.rs` → `WORDS` static in `config/asset.rs`.
