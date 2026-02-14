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
