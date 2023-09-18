<h1 align="center">Memorable Password Generator</h1>

---

### Getting Started

Memorable Pass is available on [crates.io](https://crates.io/crates/memorable-pass) under the name memorable-pass.
Install it with cargo by running.
```shell
cargo install memorable-pass
```

If you don't have Cargo installed or just want to manually,
You can get the latest version of Memorable Pass from the [GitHub Releases](https://github.com/AMTSupport/tools/releases/latest).

### Usage
From the terminel you can run with the default settings by invoking the command like:
```shell
./memorable
```
---
### Settings

You can find the location for your OS below.
```yaml
Windows: {FOLDERID_RoamingAppData} # eg: C:\Users\{USERNAME}\AppData\Roaming
Linux: $HOME/.config # or $XDG_CONFIG_HOME
Mac: $HOME/Library/Preferences
```

You can also specify a custom location by using the `--config` flag.
```shell
./memorable --config /home/racci/Documents/config.toml
```
---
### Arguments
```
Usage: rpgen [OPTIONS]

Options:
  -w, --words <WORD_COUNT>
          How many words are used [default: 2]
  -m, --min-length <WORD_LENGTH_MIN>
          The minimum length of each word [default: 5]
  -M, --max-length <WORD_LENGTH_MAX>
          The maximum length of each word [default: 7]
      --digit-mode <FILL_MODE>
          The mode that is used to select where the digits are inserted [default: sandwich-all] [possible values: sandwich-each, sandwich-all, before-each, before-all, after-each, after-all]
      --digit-minimum <MINIMUM>
          The minimum number of digits to add to each filled area [default: 3]
      --digit-maximum <MAXIMUM>
          [default: 3]
      --separator-mode <MODE>
          [default: single] [possible values: none, single, random]
      --separator-chars <CHARS>
          [default: !@$%.&*-+=?:]
      --transformation-case <TRANSFORMATION_CASE>
          [default: capitalise] [possible values: none, capitalise, all-excluding-first, uppercase, random, alternating]
  -a, --amount <AMOUNT>
          The number of digits to add before the password. The number of passwords to generate [default: 3]
SUBCOMMANDS:
    generate    Generate some new passwords.
    help        Print this message or the help of the given subcommand(s)
```

---

### Configuration file

When using the configuration file not all values must be present, the default values will be used in their place.

Below you will find the default configuration file.
```toml
words = 2
min_length = 5
max_length = 7
transform = 'CAPITALISE'
separator_char = 'RANDOM'
separator_alphabet = '!@$%.&*-+=?:;'
match_random_char = true
digits_before = 0
digits_after = 3
amount = 3
```

---

### Using a configuration file in another location
When running the generate subcommand you can specify a configuration file to use.
This path will first be treated as an absolute path and if not found looked for in the current working directory.

Some examples of this would be: 
```shell
./rpgen generate /home/racci/Documents/config.toml
./rpgen generate config.toml
./rpgen generate ../config.toml
```

---

### Rule hierarchy
When running PGen rules will be assigned with the last checked value as the final value.
Meaning that rules are assigned in an order of default, config file, supplied config file and finally cli arguments.

---

### Running from a script
Instead of writing `./rpgen` and whatever options you need, you can instead use a batch, powershell or shell file like these:
- Shell script (Assuming you have `rpgen` in your path):
```shell
#!/bin/bash
rpgen -w 10 -m 5 -r -t ALTERNATING -s RANDOM -S =-;. -d 5 -D 0 -a 50 generate ~/Documents/config
```
- Powershell / Batch script
```shell
"C:\Users\Racci\Programs\PGen\pgen" generate "C:\Users\Racci\Programs\PGen\rules.toml"
```
