<h1 align="center">Welcome to cleaner 👋</h1>
<p>
  <a href="https://github.com/AMTSupport/tools/actions/workflows/ci.yml" target="_blank">
    <img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/AMTSupport/tools/ci.yml?style=for-the-badge">
  </a>
  <a href="https://www.gnu.org/licenses/" target="_blank">
    <img alt="License: GPLv3" src="https://img.shields.io/badge/License-GPLv3-yellow.svg?style=for-the-badge" />
  </a>
</p>

> A modular cleaner with dynamic rules, written blazing fast.

## Install

```sh
# Nix (Recommended) (Release binaries are cached)
nix run github:AMTSupport/tools#cleaner --accept-flake-config

# Cargo
cargo install --git https://github.com/AMTSupport/tools cleaner
```

## Usage

```sh
# Run all cleaners
cleaner

# Run specific cleaners (See all cleaners in `cleaner --help`)
cleaner -- trash browsers

# Run in quiet mode
cleaner --quiet
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
