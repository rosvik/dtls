> [!NOTE]
> This project is very much vibe coded. Lower your expectations code quality wise :^)

# dtls

A small Rust CLI that prints detailed information about a file. (Pronounciation: `details`)

```
$ dtls photo.jpg
photo.jpg (/Users/you/photos/photo.jpg)
─────────────────────────────────────────
Type:        image/jpeg (jpg)
Size:        373 B (373 bytes)
Permissions: rw-r--r-- (0644)
Owner:       you:staff (501:20)
Inode:       102176003
Created:     2026-05-22 23:30:24 +0200 / Modified: 2026-05-22 23:30:24 +0200 / Accessed: 2026-05-22 23:30:25 +0200
SHA256:      659011f7cc1a10a40c9064d29144d956a4c9ceda4220443297d997af2e3ca532
Extended attributes:
  com.apple.macl = 0x0d41d8cd98f00b204e9800998ecf8427e0000000000000000000000000000... (72 bytes)
  // Where from / Where the item came from / from, source, wherefrom
  com.apple.metadata:kMDItemWhereFroms = https://example.com/photo.jpg?fm=jpg&q=80, https://example.com/photo.jpg
  com.apple.provenance = 0xd41d8cd98f00b204e98009 (11 bytes)
  com.apple.quarantine = quarantine: flags=0281 at=2026-07-05 13:08:16 +0200 by=Waterfox event=9b577bfc-c658-4368-88ba-d52dc3e377af
EXIF:
  ExposureTime = 1/250 s
  FNumber = f/2.8
  PhotographicSensitivity = 400
  FocalLength = 50 mm
```

## Installation

Download a binary from [releases](https://github.com/rosvik/dtls/releases), or build and install with Cargo:

```sh
cargo install --git https://github.com/rosvik/dtls
```

## Features

- File name, absolute path, size
- File type detection: magic-byte MIME via [`infer`](https://crates.io/crates/infer), or text-vs-binary with character encoding via [`chardetng`](https://crates.io/crates/chardetng)
- Permissions (ls-style + octal), owner and group, inode, hard-link count
- Created / modified / accessed timestamps
- Symlink target
- Extended attributes
- macOS BSD file flags (`hidden`, `uchg`, `schg`, …)
- SHA256 hash
- EXIF metadata for images via [`kamadak-exif`](https://crates.io/crates/kamadak-exif) (JPEG, TIFF, HEIF, PNG, WebP)
- PNG metadata (IHDR): dimensions, bit depth, color type, compression, filter and interlace method
