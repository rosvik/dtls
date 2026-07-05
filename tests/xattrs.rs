//! Tests for src/features/xattrs.rs: extended attribute listing and the
//! macOS com.apple.* decoders.

mod common;

use common::{fixture, run};

#[test]
fn regular_file_omits_xattr_section() {
    let out = run(&fixture("hello.txt"));
    assert!(!out.contains("Extended attributes:"), "{out}");
}

/// The com.apple.* decoders, exercised against generated fixtures carrying
/// real xattrs that git can't store.
#[cfg(target_os = "macos")]
mod darwin {
    use super::common::{darwin_fixture, run};

    /// Extended attributes are arbitrary name/value pairs attached to a file
    /// (see `man 2 setxattr`). Apple's convention is reverse-DNS names, with
    /// values under the `com.apple.metadata:` prefix encoded as binary property
    /// lists and mirrored into the Spotlight index. Any xattr dtls has no
    /// specific decoder for is listed generically under "Extended attributes:"
    /// with a printable-string or hex preview of its value.
    #[test]
    fn plain_xattr_is_listed() {
        let out = run(&darwin_fixture("with-xattr.txt"));
        assert!(out.contains("Extended attributes:"), "{out}");
        assert!(out.contains("com.example.test"), "{out}");
        assert!(out.contains("hello world"), "{out}");
    }

    /// `com.apple.quarantine` is attached by browsers and other downloading
    /// apps so that Gatekeeper vets the file on first open. The value is a
    /// semicolon-separated string `flags;timestamp;agent;uuid`: quarantine
    /// flags in hex, the download time as hex Unix seconds, the app that set
    /// the flag, and a key into the QuarantineEventsV2 database. Apple doesn't
    /// document the format; the accepted reference is
    /// <https://eclecticlight.co/2017/12/11/xattr-com-apple-quarantine-the-quarantine-flag/>.
    #[test]
    fn quarantine_xattr_is_decoded() {
        let out = run(&darwin_fixture("quarantined.txt"));
        assert!(out.contains("quarantine:"), "{out}");
        assert!(out.contains("Safari"), "{out}");
        assert!(out.contains("flags=0083"), "{out}");
        assert!(
            out.contains("F8E9B7C8-1234-5678-9ABC-DEF012345678"),
            "{out}"
        );
    }

    /// `com.apple.metadata:_kMDItemUserTags` holds Finder tags as a binary
    /// plist array of strings, each `"Name\nColorIndex"` (or just `"Name"`),
    /// where the index maps 0–7 to none, gray, green, purple, blue, yellow,
    /// red, orange. The underscore prefix marks the attribute itself as
    /// private; the public API for it is the NSURL `tagNames` resource key:
    /// <https://developer.apple.com/documentation/foundation/urlresourcevalues/1792017-tagnames>.
    #[test]
    fn finder_tags_xattr_is_decoded() {
        let out = run(&darwin_fixture("tagged.txt"));
        assert!(out.contains("Finder tags:"), "{out}");
        assert!(out.contains("Important (red)"), "{out}");
        assert!(out.contains("Work"), "{out}");
    }

    /// `com.apple.metadata:kMDItemWhereFroms` records where a downloaded file
    /// came from, as a binary plist array of strings — by convention
    /// `[download URL, referrer URL]`. Decoded via the generic
    /// com.apple.metadata:* path; the schema entry's display name, description,
    /// and keywords are shown as a comment above the value:
    /// <https://developer.apple.com/documentation/coreservices/kmditemwherefroms>.
    #[test]
    fn where_from_xattr_is_decoded() {
        let out = run(&darwin_fixture("downloaded-file.txt"));
        assert!(
            out.contains("// Where from / Where the item came from / from, source, wherefrom"),
            "{out}"
        );
        assert!(
            out.contains(
                "com.apple.metadata:kMDItemWhereFroms = \
                 https://example.com/downloaded.zip, https://example.com/page.html"
            ),
            "{out}"
        );
    }

    /// `com.apple.metadata:kMDItemFinderComment` carries the comment entered
    /// in Finder's Get Info panel, as a binary plist string. (Finder also
    /// mirrors comments into .DS_Store; the xattr is what Spotlight indexes.)
    /// Decoded via the generic com.apple.metadata:* path; the schema entry's
    /// display name "Spotlight comment" leads the comment above the value:
    /// <https://developer.apple.com/documentation/coreservices/kmditemfindercomment>.
    #[test]
    fn finder_comment_xattr_is_decoded() {
        let out = run(&darwin_fixture("commented-file.txt"));
        assert!(
            out.contains("// Spotlight comment / Spotlight comment for this item"),
            "{out}"
        );
        assert!(
            out.contains("com.apple.metadata:kMDItemFinderComment = Reviewed and approved"),
            "{out}"
        );
    }

    /// `com.apple.metadata:com_apple_backup_excludeItem` marks the file as
    /// excluded from Time Machine backups. It's written by
    /// `tmutil addexclusion` (see `man 8 tmutil`) or the Core Services
    /// `CSBackupSetItemExcluded` API, and its value is a binary plist string
    /// naming the subsystem honouring the exclusion — in practice always
    /// "com.apple.backupd". Decoded via the generic com.apple.metadata:* path;
    /// the key isn't in the schema, so no descriptive comment is shown.
    #[test]
    fn time_machine_exclusion_xattr_is_decoded() {
        let out = run(&darwin_fixture("time-machine-excluded.txt"));
        assert!(
            out.contains("com.apple.metadata:com_apple_backup_excludeItem = com.apple.backupd"),
            "{out}"
        );
    }

    /// Any other `com.apple.metadata:<key>` attribute holds the value of the
    /// Spotlight attribute `<key>` as a binary plist, mirrored into the
    /// Spotlight index by mds. dtls decodes the whole family generically.
    /// `kMDItemDownloadedDate` (the download time, written by browsers
    /// alongside `kMDItemWhereFroms`) is absent from the `mdimport -A` schema
    /// on current macOS, so no descriptive comment is shown:
    /// <https://developer.apple.com/library/archive/documentation/CoreServices/Reference/MetadataAttributesRef/Reference/CommonAttrs.html>.
    #[test]
    fn downloaded_date_xattr_is_decoded_generically() {
        use chrono::{DateTime, Local};
        let out = run(&darwin_fixture("downloaded-date.txt"));
        let expected = DateTime::from_timestamp(1_600_000_000, 0)
            .unwrap()
            .with_timezone(&Local)
            .format("%Y-%m-%d %H:%M:%S %z")
            .to_string();
        assert!(
            out.contains("com.apple.metadata:kMDItemDownloadedDate ="),
            "{out}"
        );
        assert!(out.contains(&expected), "{out}");
    }

    /// A `com.apple.metadata:*` key dtls has never heard of still decodes
    /// generically, with no descriptive comment when the vendored `mdimport -A`
    /// schema has no entry for it. Values that aren't scalars or flat
    /// arrays of scalars (nested dicts/arrays) render as compact plist debug
    /// output rather than falling back to a hex dump.
    #[test]
    fn nested_metadata_xattr_rendered_as_debug() {
        let out = run(&darwin_fixture("generic-metadata.txt"));
        assert!(
            out.contains("com.apple.metadata:kMDItemDtlsTestKey ="),
            "{out}"
        );
        assert!(out.contains("version"), "{out}");
        assert!(!out.contains("0x62706c697374"), "{out}");
    }

    /// An xattr value that is neither valid UTF-8 nor a binary plist falls
    /// back to a hex preview with the byte count.
    #[test]
    fn binary_xattr_is_hex_previewed() {
        let out = run(&darwin_fixture("with-xattr.txt"));
        assert!(
            out.contains("com.example.binary = 0xdeadbeef (4 bytes)"),
            "{out}"
        );
    }

    /// An xattr outside the com.apple.metadata: namespace whose value starts
    /// with the binary-plist magic `bplist00` is decoded as a plist rather
    /// than hex-dumped.
    #[test]
    fn bplist_xattr_without_decoder_is_decoded() {
        let out = run(&darwin_fixture("custom-bplist.txt"));
        assert!(out.contains("com.example.bplist = one, two"), "{out}");
        assert!(!out.contains("0x62706c697374"), "{out}");
    }
}
