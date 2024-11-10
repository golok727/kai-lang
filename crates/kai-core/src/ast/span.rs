#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub fn contains(&self, idx: usize) -> bool {
        idx >= self.start && idx <= self.end
    }

    pub fn merge(&self, other: &Self) -> Self {
        let start = self.start.min(other.start);
        let end = self.end.max(other.end);
        Self { start, end }
    }

    pub fn src_text<'a>(&'a self, src: &'a str) -> &'a str {
        &src[self.start..self.end]
    }
}

#[inline]
pub fn span(start: usize, end: usize) -> Span {
    Span { start, end }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(5, 10);
        assert_eq!(span.start, 5);
        assert_eq!(span.end, 10);
    }

    #[test]
    fn test_contains_within_span() {
        let span = Span::new(5, 10);
        assert!(span.contains(5)); // start index
        assert!(span.contains(7)); // within the span
        assert!(span.contains(10)); // end index
    }

    #[test]
    fn test_contains_outside_span() {
        let span = Span::new(5, 10);
        assert!(!span.contains(4)); // before the start
        assert!(!span.contains(11)); // after the end
    }

    #[test]
    fn test_merge_disjoint_spans() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(15, 20);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 20);
    }

    #[test]
    fn test_merge_overlapping_spans() {
        let span1 = Span::new(5, 15);
        let span2 = Span::new(10, 20);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 20);
    }

    #[test]
    fn test_merge_contained_spans() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(6, 9);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 10);
    }

    #[test]
    fn test_src_text() {
        let span = Span::new(4, 10);
        let text = "Hello, world!";
        let result = span.src_text(text);
        assert_eq!(result, "o, wor");

        let span = Span::new(7, 12);
        let result = span.src_text(text);
        assert_eq!(result, "world");
    }

    #[test]
    fn test_span_macro() {
        let span = span(3, 7);
        assert_eq!(span.start, 3);
        assert_eq!(span.end, 7);
    }
}
