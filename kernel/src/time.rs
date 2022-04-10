use crate::{
    asm::{inb, outb, without_interrupts},
    log,
};
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};

const NANOSECONDS_PER_TICK: u64 = 838_096;
const OFFSET_1985: u64 = ((24 * (21 + 30 + 31) + 15) * 60 + 5) * 60;
static mut TIME: (AtomicU64, AtomicU64) = (AtomicU64::new(0), AtomicU64::new(0));

pub fn cmos_time() -> Timestamp {
    let disable_nmi = true;
    // Status Register B, Bit 1 (value = 2): Enables 24 hour format if set
    // Status Register B, Bit 2 (value = 4): Enables Binary mode if set
    let status = read_cmos_register(0x0b, disable_nmi);
    let from_bcd = if status & 4 > 0 { |x| x } else { |x| 10u8 * ((x & 0xf0) >> 4) + (x & 0x0f) };
    // 0x32      Century (maybe)     19–20
    let century = from_bcd(read_cmos_register(0x32, disable_nmi)) as u16;
    // 0x09      Year                0–99
    let year_mod_100 = from_bcd(read_cmos_register(0x09, disable_nmi)) as u16;
    // 0x08      Month               1–12
    let month = Month(from_bcd(read_cmos_register(0x08, disable_nmi)));
    // 0x06      Weekday             1–7, Sunday = 1
    let weekday = Weekday(from_bcd(read_cmos_register(0x06, disable_nmi)) % 7);
    // 0x07      Day of Month        1–31
    let day = Ordinal(from_bcd(read_cmos_register(0x07, disable_nmi)));
    // 0x04      Hours               0–23 in 24-hour mode,
    //                               1–12 in 12-hour mode, highest bit set if pm
    let hours_raw = from_bcd(read_cmos_register(0x04, disable_nmi));
    let hours = Hours(hours_raw & 0x7f + if (hours_raw & 0x80 > 0) && (status & 2 > 0) { 12 } else { 0 });
    // 0x02      Minutes             0–59
    let minutes = Minutes(from_bcd(read_cmos_register(0x02, disable_nmi)));
    // 0x00      Seconds             0–59
    let seconds = Seconds(from_bcd(read_cmos_register(0x00, disable_nmi)));
    let year = Year(if century == 19 || century == 20 { 100 * century + year_mod_100 } else { 2000 + year_mod_100 });
    Timestamp::from_cmos_data(weekday, day, month, year, hours, minutes, seconds)
}

fn read_cmos_register(register: u8, disable_nmi: bool) -> u8 {
    unsafe {
        outb(0x70, ((disable_nmi as u8) << 7) | register);
        inb(0x71)
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct Timestamp {
    year: Year,
    month: Month,
    day: Ordinal,
    hours: Hours,
    minutes: Minutes,
    seconds: Seconds,
}

impl Timestamp {
    fn from_cmos_data(
        weekday: Weekday,
        day: Ordinal,
        month: Month,
        year: Year,
        hours: Hours,
        minutes: Minutes,
        seconds: Seconds,
    ) -> Self {
        let ts = Self { day, month, year, hours, minutes, seconds };
        assert!(ts.weekday() == weekday);
        ts
    }
    fn from_raw(year: u16, month: u8, day: u8, hours: u8, minutes: u8, seconds: u8) -> Self {
        Self {
            year: Year(year),
            month: Month(month),
            day: Ordinal(day),
            hours: Hours(hours),
            minutes: Minutes(minutes),
            seconds: Seconds(seconds),
        }
    }

    pub fn seconds_since_epoch(&self) -> u64 {
        assert!(self.year.0 > 1984);
        let leap_years: u64 = self.year.leap_years_since_epoch();
        let this_year = OFFSET_1985 + ((self.year.0 as u64 - 1985) * 365 + leap_years) * 24 * 3600;
        log!("{}", this_year);
        let mut seconds: u64 = 0;
        for month in 1..self.month.0 {
            seconds += Month(month).days(&self.year) as u64 * 24 * 3600;
        }
        seconds += (self.day.0 as u64 - 1) * 24 * 3600;
        seconds += (self.hours.0 as u64 * 60 + self.minutes.0 as u64) * 60 + self.seconds.0 as u64;
        this_year + seconds
    }

    pub fn weekday(&self) -> Weekday {
        let mut day = self.day.0 as u16;
        let mut year = self.year.0;
        let month = self.month.0 as u16;
        day += if month < 3 {
            year -= 1;
            year + 1
        } else {
            year - 2
        };
        let weekday = (((((23 * month) / 9) + day + 4 + (year / 4)) - (year / 100) + (year / 400)) + 1) % 7;
        Weekday(weekday as u8)
    }

    pub fn from_epoch(epoch: u64) -> Self {
        assert!(epoch >= OFFSET_1985);
        let mut seconds = epoch - OFFSET_1985;
        let mut days = seconds / (24 * 3600);
        let approx_years = days / 365;
        let mut year = 1985 + approx_years as u16;
        let leap_years = Year(year).leap_years_since_epoch();
        let actual_days = approx_years * 365 + leap_years;
        if actual_days > days {
            year -= 1;
        }
        let leap_years = Year(year).leap_years_since_epoch();
        let actual_days = approx_years * 365 + leap_years;
        days -= actual_days;
        let mut month = 1;
        while days > Month(month).days(&Year(year)) as u64 {
            days -= Month(month).days(&Year(year)) as u64;
            month += 1;
        }
        let day = (days + 1) as u8;
        seconds %= 24 * 3600;
        let hours = seconds / 3600;
        seconds %= 3600;
        let minutes = seconds / 60;
        seconds %= 60;
        Self::from_raw(year, month, day, hours as u8, minutes as u8, seconds as u8)
    }

    pub fn now() -> Self {
        Self::from_epoch(unsafe { TIME.0.load(Ordering::Relaxed) })
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}, the {} of {} {}, {:02}:{:02}:{:02}",
            self.weekday(),
            self.day,
            self.month,
            self.year,
            self.hours.0,
            self.minutes.0,
            self.seconds.0
        )
    }
}

#[derive(PartialEq, Eq)]
pub struct Weekday(u8);

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let day = match self.0 % 7 {
            0 => "Saturday",
            1 => "Sunday",
            2 => "Monday",
            3 => "Tuesday",
            4 => "Wednesday",
            5 => "Thursday",
            6 => "Friday",
            _ => unreachable!(),
        };
        write!(f, "{}", day)
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct Month(u8);

impl Month {
    fn days(&self, year: &Year) -> u8 {
        match self.0 {
            1 => 31,
            2 => {
                if year.is_leap_year() {
                    29
                } else {
                    28
                }
            }
            3 => 31,
            4 => 30,
            5 => 31,
            6 => 30,
            7 => 31,
            8 => 31,
            9 => 30,
            10 => 31,
            11 => 30,
            12 => 31,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let mth = match self.0 {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => unreachable!(),
        };
        write!(f, "{}", mth)
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct Ordinal(u8);

impl fmt::Display for Ordinal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let suffix = match self.0 % 10 {
            0 => "th",
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        };
        write!(f, "{}{}", self.0, suffix)
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct Year(u16);

impl Year {
    fn is_leap_year(&self) -> bool {
        (self.0 % 4 == 0) && (self.0 % 100 != 0 || self.0 % 400 == 0)
    }
    fn days(&self) -> u16 {
        if self.is_leap_year() {
            366
        } else {
            365
        }
    }
    fn leap_years_since_epoch(&self) -> u64 {
        (1985..self.0).fold(0, |acc, x| if Year(x).is_leap_year() { acc + 1 } else { acc })
    }
}

impl fmt::Display for Year {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct Hours(u8);

impl fmt::Display for Hours {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct Minutes(u8);

impl fmt::Display for Minutes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct Seconds(u8);

impl fmt::Display for Seconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

pub unsafe fn tick() {
    without_interrupts(|| {
        let ns = TIME.1.load(Ordering::SeqCst) + NANOSECONDS_PER_TICK;
        TIME.1.store(ns % 1_000_000_000, Ordering::SeqCst);
        TIME.0.fetch_add(ns / 1_000_000_000, Ordering::SeqCst);
    });
}

pub fn init() {
    let ts = cmos_time();
    let seconds = ts.seconds_since_epoch();
    unsafe { TIME.0.store(seconds, Ordering::Relaxed) };
    log!("System time set to {} ({})...", ts, seconds);
}

#[test_case]
fn epoch_value_in_1985_is_correct() {
    let ts = Timestamp::from_raw(1985, 1, 1, 0, 0, 0);
    assert!(ts.seconds_since_epoch() == OFFSET_1985);
}

#[test_case]
fn begins_on_wednesday() {
    let ts = Timestamp::from_raw(1984, 10, 10, 8, 55, 0);
    assert!(ts.weekday().0 == 4);
}
