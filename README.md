# WhatsApp Desktop

A WhatsApp desktop application for Linux built with [Tauri](https://tauri.app/).

[![License MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/license/mit)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](.github/CODE_OF_CONDUCT.md)
[![CodeQL](https://github.com/vascoguita/whatsapp-desktop/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/vascoguita/whatsapp-desktop/actions/workflows/github-code-scanning/codeql)
[![Dependabot](https://github.com/vascoguita/whatsapp-desktop/actions/workflows/dependabot/dependabot-updates/badge.svg)](https://github.com/vascoguita/whatsapp-desktop/actions/workflows/dependabot/dependabot-updates)
[![Build](https://github.com/vascoguita/whatsapp-desktop/actions/workflows/build.yml/badge.svg)](https://github.com/vascoguita/whatsapp-desktop/actions/workflows/build.yml)

![WhatsApp Desktop banner](https://repository-images.githubusercontent.com/1329120698/ea383df6-3eba-43f5-bbe1-b81272d91cd7)

## Installation

### Debian / Ubuntu (APT)

1. Add repository:
```bash
echo "deb [trusted=yes] https://apt.fury.io/vascoguita/ /" | sudo tee /etc/apt/sources.list.d/whatsapp-desktop.list
sudo apt update
```

2. Install:
```bash
sudo apt install whats-app-desktop
```

### Fedora / RHEL (YUM)

1. Add repository:
```bash
echo "[fury]
name=WhatsApp Desktop Repository
baseurl=https://yum.fury.io/vascoguita/
enabled=1
gpgcheck=0" | sudo tee /etc/yum.repos.d/whatsapp-desktop.repo
```

2. Install:
```bash
sudo yum install whats-app-desktop
```

### Arch Linux (Pacman)

1. Add repository:
```bash
echo "[fury]
SigLevel = Optional
Server = https://pacman.fury.io/vascoguita/\$arch" | sudo tee -a /etc/pacman.conf
sudo pacman -Syu
```

2. Install:
```bash
sudo pacman -S whatsapp-desktop
```

### AppImage (Universal)

1. Download:

Download the `.AppImage` file from the [latest GitHub Release](https://github.com/vascoguita/whatsapp-desktop/releases/latest).

2. Run:
```bash
chmod +x WhatsApp\ Desktop_*_amd64.AppImage
./WhatsApp\ Desktop_*_amd64.AppImage
```

## License

This project is licensed under the [MIT License](LICENSE).

## Code of Conduct

Please review our [Code of Conduct](.github/CODE_OF_CONDUCT.md) to understand
the expectations for behavior within the project community.

## Security Policy

For information on our security policy and reporting vulnerabilities, please
check our [Security Policy](.github/SECURITY.md).

## Contributing Guidelines

We welcome contributions! Before getting started, please read our
[Contributing Guidelines](.github/CONTRIBUTING.md) for information on how to
contribute to the project.

## Acknowledgements

- Dimitris Lampridis ([@gnulabis](https://github.com/gnulabis)) for testing
  the releases on Arch Linux.
- Jimil Desai ([@jimil749](https://github.com/jimil749)) for testing the
  releases on Ubuntu.

## Support

Development of this project is supported by [Ensita](https://www.ensita.org/),
a non-profit organization building open-source, privacy-respecting software.
