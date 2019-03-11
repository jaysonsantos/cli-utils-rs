use std::ops::RangeInclusive;
use std::str::FromStr;

use failure::{self, format_err, ResultExt};

#[cfg(target_os = "windows")]
pub static LINE_ENDING: &[u8; 2] = b"\r\n";

#[cfg(not(target_os = "windows"))]
pub static LINE_ENDING: &[u8; 1] = b"\n";

/// Try and convert strings to range
///
/// ```rust
/// let r = "1".parse::<Range>().unwrap().into_inner();
/// assert_eq!(r, (1..=1));
///
/// let r = "1-10".parse::<Range>().unwrap().into_inner();
/// assert_eq!(r, (1..=10));
/// ```
pub struct Range(RangeInclusive<usize>);

impl Range {
    pub fn into_inner(self) -> RangeInclusive<usize> {
        self.0
    }
}

impl FromStr for Range {
    type Err = failure::Error;

    fn from_str(data: &str) -> Result<Range, Self::Err> {
        if data.contains('-') {
            let parts: Result<Vec<usize>, _> = data.split('-').take(2).map(str::parse).collect();
            let parts = parts.context(format!("Invalid range format {:?}", data))?;
            if parts.len() != 2 {
                return Err(format_err!("Invalid line length range {}", data));
            }
            let beginning = parts[0];
            let ending = parts[1];
            return Ok(Range(beginning..=ending));
        }
        let ending: usize = data.parse()?;
        let beginning = ending;
        Ok(Range(beginning..=ending))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::Range;

    #[test]
    fn range() {
        let r = "1".parse::<Range>().unwrap().into_inner();
        assert_eq!(r, (1..=1));

        let r = "1-10".parse::<Range>().unwrap().into_inner();
        assert_eq!(r, (1..=10));
    }
}
