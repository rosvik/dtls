#!/usr/bin/env bash
# Generates fixture files carrying macOS-only extended attributes into
# tests/fixtures/generated/darwin/. Git cannot preserve xattrs across
# commit/checkout, so these fixtures aren't checked in — run this script
# locally to (re)create them before poking at dtls by hand.
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

echo "Generated fixtures in $out_dir:"
for f in "$out_dir"/*; do
  echo
  echo "--- $f ---"
  xattr -l "$f"
done
