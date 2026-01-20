use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenSelection {
    All,
    None,
    Parts(Vec<GenRange>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenRange {
    pub start: u32,
    pub end: u32,
    pub step: u32,
}

impl GenSelection {
    pub fn matches(&self, gen: u32) -> bool {
        match self {
            GenSelection::All => true,
            GenSelection::None => false,
            GenSelection::Parts(ranges) => ranges.iter().any(|range| range.matches(gen)),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, GenSelection::None)
    }
}

impl GenRange {
    pub fn matches(&self, gen: u32) -> bool {
        if gen < self.start || gen > self.end {
            return false;
        }
        let offset = gen - self.start;
        offset % self.step == 0
    }
}

#[derive(Debug, Clone)]
pub struct SelectionParseError {
    message: String,
}

impl fmt::Display for SelectionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SelectionParseError {}

pub fn parse_gen_selection(spec: &str) -> Result<GenSelection, SelectionParseError> {
    let trimmed = spec.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
        return Ok(GenSelection::None);
    }
    if trimmed.eq_ignore_ascii_case("all") {
        return Ok(GenSelection::All);
    }
    let mut ranges = Vec::new();
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let range = parse_part(part)?;
        ranges.push(range);
    }
    if ranges.is_empty() {
        return Ok(GenSelection::None);
    }
    Ok(GenSelection::Parts(ranges))
}

fn parse_part(part: &str) -> Result<GenRange, SelectionParseError> {
    let (range_part, step) = if let Some((range, step)) = part.split_once('/') {
        let step = parse_u32(step, "step", part)?;
        if step == 0 {
            return Err(SelectionParseError {
                message: format!("step must be > 0 in '{part}'"),
            });
        }
        (range.trim(), step)
    } else {
        (part, 1)
    };

    if let Some((start, end)) = range_part.split_once('-') {
        let start = parse_u32(start, "start", part)?;
        let end = parse_u32(end, "end", part)?;
        if start > end {
            return Err(SelectionParseError {
                message: format!("start must be <= end in '{part}'"),
            });
        }
        Ok(GenRange { start, end, step })
    } else {
        let value = parse_u32(range_part, "generation", part)?;
        Ok(GenRange {
            start: value,
            end: value,
            step,
        })
    }
}

fn parse_u32(value: &str, label: &str, part: &str) -> Result<u32, SelectionParseError> {
    let value = value.trim();
    value.parse::<u32>().map_err(|_| SelectionParseError {
        message: format!("invalid {label} value '{value}' in '{part}'"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all() {
        let selection = parse_gen_selection("all").unwrap();
        assert!(selection.matches(0));
        assert!(selection.matches(42));
    }

    #[test]
    fn parse_none() {
        let selection = parse_gen_selection("none").unwrap();
        assert!(!selection.matches(0));
    }

    #[test]
    fn parse_single() {
        let selection = parse_gen_selection("25").unwrap();
        assert!(selection.matches(25));
        assert!(!selection.matches(24));
    }

    #[test]
    fn parse_range() {
        let selection = parse_gen_selection("10-50").unwrap();
        assert!(selection.matches(10));
        assert!(selection.matches(50));
        assert!(!selection.matches(9));
    }

    #[test]
    fn parse_range_step() {
        let selection = parse_gen_selection("0-200/10").unwrap();
        assert!(selection.matches(0));
        assert!(selection.matches(20));
        assert!(!selection.matches(25));
    }

    #[test]
    fn parse_combo() {
        let selection = parse_gen_selection("0-2,0-500/10,250,300-320").unwrap();
        assert!(selection.matches(0));
        assert!(selection.matches(2));
        assert!(selection.matches(10));
        assert!(selection.matches(250));
        assert!(selection.matches(315));
        assert!(!selection.matches(251));
    }
}
