use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

pub struct EpubSkeleton {
    /// Books/<book_title (book_id)>/
    pub root: PathBuf,
    pub meta_inf: PathBuf,
    pub oebps: PathBuf,
}

impl EpubSkeleton {
    /// Plan the output directory structure using the sanitized title + bookid.
    pub fn plan(base_books_dir: &Path, title: &str, bookid: &str) -> Self {
        // Maximum number of bytes in a filename.
        const MAX_BYTES: usize = 255;
        let clean_title = sanitize_filename(title);
        let root_name = if !clean_title.is_empty() {
            // Title length should take into account the bookid, space, and () characters.
            let title_max_length = MAX_BYTES.saturating_sub(3 + bookid.len());
            let truncated_title = truncate_utf8_by_byte(&clean_title, title_max_length);
            format!("{} ({})", truncated_title, bookid)
        } else {
            format!("({})", bookid)
        };
        let root_dir = base_books_dir.join(root_name);
        Self {
            meta_inf: root_dir.join("META-INF"),
            oebps: root_dir.join("OEBPS"),
            root: root_dir,
        }
    }

    /// Initialize EpubSkeleton by creating directories and required files.
    pub fn initialize(&self) -> Result<()> {
        self.create_dirs()?;
        self.write_mimetype()?;
        self.write_container_xml()?;
        Ok(())
    }

    /// Create the directories defined in the struct.
    pub fn create_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.oebps)
            .with_context(|| format!("Creating directory {}", self.oebps.display()))?;
        fs::create_dir_all(&self.meta_inf)
            .with_context(|| format!("Creating directory {}", self.meta_inf.display()))?;
        Ok(())
    }

    /// Write META-INF/container.xml pointing to OEBPS/content.opf.
    pub fn write_container_xml(&self) -> Result<()> {
        let path = self.meta_inf.join("container.xml");
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0">
            <rootfiles>
            <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
            </rootfiles>
            </container>
            "#;
        fs::write(&path, xml).with_context(|| format!("Writing file {}", path.display()))?;
        Ok(())
    }

    /// Write the plaintext "mimetype" file at the root (no newline).
    pub fn write_mimetype(&self) -> Result<()> {
        let path = self.root.join("mimetype");
        // EXACT bytes required by OCF; do not add '\n'.
        fs::write(&path, b"application/epub+zip")
            .with_context(|| format!("Writing file {}", path.display()))?;
        Ok(())
    }
}

/// Sanitize a filename component for cross‑platform compatibility.
/// Applies sensible defaults:
/// - Normalize to NFC
/// - Replace illegal characters: <>:"/\\|?*
/// - Remove control characters
/// - Collapse whitespace
/// - Trim whitespace
fn sanitize_filename(input: &str) -> String {
    // Normalize to NFC to ensure consistency - characters displayed the same are stored the same.
    let mut s = input.nfc().collect::<String>();

    // Replace illegal Windows/FAT characters + control chars
    const ILLEGAL: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut cleaned = String::with_capacity(s.len());

    for ch in s.chars() {
        if ch.is_control() || ILLEGAL.contains(&ch) {
            cleaned.push('_');
        } else {
            cleaned.push(ch);
        }
    }
    s = cleaned;

    // Collapse whitespace
    let mut cleaned = String::with_capacity(s.len());
    let mut prev_was_whitespace = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_was_whitespace {
                cleaned.push(' ');
                prev_was_whitespace = true;
            }
        } else {
            cleaned.push(ch);
            prev_was_whitespace = false;
        }
    }
    cleaned.trim().to_string()
}

/// Truncate a UTF‑8 string safely without splitting codepoints.
fn truncate_utf8_by_byte(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }

    let mut end = max_bytes;
    // Back up until we end with a non-continuation byte.
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }

    if end == 0 {
        return "";
    }

    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::EpubSkeleton;
    use tempfile::TempDir;
    use quick_xml::{Reader, events::Event};
    use std::fs;

    /// Make a temp directory with a predictable prefix.
    fn temp(label: &str) -> TempDir {
        tempfile::Builder::new()
            .prefix(&format!("safaribooks-rs-{}", label))
            .tempdir()
            .unwrap_or_else(|_| panic!("Create tempdir with label: {}", label))
    }

    #[test]
    fn initialize_skeleton() {
        // GIVEN
        let tmp = temp("initialize");
        let base = tmp.path();
        let skel = EpubSkeleton::plan(base, "A Title", "1234567890123");

        // WHEN
        skel.initialize().expect("Initialize skeleton");

        // THEN: directory structure exists
        assert!(skel.root.exists(), "Root dir missing: {}", skel.root.display());
        assert!(skel.oebps.exists(), "OEBPS dir missing: {}", skel.oebps.display());
        assert!(skel.meta_inf.exists(), "META-INF dir missing: {}", skel.meta_inf.display());
    }

    #[test]
    fn mimetype_exact() {
        // GIVEN
        let tmp = temp("mimetype");
        let base = tmp.path();
        let skel = EpubSkeleton::plan(base, "A Title", "1234567890123");

        // WHEN
        skel.create_dirs().expect("Create skeleton dirs");
        skel.write_mimetype().expect("Write mimetype");

        // THEN: file exists
        let mimetype = skel.root.join("mimetype");
        assert!(mimetype.exists(), "Mimetype file not found");

        // mimetype has *exact* bytes with *no* trailing newline.
        let bytes = fs::read(&mimetype).expect("Read mimetype");
        assert_eq!(
            bytes.as_slice(),
            b"application/epub+zip",
            "mimetype must be exactly 'application/epub+zip' with NO trailing newline"
        );
    }

    #[test]
    fn container_xml_well_formed() {
        // GIVEN
        let tmp = temp("container");
        let base = tmp.path();
        let skel = EpubSkeleton::plan(base, "Another Title", "9876543210");

        // WHEN
        skel.create_dirs().expect("Create skeleton dirs");
        skel.write_container_xml().expect("Write container.xml");

        // THEN: file exists
        let container = skel.meta_inf.join("container.xml");
        assert!(container.exists(), "META-INF/container.xml not found");

        // Parse with quick-xml to ensure it is well-formed and to inspect elements.
        let xml = fs::read_to_string(&container).expect("Read container.xml");
        let mut reader = Reader::from_str(xml.trim());

        // Walk events; ensure <container> and expected <rootfile> are present with correct attributes.
        let mut saw_container = false;
        let mut saw_rootfiles = false;
        let mut saw_rootfile_ok = false;

        let mut buf = Vec::<u8>::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e) | Event::Empty(e)) => {
                    let name_tmp = e.name();
                    let name = name_tmp.as_ref();
                    if name == b"container" {
                        saw_container = true;
                    } else if name == b"rootfiles" {
                        saw_rootfiles = true;
                    } else if name == b"rootfile" {
                        // Check attributes on rootfile
                        let mut full_path_ok = false;
                        let mut media_type_ok = false;

                        for a in e.attributes().flatten() {
                            if a.key.as_ref() == b"full-path" && a.value.as_ref() == b"OEBPS/content.opf" {
                                full_path_ok = true;
                            }
                            else if a.key.as_ref() == b"media-type"
                                && a.value.as_ref() == b"application/oebps-package+xml"
                            {
                                media_type_ok = true;
                            }
                        }
                        if full_path_ok && media_type_ok {
                            saw_rootfile_ok = true;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Ok(_) => {}
                Err(e) => panic!("XML parse error at position {}: {e}", reader.buffer_position()),
            }
            buf.clear();
        }

        assert!(saw_container, "container.xml is missing <container> root element");
        assert!(saw_rootfiles, "container.xml is missing <rootfiles> element");
        assert!(
            saw_rootfile_ok,
            "container.xml <rootfile> must have full-path='OEBPS/content.opf' \
            and media-type='application/oebps-package+xml'"
        );
    }
}
