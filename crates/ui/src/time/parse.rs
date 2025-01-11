use std::num::ParseFloatError;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Could not parse the seconds, input: {1}, error: {0}")]
    Second(ParseFloatError, String),
    #[error("Could not parse the minutes, input: {1}, error: {0}")]
    Minute(ParseFloatError, String),
    #[error("Could not parse the hours, input: {1}, error: {0}")]
    Hour(ParseFloatError, String),
    #[error("Durations need a suffix or one `:`")]
    NoColonOrUnit(String),
}

fn second_err(e: ParseFloatError, s: &str) -> ParseError {
    ParseError::Second(e, s.to_owned())
}
fn minute_err(e: ParseFloatError, s: &str) -> ParseError {
    ParseError::Minute(e, s.to_owned())
}
fn hour_err(e: ParseFloatError, s: &str) -> ParseError {
    ParseError::Hour(e, s.to_owned())
}

/// Parses a string in format
///     hh:mm:ss,
///     mm:ss,
///     :ss,
pub(crate) fn parse_colon_duration(arg: &str) -> Result<f32, ParseError> {
    let Some((rest, seconds)) = arg.rsplit_once(':') else {
        return Err(ParseError::NoColonOrUnit(arg.to_string()));
    };

    let mut seconds = seconds.parse().map_err(|e| second_err(e, arg))?;
    if rest.is_empty() {
        return Ok(seconds);
    }

    let Some((hours, minutes)) = rest.rsplit_once(':') else {
        let minutes: f32 = rest.parse().map_err(|e| minute_err(e, arg))?;
        seconds += 60.0 * minutes;
        return Ok(seconds);
    };
    seconds += 60.0 * minutes.parse::<f32>().map_err(|e| minute_err(e, minutes))?;
    if hours.is_empty() {
        return Ok(seconds);
    };
    seconds += 60.0 * 60.0 * hours.parse::<f32>().map_err(|e| hour_err(e, hours))?;
    Ok(seconds)
}

/// Parse a string in two different formats to a `Duration`. The formats are:
///  - 10h
///  - 15m
///  - 30s
///  - hh:mm:ss,
///  - mm:ss,
///  - :ss,
///
///  suffixes like d (days), w (weeks) and y (years) are also supported
pub(crate) fn parse_duration(arg: &str) -> Result<Duration, ParseError> {
    let suffix_and_factor = [
        ('y', 365 * 24 * 60 * 60),
        ('w', 7 * 24 * 60 * 60),
        ('d', 24 * 60 * 60),
        ('h', 60 * 60),
        ('m', 60),
        ('s', 1),
    ];

    for (suffix, factor) in suffix_and_factor {
        if let Some(num) = arg.strip_suffix(suffix) {
            let num = num.parse::<f32>().map_err(|e| hour_err(e, num))?;
            return Ok(std::time::Duration::from_secs_f32(num * factor as f32));
        }
    }
    let seconds = parse_colon_duration(arg)?;
    Ok(std::time::Duration::from_secs_f32(seconds))
}
