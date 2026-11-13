#!/bin/sh
# Removes the `gv` alias created by postinst (only when it still points at gitvault).
set -e
if [ "$(readlink /usr/bin/gv 2>/dev/null)" = "/usr/bin/gitvault" ]; then
  rm -f /usr/bin/gv
fi
exit 0
