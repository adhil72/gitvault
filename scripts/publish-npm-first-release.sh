#!/usr/bin/env bash
# One-time setup: store the npm token as the NPM_TOKEN repo secret, then publish
# @adhil28/gitvault to the public npm registry. Run it from your own terminal.
#
# The token is read silently, sent to `gh secret set` on stdin, and used for the
# publish through a temporary npmrc that is deleted on exit. It is never printed
# or passed as a command-line argument.
set -euo pipefail

REPO="adhil72/gitvault"
PKG_DIR="$(cd "$(dirname "$0")/.." && pwd)/packages/node"

need() { command -v "$1" >/dev/null || { echo "missing: $1" >&2; exit 1; }; }
need gh; need npm; need pnpm; need jq

version=$(jq -r .version "$PKG_DIR/package.json")
echo "Package version: $version"

if timeout 30 npm view "@adhil28/gitvault@$version" version >/dev/null 2>&1; then
  echo "@adhil28/gitvault@$version is already on npmjs.com. Nothing to do."
  exit 0
fi

echo
echo "Paste an npm Automation access token (input is hidden), then press Enter:"
read -rs NPM_TOKEN
echo
[ -n "$NPM_TOKEN" ] || { echo "empty token" >&2; exit 1; }

echo "Setting the NPM_TOKEN secret on $REPO ..."
printf '%s' "$NPM_TOKEN" | gh secret set NPM_TOKEN --repo "$REPO"

tmp_npmrc=$(mktemp)
chmod 600 "$tmp_npmrc"
trap 'rm -f "$tmp_npmrc"' EXIT
printf '%s\n' '//registry.npmjs.org/:_authToken=${NODE_AUTH_TOKEN}' > "$tmp_npmrc"

echo "Building ..."
(cd "$(dirname "$PKG_DIR")/.." && pnpm --filter @adhil28/gitvault build)

echo "Publishing @adhil28/gitvault@$version to npmjs.com ..."
cd "$PKG_DIR"
NPM_CONFIG_USERCONFIG="$tmp_npmrc" NODE_AUTH_TOKEN="$NPM_TOKEN" npm publish --access public
unset NPM_TOKEN

echo
echo "Published. Verifying:"
timeout 30 npm view "@adhil28/gitvault@$version" version
echo "Install with: npm install @adhil28/gitvault"
