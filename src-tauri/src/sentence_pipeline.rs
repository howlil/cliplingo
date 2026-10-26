//! F2.8 complete-translation boundary. UAX#29 returns lossless sentence
//! substrings; every source part must be represented by a successful native
//! inference before a result may become visible. This is not semantic QA.
use unicode_segmentation::UnicodeSegmentation;

pub const MAX_SENTENCES: usize = 8;
pub const MAX_TARGET_UTF16: usize = 4_096;

#[derive(Debug, PartialEq, Eq)]
pub enum PipelineFailure<E> {
    Empty,
    TooManySentences,
    Model(E),
    EmptyTarget,
    OutputTooLong,
    Superseded,
}

fn is_abbreviation(chunk: &str) -> bool {
    let last = chunk
        .trim_end()
        .trim_end_matches(['"', '\'', '”', '’', ')', ']'])
        .split_whitespace()
        .last()
        .unwrap_or_default()
        .to_ascii_lowercase();
    matches!(last.as_str(),
        "mr." | "mrs." | "ms." | "dr." | "prof." | "sr." |
        "jr." | "st." | "vs." | "e.g." | "i.e." |
        "u.s." | "u.k." | "etc.")
}

/// Keeps UAX#29's original bytes, including punctuation-only ranges and
/// whitespace; avoids treating a known honorific as a standalone sentence.
/// A lossless split does NOT imply that an MT model preserves all meaning.
pub fn source_sentences(source: &str) -> Result<Vec<String>, PipelineFailure<()>> {
    if source.trim().is_empty() {
        return Err(PipelineFailure::Empty);
    }
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    for part in source.split_sentence_bounds() {
        let has_content = part.chars().any(char::is_alphanumeric);
        if current.is_empty() || !has_content || is_abbreviation(&current) {
            current.push_str(part);
        } else {
            segments.push(std::mem::take(&mut current));
            current.push_str(part);
        }
        if segments.len() >= MAX_SENTENCES {
            return Err(PipelineFailure::TooManySentences);
        }
    }
    if !current.is_empty() { segments.push(current); }
    if segments.len() > MAX_SENTENCES { return Err(PipelineFailure::TooManySentences); }
    // UAX#29 defines the split as lossless; this guard protects future edits
    // from discarding punctuation-only or whitespace-only spans.
    if segments.concat() != source { return Err(PipelineFailure::Empty); }
    Ok(segments)
}

/// Each segment is sent to one model call in original order.  If ANY model
/// call fails/returns empty, or the newest hotkey supersedes this request,
/// no partially translated text escapes to the native result callback.
pub fn translate_complete<E>(
    source: &str,
    mut infer: impl FnMut(&str) -> Result<String, E>,
    is_superseded: impl Fn() -> bool,
) -> Result<String, PipelineFailure<E>> {
    let segments = source_sentences(source).map_err(|e| match e {
        PipelineFailure::Empty => PipelineFailure::Empty,
        PipelineFailure::TooManySentences => PipelineFailure::TooManySentences,
        _ => unreachable!(),
    })?;
    let mut target = String::new();
    for segment in segments {
        if is_superseded() { return Err(PipelineFailure::Superseded); }
        let text = infer(segment.trim()).map_err(PipelineFailure::Model)?;
        if text.trim().is_empty() || text.contains('\0') {
            return Err(PipelineFailure::EmptyTarget);
        }
        if !target.is_empty() { target.push(' '); }
        target.push_str(text.trim());
        if target.encode_utf16().count() > MAX_TARGET_UTF16 {
            return Err(PipelineFailure::OutputTooLong);
        }
    }
    if is_superseded() { return Err(PipelineFailure::Superseded); }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn reproduces_issue_28_as_two_independent_sentences_without_source_loss() {
        let input = "The report summarizes the experiment. We tested several settings, recorded the response time, and compared the results.";
        let parts = source_sentences(input).unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts.concat(), input);
        assert!(parts[0].contains("report summarizes"));
        assert!(parts[1].contains("recorded the response time"));
        let mut called = Vec::new();
        let translated = translate_complete(input, |s| {
            called.push(s.to_owned());
            Ok::<_, ()>(format!("[{s}]"))
        }, || false).unwrap();
        assert_eq!(called.len(), 2);
        assert!(translated.contains("report summarizes"));
        assert!(translated.contains("recorded the response time"));
    }

    #[test]
    fn single_sentence_decimal_and_abbreviated_name_do_not_create_spurious_segments() {
        let cases = [
            ("The price is 25.50 dollars.", 1),
            ("Dr. Smith arrived at 09:30.", 1),
            ("Mr. Fox jumped. The dog followed.", 2),
            ("The value is 3.14. Try again.", 2),
            ("He said, \"Try again.\" Then left.", 2),
        ];
        for (input, expected) in cases {
            let segments = source_sentences(input).unwrap();
            assert_eq!(segments.len(), expected, "incorrect boundary: {input:?}: {segments:?}");
            assert_eq!(segments.concat(), input);
        }
    }

    #[test]
    fn failed_later_sentence_must_not_yield_any_partial_target() {
        let mut calls = 0;
        let result = translate_complete("First sentence. Second sentence.", |_| {
            calls += 1;
            if calls == 2 { Err("model_failed") }
            else { Ok("Translated first".to_owned()) }
        }, || false);
        assert_eq!(calls, 2);
        assert_eq!(result, Err(PipelineFailure::Model("model_failed")));
    }

    #[test]
    fn newer_hotkey_aborts_before_next_sentence_or_presentation() {
        let new_request = Cell::new(false);
        let mut calls = 0;
        let result = translate_complete("One. Two. Three.", |_| {
            calls += 1;
            new_request.set(true);
            Ok::<_, ()>("Ada".to_owned())
        }, || new_request.get());
        assert_eq!(calls, 1);
        assert_eq!(result, Err(PipelineFailure::Superseded));
    }

    #[test]
    fn refuses_oversized_or_empty_instead_of_truncating() {
        assert_eq!(source_sentences("   "), Err(PipelineFailure::Empty));
        assert_eq!(source_sentences(&"A complete sentence. ".repeat(MAX_SENTENCES+1)),
                   Err(PipelineFailure::TooManySentences));
        assert_eq!(translate_complete("Hello.", |_| Ok::<_,()>(" ".into()), || false),
                   Err(PipelineFailure::EmptyTarget));
        assert_eq!(translate_complete("Hello.", |_| Ok::<_,()>("😀".repeat(2049)), || false),
                   Err(PipelineFailure::OutputTooLong));
    }
}
