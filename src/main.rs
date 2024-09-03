use std::error::Error;
use chrono::{DateTime, Days, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Utc};
use log::info;

use julian::Calendar;

fn julian2datetime(j: f64) -> DateTime<Local> {
    let date = Calendar::GREGORIAN.at_jdn(j.floor() as i32);
    let rem = (j - j.floor()) * 24.0;
    let date = NaiveDate::try_from(date).unwrap();
    let h = rem.floor();
    let rem = (rem - h) * 60.0;
    let m = rem.floor();
    let rem = (rem - m) * 60.0;
    let s = rem.floor();
    let time = NaiveTime::from_hms_opt(h as u32, m as u32, s as u32).unwrap(); 
    let date = NaiveDateTime::new(date, time); 
    let date: DateTime<Utc> = Utc.from_utc_datetime(&date);
    let date: DateTime<Local> = date.into();
    let date = date + TimeDelta::hours(12);
    let date = date.checked_sub_days(Days::new(1)).unwrap();
    date
}

fn mean_solar_time(n: f64, long: f64) -> f64 {
    n - long / 360.0
}

fn solar_mean_anomaly(j_star: f64) -> f64 {
    (357.5291 + 0.98560028 * j_star) % 360.0
}

fn normalized_date(j_date: f64) -> f64 {
    (j_date - 2451545.0 + 0.0008).ceil()
}

fn equation_of_the_center(m: f64) -> f64 {
    let m_rad = m.to_radians();
    1.9148 * m_rad.sin() + 0.02 * (2.0 * m_rad).sin() + 0.0003 * (3.0 * m_rad).sin()
}

fn ecliptic_longitude(m: f64, c: f64) -> f64 {
    (m + c + 180.0 + 102.9372) % 360.0
}

fn declination_of_the_sun(lambda: f64) -> f64 {
    (lambda.to_radians().sin() * (23.4397_f64).to_radians().sin()).asin().to_degrees()
}

fn hour_angle(lat: f64, delta: f64) -> f64 {
    let rlat = lat.to_radians();
    let rdel = delta.to_radians();
    (((-0.833_f64).to_radians().sin() - rlat.sin() * rdel.sin()) / (rlat.cos() * rdel.cos())).acos().to_degrees()
}

fn transit(j_star: f64, m: f64, lambda: f64) -> f64 {
    2451545.0 + j_star + 0.0053 * m.to_radians().sin() - 0.0069 * (2.0 * lambda).to_radians().sin()
}

fn get_sunrise_sunset(lat: f64, long: f64, today: f64) -> (f64, f64) {
    let n = normalized_date(today);
    info!("Normalized date: {}", n);
    let j_star = mean_solar_time(n, long);
    info!("Mean solar time: {}", j_star);
    let m = solar_mean_anomaly(j_star);
    info!("Solar mean anomaly {}", m);
    let c = equation_of_the_center(m);
    info!("Equation of the center: {}", c);
    let lambda = ecliptic_longitude(m, c);
    info!("Ecliptic longitude: {}", lambda);
    let delta = declination_of_the_sun(lambda);
    info!("Declination of the sun: {}", delta);
    let omega_0 = hour_angle(lat, delta);
    info!("Hour angle: {}", omega_0);
    let j_transit = transit(j_star, m, lambda);
    info!("Jtransit: {}", j_transit);
    let j_rise = j_transit - omega_0 / 360.0;
    let j_set = j_transit + omega_0 / 360.0;
    (j_rise, j_set)
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let lat = 48.0 + 21.0 / 60.0 + 19.1 / (60.0_f64).powi(2);
    info!("Lat: {}", lat);
    let long = 9.0 + 54.0 / 60.0 + 21.9 / (60.0_f64).powi(2);
    info!("Long: {}", long);
    let today = Calendar::JULIAN.now()?.0.julian_day_number() as f64;
    info!("Jtoday: {}", today);
    let (rise, set) = get_sunrise_sunset(lat, long, today);
    info!("{}", rise);
    info!("{}", set);
    let rise = julian2datetime(rise);
    let set = julian2datetime(set);
    let len = set - rise;
    println!("Sunrise: {}", rise.to_rfc2822());
    println!("Sunset: {}", set.to_rfc2822());
    println!("Sun length: {}h, {}m, {}s", len.num_hours(), len.num_minutes() - len.num_hours() * 60, len.num_seconds() - len.num_minutes() * 60);
    Ok(())
}

