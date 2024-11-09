#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    start: usize,
    end: usize,
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
}

#[inline]
pub fn span(start: usize, end: usize) -> Span {
    Span { start, end }
}
