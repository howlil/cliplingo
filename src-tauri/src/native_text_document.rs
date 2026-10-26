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


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextEndpoint { Start, End }

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

    // Range movement is defined by complete Unicode grapheme boundaries.
    // Never retain a historical translation in a handle or range object.
    fn grapheme_boundaries(&self) -> Vec<usize> {
        let mut positions = vec![0];
        let mut index = 0;
        for grapheme in self.name().graphemes(true) {
            index += grapheme.encode_utf16().count();
            positions.push(index);
        }
        positions
    }

    fn checked_offsets(&self, range: TextRange) -> Option<(usize, usize)> {
        (range.generation == self.generation
            && range.start_utf16 <= range.end_utf16
            && range.end_utf16 <= self.utf16.len())
            .then_some((range.start_utf16, range.end_utf16))
    }

    pub(crate) fn range_at(&self, start: usize, end: usize) -> Option<TextRange> {
        if start > end || end > self.utf16.len() { return None; }
        let boundaries = self.grapheme_boundaries();
        if boundaries.binary_search(&start).is_err()
            || boundaries.binary_search(&end).is_err() { return None; }
        Some(TextRange { generation: self.generation, start_utf16: start, end_utf16: end })
    }

    pub(crate) fn compare_endpoints(
        &self, first: TextRange, a: TextEndpoint, second: TextRange, b: TextEndpoint,
    ) -> Option<std::cmp::Ordering> {
        let (s1, e1) = self.checked_offsets(first)?;
        let (s2, e2) = self.checked_offsets(second)?;
        let left = if a == TextEndpoint::Start { s1 } else { e1 };
        let right = if b == TextEndpoint::Start { s2 } else { e2 };
        Some(left.cmp(&right))
    }

    // Returns the new range and the actual number of moved characters.
    // Crossing endpoints collapse the opposite endpoint per UIA semantics.
    pub(crate) fn move_character_endpoint(
        &self, range: TextRange, endpoint: TextEndpoint, count: isize,
    ) -> Option<(TextRange, isize)> {
        let (start, end) = self.checked_offsets(range)?;
        let boundaries = self.grapheme_boundaries();
        let old = if endpoint == TextEndpoint::Start { start } else { end };
        let index = boundaries.binary_search(&old).ok()?;
        let next = (index as isize).saturating_add(count)
            .clamp(0, (boundaries.len() - 1) as isize) as usize;
        let new_offset = boundaries[next];
        let result = if endpoint == TextEndpoint::Start {
            TextRange { start_utf16: new_offset, end_utf16: end.max(new_offset), ..range }
        } else {
            TextRange { start_utf16: start.min(new_offset), end_utf16: new_offset, ..range }
        };
        Some((result, next as isize - index as isize))
    }

    // Document is the other mandatory UIA text unit. It has two boundaries.
    pub(crate) fn move_document_endpoint(
        &self, range: TextRange, endpoint: TextEndpoint, count: isize,
    ) -> Option<(TextRange, isize)> {
        let (start, end) = self.checked_offsets(range)?;
        let old = if endpoint == TextEndpoint::Start { start } else { end };
        let next = if count > 0 { self.utf16.len() }
            else if count < 0 { 0 } else { old };
        let moved = if next == old { 0 } else if count > 0 { 1 } else { -1 };
        let result = if endpoint == TextEndpoint::Start {
            TextRange { start_utf16: next, end_utf16: end.max(next), ..range }
        } else {
            TextRange { start_utf16: start.min(next), end_utf16: next, ..range }
        };
        Some((result, moved))
    }

    /// Compare complete offsets only for the current generation. A retained
    /// COM range cannot outlive replacement of its translated target.
    pub(crate) fn compare_ranges(&self, left: TextRange, right: TextRange) -> Option<bool> {
        Some(self.checked_offsets(left)? == self.checked_offsets(right)?)
    }

    /// Move an endpoint to another *current* range. When it crosses the other
    /// endpoint, collapse that endpoint as required by UIA range semantics.
    pub(crate) fn move_endpoint_to_range(
        &self, range: TextRange, endpoint: TextEndpoint,
        other: TextRange, other_endpoint: TextEndpoint,
    ) -> Option<TextRange> {
        let (start, end) = self.checked_offsets(range)?;
        let (other_start, other_end) = self.checked_offsets(other)?;
        let destination = if other_endpoint == TextEndpoint::Start {
            other_start
        } else {
            other_end
        };
        // All application-created ranges have grapheme-safe endpoints. Reject
        // arbitrary invalid ranges before passing them to a future COM layer.
        self.range_at(start, end)?;
        self.range_at(other_start, other_end)?;
        let (next_start, next_end) = if endpoint == TextEndpoint::Start {
            (destination, end.max(destination))
        } else {
            (start.min(destination), destination)
        };
        self.range_at(next_start, next_end)
    }

    /// Expand a range to one complete grapheme at its start, or the final
    /// grapheme when the caret is at document end. Empty text stays empty.
    pub(crate) fn expand_character(&self, range: TextRange) -> Option<TextRange> {
        let (start, end) = self.checked_offsets(range)?;
        self.range_at(start, end)?;
        if self.utf16.is_empty() { return self.range_at(0, 0); }
        self.character_at(start.min(self.utf16.len() - 1))
    }

    /// Find only within the live range. Outer None means invalidated range;
    /// Some(None) is a valid search with no match. Offsets never become
    /// owned strings in the range handle, including backward search.
    ///
    /// The comparison is Unicode grapheme-aware. Case-insensitive matching
    /// uses Unicode lowercase (not locale-specific collation), so it cannot
    /// claim complete language-specific FindText semantics yet.
    pub(crate) fn find_graphemes(
        &self, within: TextRange, needle: &str,
        backward: bool, ignore_case: bool,
    ) -> Option<Option<TextRange>> {
        let (start, end) = self.checked_offsets(within)?;
        self.range_at(start, end)?;
        if needle.is_empty() { return Some(None); }
        let haystack = self.read_range(within, None)?;
        let clusters: Vec<&str> = haystack.graphemes(true).collect();
        let wanted: Vec<&str> = needle.graphemes(true).collect();
        if wanted.len() > clusters.len() { return Some(None); }
        let matches_at = |position: usize| wanted.iter().enumerate().all(|(offset, query)| {
            let current = clusters[position + offset];
            if ignore_case {
                current.to_lowercase() == query.to_lowercase()
            } else {
                current == *query
            }
        });
        let positions = 0..=(clusters.len() - wanted.len());
        let found = if backward {
            positions.rev().find(|&i| matches_at(i))
        } else {
            positions.find(|&i| matches_at(i))
        };
        let Some(index) = found else { return Some(None); };
        let prefix: usize = clusters[..index].iter().map(|c| c.encode_utf16().count()).sum();
        let length: usize = clusters[index..index + wanted.len()]
            .iter().map(|c| c.encode_utf16().count()).sum();
        Some(self.range_at(start + prefix, start + prefix + length))
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
    fn navigation_uses_graphemes_and_clamps_crossing_endpoints() {
        let mut doc = NativeTextDocument::default();
        doc.replace("A🐱e\u{301}👩‍💻\r\nZ");
        // Boundaries in UTF-16: 0,1,3,5,10,12,13.
        assert!(doc.range_at(1, 10).is_some());
        assert!(doc.range_at(2, 3).is_none()); // surrogate
        assert!(doc.range_at(4, 5).is_none()); // combining accent
        assert!(doc.range_at(6, 10).is_none()); // ZWJ sequence
        assert!(doc.range_at(11, 12).is_none()); // CRLF
        let whole = doc.document_range();
        let start = doc.range_at(1, 5).unwrap();
        let (moved, count) = doc.move_character_endpoint(start, TextEndpoint::Start, 4).unwrap();
        assert_eq!(count, 4);
        assert_eq!(moved.start_utf16, 12);
        assert_eq!(moved.end_utf16, 12);
        let (first, count) = doc.move_character_endpoint(moved, TextEndpoint::End, isize::MIN).unwrap();
        assert_eq!(count, -5);
        assert_eq!((first.start_utf16, first.end_utf16), (0, 0));
        let (last, count) = doc.move_document_endpoint(first, TextEndpoint::Start, 99).unwrap();
        assert_eq!(count, 1);
        assert_eq!((last.start_utf16, last.end_utf16), (13, 13));
        assert_eq!(doc.compare_endpoints(whole, TextEndpoint::Start, last, TextEndpoint::End),
            Some(std::cmp::Ordering::Less));
        doc.clear();
        assert_eq!(doc.move_character_endpoint(whole, TextEndpoint::Start, 1), None);
        assert_eq!(doc.move_document_endpoint(last, TextEndpoint::End, -1), None);
        assert_eq!(doc.compare_endpoints(whole, TextEndpoint::Start, last, TextEndpoint::End), None);
    }


    #[test]
    fn move_endpoint_to_range_compares_and_invalidates_after_replacement() {
        let mut document = NativeTextDocument::default();
        document.replace("Hello 🐱 World");
        let left = document.range_at(0, 5).unwrap();
        let right = document.range_at(8, 13).unwrap();
        assert_eq!(document.compare_ranges(left, left), Some(true));
        assert_eq!(document.compare_ranges(left, right), Some(false));
        let collapsed = document.move_endpoint_to_range(
            left, TextEndpoint::Start, right, TextEndpoint::End
        ).unwrap();
        assert_eq!(document.read_range(collapsed, None).as_deref(), Some(""));
        let from_end = document.move_endpoint_to_range(
            right, TextEndpoint::End, left, TextEndpoint::Start
        ).unwrap();
        assert_eq!(document.read_range(from_end, None).as_deref(), Some(""));
        document.replace("Different target");
        assert_eq!(document.compare_ranges(left, left), None);
        assert_eq!(document.move_endpoint_to_range(
            left, TextEndpoint::End, document.document_range(), TextEndpoint::End
        ), None);
    }

    #[test]
    fn find_text_is_bounded_to_live_scope_and_grapheme_boundaries() {
        let mut document = NativeTextDocument::default();
        document.replace("Cat 🐱 cat e\u{301} CAT");
        let all = document.document_range();
        let first = document.find_graphemes(all, "cat", false, true).unwrap().unwrap();
        let last = document.find_graphemes(all, "cat", true, true).unwrap().unwrap();
        assert_eq!(document.read_range(first, None).as_deref(), Some("Cat"));
        assert_eq!(document.read_range(last, None).as_deref(), Some("CAT"));
        assert_eq!(document.find_graphemes(all, "absent", false, true), Some(None));
        assert_eq!(document.find_graphemes(all, "", false, true), Some(None));
        let accent = document.find_graphemes(all, "e\u{301}", false, false).unwrap().unwrap();
        assert_eq!(document.read_range(accent, None).as_deref(), Some("e\u{301}"));
        assert_eq!(document.find_graphemes(first, "CAT", true, false), Some(None));
        assert_eq!(document.expand_character(document.range_at(3, 3).unwrap())
            .and_then(|r| document.read_range(r, None)).as_deref(), Some(" "));
        let end = document.range_at(17, 17).unwrap();
        assert_eq!(document.expand_character(end).and_then(|r| document.read_range(r, None))
            .as_deref(), Some("T"));
        document.clear();
        assert_eq!(document.find_graphemes(all, "Cat", false, false), None);
        assert_eq!(document.expand_character(all), None);
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
