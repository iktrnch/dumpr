#!/usr/bin/env bash
set -euo pipefail

private_key="${AUR_SSH_PRIVATE_KEY:?AUR_SSH_PRIVATE_KEY is required}"
install -m 700 -d ~/.ssh
printf '%s\n' "$private_key" > ~/.ssh/id_ed25519
chmod 600 ~/.ssh/id_ed25519
ssh-keyscan aur.archlinux.org >> ~/.ssh/known_hosts
