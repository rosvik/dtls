#!/usr/bin/env bash
# Generates fixture files carrying macOS-only extended attributes and file
# flags into tests/fixtures/generated/darwin/. Git cannot preserve xattrs
# across commit/checkout, so these fixtures aren't checked in — the
# integration tests run this script automatically, and you can run it by
# hand to (re)create the files before poking at dtls manually.
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "generate-darwin-fixtures.sh only works on macOS (uses xattr/plutil)." >&2
  exit 1
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
out_dir="$script_dir/fixtures/generated/darwin"

rm -rf "$out_dir"
mkdir -p "$out_dir"

plist_header='<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">'

# Sets $attr on $file to the binary-plist encoding of the plist body $xml_body.
set_plist_xattr() {
  local file="$1" attr="$2" xml_body="$3"
  local tmp
  tmp="$(mktemp)"
  printf '%s\n%s\n</plist>' "$plist_header" "$xml_body" | plutil -convert binary1 -o "$tmp" -
  xattr -x -w "$attr" "$(xxd -p "$tmp" | tr -d '\n')" "$file"
  rm -f "$tmp"
}

# --- Where from (kMDItemWhereFroms): download source URLs ---
where_from_file="$out_dir/downloaded-file.txt"
echo "downloaded content" >"$where_from_file"
set_plist_xattr "$where_from_file" "com.apple.metadata:kMDItemWhereFroms" \
  '<array>
  <string>https://example.com/downloaded.zip</string>
  <string>https://example.com/page.html</string>
</array>'

# --- Finder comment (kMDItemFinderComment) ---
finder_comment_file="$out_dir/commented-file.txt"
echo "a file with a Finder comment" >"$finder_comment_file"
set_plist_xattr "$finder_comment_file" "com.apple.metadata:kMDItemFinderComment" \
  '<string>Reviewed and approved</string>'

# --- Time Machine exclusion (com_apple_backup_excludeItem) ---
time_machine_file="$out_dir/time-machine-excluded.txt"
echo "excluded from backup" >"$time_machine_file"
set_plist_xattr "$time_machine_file" "com.apple.metadata:com_apple_backup_excludeItem" \
  '<string>com.apple.backupd</string>'

# --- Plain (non-plist) extended attribute ---
plain_xattr_file="$out_dir/with-xattr.txt"
echo "a file with a plain xattr" >"$plain_xattr_file"
xattr -w "com.example.test" "hello world" "$plain_xattr_file"

# --- Quarantine (com.apple.quarantine): flags;timestamp;agent;uuid ---
quarantined_file="$out_dir/quarantined.txt"
echo "downloaded and quarantined" >"$quarantined_file"
xattr -w "com.apple.quarantine" \
  "0083;5e8c5b22;Safari;F8E9B7C8-1234-5678-9ABC-DEF012345678" "$quarantined_file"

# --- Finder tags (_kMDItemUserTags): "name\ncolor-index" or plain name ---
tagged_file="$out_dir/tagged.txt"
echo "a file with Finder tags" >"$tagged_file"
set_plist_xattr "$tagged_file" "com.apple.metadata:_kMDItemUserTags" \
  '<array>
  <string>Important
6</string>
  <string>Work</string>
</array>'

# --- Hidden file flag (chflags hidden) ---
hidden_file="$out_dir/hidden.txt"
echo "hidden from Finder" >"$hidden_file"
chflags hidden "$hidden_file"

echo "Generated fixtures in $out_dir:"
for f in "$out_dir"/*; do
  echo
  echo "--- $f ---"
  xattr -l "$f"
done
