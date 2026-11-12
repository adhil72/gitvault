#!/usr/bin/env bash
# Builds a signed, flat apt repository from one .deb.
#
#   scripts/build-apt-repo.sh <deb> <output-dir>
#
# Needs: dpkg-scanpackages (dpkg-dev), gpg, gpgv, and the signing secret key already in the gpg
# keyring. The passphrase, if the key has one, is read from APT_GPG_PASSPHRASE and passed on
# stdin so it never appears in a process list.
#
# Output (all uploaded next to the .deb as release assets):
#   GitVault.deb  Packages  Packages.gz  Release  InRelease  Release.gpg
#   gitvault-archive-keyring.gpg (binary, for `signed-by=`)  gitvault-archive-keyring.asc (armored)
set -euo pipefail

deb="${1:?usage: build-apt-repo.sh <deb> <output-dir>}"
out="${2:?usage: build-apt-repo.sh <deb> <output-dir>}"

mkdir -p "$out"
out="$(cd "$out" && pwd)"
# A stable file name lets the repository live at releases/latest/download without versioned paths.
cp "$deb" "$out/GitVault.deb"

fingerprint="$(gpg --batch --list-secret-keys --with-colons | awk -F: '/^fpr/ { print $10; exit }')"
if [ -z "$fingerprint" ]; then
  echo "error: no GPG secret key in the keyring" >&2
  exit 1
fi

sign() {
  # --passphrase-fd 0 keeps the passphrase out of argv; gpg ignores it for keys without one.
  printf '%s' "${APT_GPG_PASSPHRASE:-}" | gpg --batch --yes --pinentry-mode loopback \
    --passphrase-fd 0 --default-key "$fingerprint" "$@"
}

cd "$out"
# Run from inside the folder so `Filename:` is the relative `./GitVault.deb`.
dpkg-scanpackages --multiversion . /dev/null > Packages 2> scan.log || { cat scan.log >&2; exit 1; }
rm scan.log
gzip -9n -k -f Packages

hashes() {
  for file in Packages Packages.gz; do
    printf ' %s %s %s\n' "$(sha256sum "$file" | cut -d' ' -f1)" "$(stat -c %s "$file")" "$file"
  done
}

{
  echo "Origin: GitVault"
  echo "Label: GitVault"
  echo "Suite: stable"
  echo "Codename: stable"
  echo "Architectures: amd64"
  echo "Date: $(LC_ALL=C date -u '+%a, %d %b %Y %H:%M:%S UTC')"
  echo "Description: GitVault desktop app and CLI"
  echo "SHA256:"
  hashes
} > Release

sign --armor --detach-sign --output Release.gpg Release
sign --clearsign --output InRelease Release

gpg --batch --export "$fingerprint" > gitvault-archive-keyring.gpg
gpg --batch --armor --export "$fingerprint" > gitvault-archive-keyring.asc

# Fail the release rather than publish a repository that clients would reject.
gpgv --keyring "$out/gitvault-archive-keyring.gpg" Release.gpg Release
gpgv --keyring "$out/gitvault-archive-keyring.gpg" InRelease
echo "Signed apt repository ready in $out (key $fingerprint)"
