<h1 align="center">Welcome to country-ip 👋</h1>
<p>
  <a href="https://github.com/AMTSupport/tools/actions/workflows/ci.yml" target="_blank">
    <img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/AMTSupport/tools/ci.yml?style=for-the-badge">
  </a>
  <a href="https://github.com/AMTSupport/tools/">
  </a>
  <a href="https://www.gnu.org/licenses/" target="_blank">
    <img alt="License: GPLv3" src="https://img.shields.io/badge/License-GPLv3-yellow.svg?style=for-the-badge" />
  </a>
</p>

> An IP to country / country to IP resolver that supports IPv4 and IPv6.
>
> Using APNIC, ARIN, RIPE, LACNIC and AFRINIC databases.

## Install

```sh
# Nix (Recommended) (Release binaries are cached)
nix run github:AMTSupport/tools#country-ip

# Cargo
cargo install --git https://github.com/AMTSupport/tools country-ip
```

## Usage

```sh
# Get the country of an IP
country-ip lookup $ip

# Get a random IP of a given country
country-ip get $country
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
