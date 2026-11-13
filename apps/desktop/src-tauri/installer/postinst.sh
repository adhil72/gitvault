#!/bin/sh
# Adds the short `gv` alias next to the `gitvault` command the package installs.
set -e
if [ -x /usr/bin/gitvault ]; then
  ln -sf /usr/bin/gitvault /usr/bin/gv
fi
exit 0
