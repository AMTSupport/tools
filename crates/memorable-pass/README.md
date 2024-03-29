<h1 align="center">Welcome to country-ip 👋</h1>
<p>
  <a href="https://github.com/AMTSupport/tools/actions/workflows/ci.yml" target="_blank">
    <img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/AMTSupport/tools/ci.yml?style=for-the-badge">
  </a>
  <a href="https://www.gnu.org/licenses/" target="_blank">
    <img alt="License: GPLv3" src="https://img.shields.io/badge/License-GPLv3-yellow.svg?style=for-the-badge" />
  </a>
</p>

> A memorable password generator written for secure and easy to remember passwords.
>
> Passwords are generated using a cryptographically secure random number generator.

## Install

```sh
# Nix (Recommended) (Release binaries are cached)
nix run github:AMTSupport/tools#memorable-pass

# Cargo
cargo install --git https://github.com/AMTSupport/tools memorable-pass
```

## Usage

```sh
# Generate a password with the default settings
memorable-pass generate

# Generate a more secure password by increasing the word length
memorable-pass generate --min-length 7 --max-length 9

# Generate a password with alternating case characters
memorable-pass generate --transformation-case alternating

# View all possible rules
memorable-pass help generate
```

## Author

👤 **James Draycott**

* Github: [@DaRacci](https://github.com/DaRacci)

## 🤝 Contributing

Contributions, issues and feature requests are welcome!<br />Feel free to check [issues page](https://github.com/AMTSupport/tools/issues). You can also take a look at the [contributing guide](https://github.com/AMTSupport/tools/blob/master/CONTRIBUTING.md).

## Show your support

Give a ⭐️ if this project helped you!

## 📝 License

Copyright © 2023 [James Draycott](https://github.com/DaRacci).<br />
This project is [GPLv3](https://www.gnu.org/licenses/) licensed.
