// Copyright ⓒ 2016 Daniel Keep.
//
// Licensed under the MIT license (see LICENSE or <http://opensource.org
// /licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
// <http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
// files in the project carrying such notice may not be copied, modified,
// or distributed except according to those terms.
//
//! Scanner implementations for `std::time` types.
//!
use std::time::Duration;
use strcursor::StrCursor;
use ScanError;
use input::ScanInput;
use scanner::ScanFromStr;
use util::MsgErr;

/**
Parses an ISO 8601 format duration into a `std::time::Duration`.

Specifically, it supports the following syntax:

```text
PT[nH][nM][nS]
```

Each `n` is either an integer or a fractional value using `.` or `,` as the decimal point.  It supports durations down to nanosecond precision using fractional seconds.  Each component may have an arbitrarily large value (*e.g.* `PT2H76M`).

## `duration-iso8601-dates` feature

By default, durations involving date components (years, months, weeks, and days) are not enabled.  This is because the `Duration` type stores nanoseconds.  It is impossible to convert date components into seconds.  As a simple example, consider that "1 month", depending on context, could reasonably be converted into the equivalent of 28 days, 29 days, 30 days, 31 days, 30 days and 8 hours, 30 days 8 hours and 30 minutes, or some other value offset by plus or minus one leap second in order to account for changes in the Earth's rotational and/or orbital speeds.

The only *correct* way to store such durations is in their fully decomposed, component form.  Rust does not support this, thus they are disabled.

However, parsing such durations may occasionally be useful in limited contexts.  Provided you understand the above drawbacks, support for these durations can be enabled with the `duration-iso8601-dates` feature.  It uses the following conversions:

* 1 year = 365.25 days (one fourth of 3×365 + 366 days)
* 1 month = 30 days, 10 hours, 30 minutes (one twelfth of 365.25 days)
* 1 week = 7 days
* 1 day = 24 hours

With this feature, the following additional syntaxes are enabled:

* `P[nY][nM][nD][T[nH][nM][nS]]` - a fuller form of the above syntax.
* `Pyyyy-mm-ddThh:mm:ss`, `PyyyymmddThhmmss` - shorthand for a "full" date duration; note that the individual components *may not* exceed their "conventional" maximum value; *e.g.* you cannot have 25 hours.
* `PnW` - number of weeks.
*/
pub enum Iso8601Duration {}

impl<'a> ScanFromStr<'a> for Iso8601Duration {
    type Output = Duration;
    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let cur = StrCursor::new_at_start(s.as_str());
        let (dur, cur) = try!(scan_8601(cur));
        Ok((dur, cur.byte_pos()))
    }
}

const SECS_IN_SEC: u64 = 1;
const SECS_IN_MIN: u64 = 60;
const SECS_IN_HOUR: u64 = 60 * SECS_IN_MIN;

#[cfg(feature="duration-iso8601-dates")]
const SECS_IN_DAY: u64 = 24 * SECS_IN_HOUR;
#[cfg(feature="duration-iso8601-dates")]
const SECS_IN_WEEK: u64 = 7 * SECS_IN_DAY;
#[cfg(feature="duration-iso8601-dates")]
const SECS_IN_MONTH: u64 = 30 * SECS_IN_DAY + 10 * SECS_IN_HOUR + 30 * SECS_IN_MIN;
#[cfg(feature="duration-iso8601-dates")]
const SECS_IN_YEAR: u64 = 365 * SECS_IN_DAY + 6 * SECS_IN_HOUR;

const NANOS_IN_SEC: u32 = 1_000_000_000;

#[cfg(not(feature="duration-iso8601-dates"))]
#[cfg(test)]
#[test]
fn test_iso_8601_duration_dates() {
    use ScanError as SE;
    use ScanErrorKind as SEK;

    let scan = Iso8601Duration::scan_from;

    assert_match!(
        scan("P1W"),
        Err(SE { kind: SEK::Syntax(_), ref at, .. }) if at.offset() == 0
    );
    assert_match!(
        scan("P1Y"),
        Err(SE { kind: SEK::Syntax(_), ref at, .. }) if at.offset() == 0
    );
    assert_match!(
        scan("P1M"),
        Err(SE { kind: SEK::Syntax(_), ref at, .. }) if at.offset() == 0
    );
    assert_match!(
        scan("P1D"),
        Err(SE { kind: SEK::Syntax(_), ref at, .. }) if at.offset() == 0
    );
}

#[cfg(feature="duration-iso8601-dates")]
#[cfg(test)]
#[test]
fn test_iso_8601_duration_dates() {
    let scan = Iso8601Duration::scan_from;

    assert_match!(
        scan("P1W"),
        Ok((d, 3)) if d == Duration::new(SECS_IN_WEEK, 0)
    );
    assert_match!(
        scan("P1Y"),
        Ok((d, 3)) if d == Duration::new(SECS_IN_YEAR, 0)
    );
    assert_match!(
        scan("P1M"),
        Ok((d, 3)) if d == Duration::new(SECS_IN_MONTH, 0)
    );
    assert_match!(
        scan("P1D"),
        Ok((d, 3)) if d == Duration::new(SECS_IN_DAY, 0)
    );

    assert_match!(
        scan("P1Y2M3DT4H5M6.7S"),
        Ok((d, 16)) if d == Duration::new(
            1*SECS_IN_YEAR
            + 2*SECS_IN_MONTH
            + 3*SECS_IN_DAY
            + 4*SECS_IN_HOUR
            + 5*SECS_IN_MIN
            + 6*SECS_IN_SEC,
            700_000_000
        )
    );

    assert_match!(
        scan("P6789-01-23T12:34:56"),
        Ok((d, 20)) if d == Duration::new(
            6789*SECS_IN_YEAR
            + 01*SECS_IN_MONTH
            + 23*SECS_IN_DAY
            + 12*SECS_IN_HOUR
            + 34*SECS_IN_MIN
            + 56*SECS_IN_SEC,
            0
        )
    );

    assert_match!(
        scan("P67890123T123456"),
        Ok((d, 16)) if d == Duration::new(
            6789*SECS_IN_YEAR
            + 01*SECS_IN_MONTH
            + 23*SECS_IN_DAY
            + 12*SECS_IN_HOUR
            + 34*SECS_IN_MIN
            + 56*SECS_IN_SEC,
            0
        )
    );
}

#[cfg(test)]
#[test]
fn test_iso_8601_duration() {
    use ScanError as SE;
    use ScanErrorKind as SEK;

    let scan = Iso8601Duration::scan_from;

    assert_match!(scan("PT1H"), Ok((d, 4)) if d == Duration::new(SECS_IN_HOUR, 0));
    assert_match!(scan("PT1M"), Ok((d, 4)) if d == Duration::new(SECS_IN_MIN, 0));
    assert_match!(scan("PT1S"), Ok((d, 4)) if d == Duration::new(SECS_IN_SEC, 0));
    assert_match!(
        scan("PT12H34M56.78S"),
        Ok((d, 14)) if d == Duration::new(12*SECS_IN_HOUR + 34*SECS_IN_MIN + 56, 780_000_000)
    );
    assert_match!(
        scan("PT34M56.78S"),
        Ok((d, 11)) if d == Duration::new(34*SECS_IN_MIN + 56, 780_000_000)
    );
    assert_match!(
        scan("PT12H56.78S"),
        Ok((d, 11)) if d == Duration::new(12*SECS_IN_HOUR + 56, 780_000_000)
    );
    assert_match!(
        scan("PT12H34M56S"),
        Ok((d, 11)) if d == Duration::new(12*SECS_IN_HOUR + 34*SECS_IN_MIN + 56, 0)
    );
    assert_match!(
        scan("PT12H34M"),
        Ok((d, 8)) if d == Duration::new(12*SECS_IN_HOUR + 34*SECS_IN_MIN, 0)
    );

    assert_match!(
        scan("PT0.5H"),
        Ok((d, 6)) if d == Duration::new(30*SECS_IN_MIN, 0)
    );
    assert_match!(
        scan("PT0.5M"),
        Ok((d, 6)) if d == Duration::new(30*SECS_IN_SEC, 0)
    );
    assert_match!(
        scan("PT0.5S"),
        Ok((d, 6)) if d == Duration::new(0, NANOS_IN_SEC/2)
    );

    assert_match!(
        scan("PT0.000000001S"),
        Ok((d, 14)) if d == Duration::new(0, 1)
    );
    assert_match!(
        scan("PT0.000000000016666666666666667M"),
        Ok((d, 32)) if d == Duration::new(0, 1)
    );
    assert_match!(
        scan("PT0.0000000000002777777777777778H"),
        Ok((d, 33)) if d == Duration::new(0, 1)
    );

    assert_match!(scan(""), Err());
    assert_match!(scan("a while"), Err());
    assert_match!(scan("P"), Err());
    assert_match!(scan("Px"), Err());
    assert_match!(scan("PY"), Err());
    assert_match!(scan("PM"), Err());
    assert_match!(scan("PD"), Err());
    assert_match!(scan("PW"), Err());
    assert_match!(scan("P1H"), Err());
    assert_match!(scan("P1S"), Err());
}

type ScanResult<T, L = usize> = Result<(T, L), ScanError>;

fn scan_8601(cur: StrCursor) -> ScanResult<Duration, StrCursor> {
    // See: <https://en.wikipedia.org/wiki/ISO_8601#Durations>,
    // <https://html.spec.whatwg.org/multipage/infrastructure.html
    // #valid-duration-string>.
    //
    let cur = match cur.next_cp() {
        Some(('P', cur)) => cur,
        _ => return Err(ScanError::syntax("expected `P`").add_offset(cur.byte_pos())),
    };

    return match cur.next_cp() {
        Some(('T', cur)) => given_date(Duration::new(0, 0), cur),
        _ => date(cur),
    };

    #[cfg(not(feature="duration-iso8601-dates"))]
    fn date(_: StrCursor) -> ScanResult<Duration, StrCursor> {
        Err(ScanError::syntax("durations with date components not supported"))
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn date(cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let (int, int_cur) = try!(scan_integer(cur));
        let int_len = cur.slice_between(int_cur).unwrap().len();

        match int_cur.next_cp() {
            Some(('.', cur)) | Some((',', cur)) => date_leading_frac(int, cur),
            Some(('T', cur)) => {
                if int_len != "YYYYMMDD".len() {
                    return Err(ScanError::syntax("expected date in `YYYYMMDD` format")
                                   .add_offset(cur.byte_pos()));
                }
                time_compound(int, cur)
            }
            Some(('-', cur)) => {
                if int_len != "YYYY".len() {
                    return Err(ScanError::syntax("expected year in `YYYY-MM-DD` format")
                                   .add_offset(cur.byte_pos()));
                }
                date_split_month(try!(dur_years(int, 0.0)), cur)
            }
            Some(('Y', cur)) => {
                let y = try!(dur_years(int, 0.0));
                given_year(y, cur)
            }
            Some(('M', cur)) => {
                let m = try!(dur_months(int, 0.0));
                given_month(m, cur)
            }
            Some(('D', cur)) => {
                let d = try!(dur_days(int, 0.0));
                given_day(d, cur)
            }
            Some(('W', cur)) => {
                let w = try!(dur_weeks(int, 0.0));
                Ok((w, cur))
            }
            _ => {
                Err(ScanError::syntax("expected number followed by one of `T`, `Y`, `M`, `D`, or \
                                       `W`")
                        .add_offset(cur.byte_pos()))
            }
        }
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn date_leading_frac(int: u64, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let ((int, frac), cur) = try!(scan_real_frac(int, cur));
        match cur.next_cp() {
            Some(('Y', cur)) => given_year(try!(dur_years(int, frac)), cur),
            Some(('M', cur)) => given_month(try!(dur_months(int, frac)), cur),
            Some(('D', cur)) => given_day(try!(dur_days(int, frac)), cur),
            Some(('W', cur)) => {
                let w = try!(dur_weeks(int, frac));
                Ok((w, cur))
            }
            _ => {
                Err(ScanError::syntax("expected real number followed by one of `Y`, `M`, `D`, or \
                                       `W`")
                        .add_offset(cur.byte_pos()))
            }
        }
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn time_compound(date: u64, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let (time, time_cur) = try!(scan_integer(cur));
        let time_len = cur.slice_between(time_cur).unwrap().len();

        if time_len != "HHMMSS".len() {
            return Err(ScanError::syntax("expected time in `hhmmss` format")
                           .add_offset(cur.byte_pos()));
        }

        let years = date / 1_00_00;
        let months = (date / 1_00) % 1_00;
        let days = date % 1_00;

        if months > 12 {
            return Err(ScanError::syntax("months cannot exceed 12 in this format"));
        }

        if days > 31 {
            return Err(ScanError::syntax("days cannot exceed 31 in this format"));
        }

        let hours = time / 1_00_00;
        let mins = (time / 1_00) % 1_00;
        let secs = time % 1_00;

        if hours > 24 {
            return Err(ScanError::syntax("hours cannot exceed 24 in this format"));
        }

        if mins > 60 {
            return Err(ScanError::syntax("minutes cannot exceed 60 in this format"));
        }

        if secs > 61 {
            return Err(ScanError::syntax("days cannot exceed 61 in this format"));
        }

        let years_dur = try!(dur_years(years, 0.0));
        let months_dur = try!(dur_months(months, 0.0));
        let days_dur = try!(dur_days(days, 0.0));
        let hours_dur = try!(dur_hours(hours, 0.0));
        let mins_dur = try!(dur_mins(mins, 0.0));
        let secs_dur = try!(dur_secs(secs, 0.0));

        checked_add_dur(years_dur, months_dur)
            .and_then(|lhs| checked_add_dur(lhs, days_dur))
            .and_then(|lhs| checked_add_dur(lhs, hours_dur))
            .and_then(|lhs| checked_add_dur(lhs, mins_dur))
            .and_then(|lhs| checked_add_dur(lhs, secs_dur))
            .map(|dur| (dur, time_cur))
            .ok_or_else(|| ScanError::other(MsgErr("overflow in duration")))
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn date_split_month(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let (months, months_cur) = try!(scan_integer(cur));
        let months_len = cur.slice_between(months_cur).unwrap().len();

        if months_len != "MM".len() {
            return Err(ScanError::syntax("expected month in `YYYY-MM-DD` format")
                           .add_offset(cur.byte_pos()));
        }

        match months_cur.next_cp() {
            Some(('-', cur)) => date_split_day(dur + try!(dur_months(months, 0.0)), cur),
            _ => {
                Err(ScanError::syntax("expected `-` after month in `YYYY-MM-DD` format")
                        .add_offset(cur.byte_pos()))
            }
        }
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn date_split_day(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let (days, days_cur) = try!(scan_integer(cur));
        let days_len = cur.slice_between(days_cur).unwrap().len();

        if days_len != "DD".len() {
            return Err(ScanError::syntax("expected day in `YYYY-MM-DD` format")
                           .add_offset(cur.byte_pos()));
        }

        match days_cur.next_cp() {
            Some(('T', cur)) => date_split_hour(dur + try!(dur_days(days, 0.0)), cur),
            _ => Err(ScanError::syntax("expected `T` following date").add_offset(cur.byte_pos())),
        }
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn date_split_hour(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let (hours, hours_cur) = try!(scan_integer(cur));
        let hours_len = cur.slice_between(hours_cur).unwrap().len();

        if hours_len != "hh".len() {
            return Err(ScanError::syntax("expected time in `hh:mm:ss` format")
                           .add_offset(cur.byte_pos()));
        }

        match hours_cur.next_cp() {
            Some((':', cur)) => date_split_min(dur + try!(dur_hours(hours, 0.0)), cur),
            _ => {
                Err(ScanError::syntax("expected time in `hh:mm:ss` format")
                        .add_offset(cur.byte_pos()))
            }
        }
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn date_split_min(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let (mins, mins_cur) = try!(scan_integer(cur));
        let mins_len = cur.slice_between(mins_cur).unwrap().len();

        if mins_len != "mm".len() {
            return Err(ScanError::syntax("expected time in `hh:mm:ss` format")
                           .add_offset(cur.byte_pos()));
        }

        match mins_cur.next_cp() {
            Some((':', cur)) => date_split_sec(dur + try!(dur_mins(mins, 0.0)), cur),
            _ => {
                Err(ScanError::syntax("expected time in `hh:mm:ss` format")
                        .add_offset(cur.byte_pos()))
            }
        }
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn date_split_sec(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let (secs, secs_cur) = try!(scan_integer(cur));
        let secs_len = cur.slice_between(secs_cur).unwrap().len();

        if secs_len != "ss".len() {
            return Err(ScanError::syntax("expected time in `hh:mm:ss` format")
                           .add_offset(cur.byte_pos()));
        }

        Ok((dur + try!(dur_secs(secs, 0.0)), secs_cur))
    }

    macro_rules! add_dur {
        ($a:expr, $b:expr) => {
            try!(checked_add_dur($a, try!($b))
                .ok_or_else(|| ScanError::other(MsgErr("duration overflowed"))))
        };
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn given_year(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        match cur.next_cp() {
            Some(('0'...'9', _)) => (),
            Some(('T', cur)) => return given_date(dur, cur),
            _ => return Ok((dur, cur)),
        }

        let ((int, frac), cur) = try!(scan_real(cur));
        match cur.next_cp() {
            Some(('M', cur)) => given_month(add_dur!(dur, dur_months(int, frac)), cur),
            Some(('D', cur)) => given_day(add_dur!(dur, dur_days(int, frac)), cur),
            _ => {
                Err(ScanError::syntax("expected number followed by one of `M`, or `D`")
                        .add_offset(cur.byte_pos()))
            }
        }
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn given_month(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        match cur.next_cp() {
            Some(('0'...'9', _)) => (),
            Some(('T', cur)) => return given_date(dur, cur),
            _ => return Ok((dur, cur)),
        }

        let ((int, frac), cur) = try!(scan_real(cur));
        match cur.next_cp() {
            Some(('D', cur)) => given_day(add_dur!(dur, dur_days(int, frac)), cur),
            _ => {
                Err(ScanError::syntax("expected number followed by `D`").add_offset(cur.byte_pos()))
            }
        }
    }

    #[cfg(feature="duration-iso8601-dates")]
    fn given_day(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        match cur.next_cp() {
            Some(('T', cur)) => given_date(dur, cur),
            _ => Ok((dur, cur)),
        }
    }

    fn given_date(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        let ((int, frac), cur) = try!(scan_real(cur));
        match cur.next_cp() {
            Some(('H', cur)) => given_hour(add_dur!(dur, dur_hours(int, frac)), cur),
            Some(('M', cur)) => given_min(add_dur!(dur, dur_mins(int, frac)), cur),
            Some(('S', cur)) => given_sec(add_dur!(dur, dur_secs(int, frac)), cur),
            _ => {
                Err(ScanError::syntax("expected number followed by one of `H`, `M`, or `S`")
                        .add_offset(cur.byte_pos()))
            }
        }
    }

    fn given_hour(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        match cur.next_cp() {
            Some(('0'...'9', _)) => (),
            _ => return Ok((dur, cur)),
        }

        let ((int, frac), cur) = try!(scan_real(cur));
        match cur.next_cp() {
            Some(('M', cur)) => given_min(add_dur!(dur, dur_mins(int, frac)), cur),
            Some(('S', cur)) => given_sec(add_dur!(dur, dur_secs(int, frac)), cur),
            _ => {
                Err(ScanError::syntax("expected number followed by one of `M`, or `S`")
                        .add_offset(cur.byte_pos()))
            }
        }
    }

    fn given_min(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        match cur.next_cp() {
            Some(('0'...'9', _)) => (),
            _ => return Ok((dur, cur)),
        }

        let ((int, frac), cur) = try!(scan_real(cur));
        match cur.next_cp() {
            Some(('S', cur)) => given_sec(add_dur!(dur, dur_secs(int, frac)), cur),
            _ => {
                Err(ScanError::syntax("expected number followed by `S`").add_offset(cur.byte_pos()))
            }
        }
    }

    fn given_sec(dur: Duration, cur: StrCursor) -> ScanResult<Duration, StrCursor> {
        Ok((dur, cur))
    }
}

fn checked_add_dur(a: Duration, b: Duration) -> Option<Duration> {
    let (a_s, a_ns) = (a.as_secs(), a.subsec_nanos());
    let (b_s, b_ns) = (b.as_secs(), b.subsec_nanos());
    let c_ns = a_ns + b_ns;
    let (c_ns, c_carry) = match c_ns {
        c_ns if c_ns > NANOS_IN_SEC => (c_ns - NANOS_IN_SEC, 1),
        c_ns => (c_ns, 0),
    };
    a_s.checked_add(b_s)
       .and_then(|c_s| c_s.checked_add(c_carry))
       .map(|c_s| Duration::new(c_s, c_ns))
}

macro_rules! dur_conv {
    (
        $($(#[$attrs:meta])* fn $fn_name:ident($name:expr, $scale:expr);)*
    ) => {
        $(
            $(#[$attrs])*
            fn $fn_name(int: u64, frac: f64) -> Result<Duration, ScanError> {
                const MSG: &'static str = concat!("overflow converting ",
                    $name, " into seconds");
                assert!(0.0f64 <= frac && frac < 1.0f64);
                let secs = try!(int.checked_mul($scale)
                    .ok_or_else(|| ScanError::other(MsgErr(MSG))));
                
                let nanos = frac * ($scale as f64);
                let secs = try!(secs.checked_add(nanos as u64)
                    .ok_or_else(|| ScanError::other(MsgErr(MSG))));
                let nanos = (nanos.fract() * (NANOS_IN_SEC as f64)) as u32;

                Ok(Duration::new(secs, nanos))
            }
        )*
    };
}

dur_conv! {
    #[cfg(feature="duration-iso8601-dates")] fn dur_years("years", SECS_IN_YEAR);
    #[cfg(feature="duration-iso8601-dates")] fn dur_months("months", SECS_IN_MONTH);
    #[cfg(feature="duration-iso8601-dates")] fn dur_weeks("weeks", SECS_IN_WEEK);
    #[cfg(feature="duration-iso8601-dates")] fn dur_days("days", SECS_IN_DAY);

    fn dur_hours("hours", SECS_IN_HOUR);
    fn dur_mins("mins", SECS_IN_MIN);
    fn dur_secs("secs", SECS_IN_SEC);
}

fn scan_integer(cur: StrCursor) -> ScanResult<u64, StrCursor> {
    let start = cur;
    let mut cur = match cur.next_cp() {
        Some(('0'...'9', cur)) => cur,
        _ => return Err(ScanError::syntax("expected digit").add_offset(cur.byte_pos())),
    };

    loop {
        cur = match cur.next_cp() {
            Some(('0'...'9', cur)) => cur,
            _ => {
                let v = try!(start.slice_between(cur)
                                  .unwrap()
                                  .parse()
                                  .map_err(|e| ScanError::int(e).add_offset(cur.byte_pos())));
                return Ok((v, cur));
            }
        }
    }
}

// NOTE**: This is pretty horrible.  The issue is that because `,` is a valid decimal point, we can't just use `f64::from_str`.  One possibility would be to throw the string into a stack array, mutate it, *then* pass it on... but that means *yet another dependency*.  I'm not sure it's worth it for the moderate horribleness of the following code.
//
// So yes, I know this sucks, but it's *calculated suckage*.
//
// Also, it would be nice if this could accurately parse (say) nanoseconds as fractional years... but that would again require us to forward to `f64::from_str` for the fractional part.  That's why this function returns `(u64, f64)`; it's essentially that way on the hope that one day it'll actually be able to *use* that precision.  :P
//
fn scan_real(cur: StrCursor) -> ScanResult<(u64, f64), StrCursor> {
    let (int, cur) = try!(scan_integer(cur));
    let cur = match cur.next_cp() {
        Some(('.', cur)) | Some((',', cur)) => cur,
        _ => return Ok(((int, 0.0), cur)),
    };
    scan_real_frac(int, cur)
}

fn scan_real_frac(int: u64, cur: StrCursor) -> ScanResult<(u64, f64), StrCursor> {
    let (frac, frac_cur) = try!(scan_integer(cur));
    let frac_len = cur.slice_between(frac_cur).unwrap().len();
    let frac = frac as f64;
    let frac = frac / (10.0f64).powf(frac_len as f64);
    Ok(((int, frac), frac_cur))
}
