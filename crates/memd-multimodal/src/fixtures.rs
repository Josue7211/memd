pub const PDF_HINT: &str = "MinerU should extract the PDF into text, tables, and figures.";
pub const IMAGE_HINT: &str = "RAGAnything should preserve the image-text relationship.";
pub const VIDEO_HINT: &str = "RAGAnything should expand video frames and captions.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hints_are_non_empty() {
        assert!(!PDF_HINT.is_empty());
        assert!(!IMAGE_HINT.is_empty());
        assert!(!VIDEO_HINT.is_empty());
    }
}
