//! The bounded, generation-scoped text document backing the native UIA
//! provider. Owns only the current translated TARGET, never captured source.
//! This is the text-range model, not yet a public UIA TextPattern implementation.
//! A retained range must not disclose a result after replace/hide/destroy.
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextRange {
    generation: u64,
    start_utf16: usize,
    end_utf16: usize,
}

#[derive(Debug, Default)]
pub(crate) struct NativeTextDocument {
    utf16: Vec<u16>,
    generation: u64,
}

impl NativeTextDocument {
    pub(crate) fn replace(&mut self, translated_target: &str) {
        // Keep only one target. Change generation *before* publishing content,
        // while the caller holds the provider's Mutex.
        self.generation = self.generation.wrapping_add(1);
        self.utf16.clear();
        self.utf16.extend(translated_target.encode_utf16());
    }

    pub(crate) fn clear(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.utf16.clear();
    }

    pub(crate) fn name(&self) -> String {
        // replace() always receives valid Rust UTF-8. The u16 buffer therefore
        // contains a valid UTF-16 encoding even for surrogate-pair codepoints.
        String::from_utf16(&self.utf16).expect("internal translated target UTF-16")
    }

    pub(crate) fn is_empty(&self) -> bool { self.utf16.is_empty() }

    pub(crate) fn document_range(&self) -> TextRange {
        TextRange {
            generation: self.generation,
            start_utf16: 0,
            end_utf16: self.utf16.len(),
        }
    }

    // Return None when the range refers to a replaced/hidden document,
    // or its endpoints are invalid. Never return historical target text.
    pub(crate) fn read_range(&self, range: TextRange, max_utf16: Option<usize>) -> Option<String> {
        if range.generation != self.generation
            || range.start_utf16 > range.end_utf16
            || range.end_utf16 > self.utf16.len() {
            return None;
        }
        let mut end = range.end_utf16;
        if let Some(max) = max_utf16 {
            end = end.min(range.start_utf16.saturating_add(max));
        }
        // Avoid slicing in the middle of a UTF-16 surrogate pair. UIA text
        // ranges use UTF-16 offsets, but returned Rust strings must be valid.
        if end > range.start_utf16 && end < self.utf16.len()
            && (0xD800..=0xDBFF).contains(&self.utf16[end - 1])
            && (0xDC00..=0xDFFF).contains(&self.utf16[end]) {
            end -= 1;
        }
        // Reject an invalid start rather than returning a replacement char.
        String::from_utf16(&self.utf16[range.start_utf16..end]).ok()
    }

    pub(crate) fn character_at(&self, offset_utf16: usize) -> Option<TextRange> {
        if offset_utf16 >= self.utf16.len() { return None; }
        // A UIA Character unit is based on a user-perceived grapheme cluster,
        // not an individual UTF-16 surrogate code unit.
        let name = self.name();
        let mut start_utf16 = 0;
        for grapheme in name.graphemes(true) {
            let end_utf16 = start_utf16 + grapheme.encode_utf16().count();
            if offset_utf16 < end_utf16 {
                return Some(TextRange {
                    generation: self.generation,
                    start_utf16,
                    end_utf16,
                });
            }
            start_utf16 = end_utf16;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_document_includes_newlines_unicode_and_last_line() {
        let mut document = NativeTextDocument::default();
        document.replace("Baris satu.\nBaris dua: 🐱\r\nBaris terakhir.");
        let range = document.document_range();
        assert_eq!(document.read_range(range, None).as_deref(),
            Some("Baris satu.\nBaris dua: 🐱\r\nBaris terakhir."));
        assert_eq!(range.end_utf16, document.name().encode_utf16().count());
    }

    #[test]
    fn generation_invalidates_old_ranges_on_replace_and_hide() {
        let mut document = NativeTextDocument::default();
        document.replace("Private target A");
        let retained = document.document_range();
        document.replace("Target B");
        assert_eq!(document.read_range(retained, None), None);
        assert_eq!(document.read_range(document.document_range(), None).as_deref(), Some("Target B"));
        let b = document.document_range();
        document.clear();
        assert!(document.is_empty());
        assert_eq!(document.read_range(b, None), None);
        assert_eq!(document.name(), "");
        document.replace("Target C");
        assert_eq!(document.read_range(retained, None), None);
        assert_eq!(document.read_range(b, None), None);
    }

    #[test]
    fn max_text_length_never_returns_half_a_surrogate_pair() {
        let mut document = NativeTextDocument::default();
        document.replace("A🐱Z");
        let range = document.document_range();
        assert_eq!(range.end_utf16, 4);
        assert_eq!(document.read_range(range, Some(1)).as_deref(), Some("A"));
        assert_eq!(document.read_range(range, Some(2)).as_deref(), Some("A"));
        assert_eq!(document.read_range(range, Some(3)).as_deref(), Some("A🐱"));
        assert_eq!(document.read_range(range, Some(4)).as_deref(), Some("A🐱Z"));
    }

    #[test]
    fn character_unit_handles_surrogates_and_combining_marks() {
        let mut document = NativeTextDocument::default();
        document.replace("A🐱e\u{301}");
        assert_eq!(document.read_range(document.character_at(1).unwrap(), None).as_deref(),
            Some("🐱"));
        assert_eq!(document.read_range(document.character_at(2).unwrap(), None).as_deref(),
            Some("🐱"));
        assert_eq!(document.read_range(document.character_at(3).unwrap(), None).as_deref(),
            Some("e\u{301}"));
        assert_eq!(document.read_range(document.character_at(4).unwrap(), None).as_deref(),
            Some("e\u{301}"));
        assert_eq!(document.character_at(5), None);
    }

    #[test]
    fn rejects_range_with_non_boundary_or_crossing_endpoints() {
        let mut document = NativeTextDocument::default();
        document.replace("🐱Hello");
        let range = document.document_range();
        assert!(document.read_range(TextRange {
            start_utf16: 1, ..range
        }, None).is_none());
        assert!(document.read_range(TextRange {
            start_utf16: 4, end_utf16: 2, ..range
        }, None).is_none());
    }
}
