#![allow(unused_crate_dependencies, clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use db_handler::constants::{
    DEFAULT_PAGE_SIZE, HEALTH_TIMEOUT_MS, MAX_PAGE_NUMBER, MAX_PAGE_SIZE,
    MAX_PARAMS_PER_QUERY, MAX_QUERY_LENGTH, MAX_RESULT_ROWS, MAX_STRING_FIELD_LENGTH,
};


const _: u64 = DEFAULT_PAGE_SIZE;
const _: u64 = MAX_PAGE_SIZE;
const _: u64 = MAX_PAGE_NUMBER;
const _: u64 = HEALTH_TIMEOUT_MS;
const _: usize = MAX_QUERY_LENGTH;
const _: usize = MAX_PARAMS_PER_QUERY;
const _: usize = MAX_RESULT_ROWS;
const _: usize = MAX_STRING_FIELD_LENGTH;


const _: () = assert!(DEFAULT_PAGE_SIZE > 0);
const _: () = assert!(MAX_PAGE_SIZE > 0);
const _: () = assert!(MAX_PAGE_NUMBER > 0);
const _: () = assert!(MAX_QUERY_LENGTH > 0);
const _: () = assert!(MAX_PARAMS_PER_QUERY > 0);
const _: () = assert!(MAX_RESULT_ROWS > 0);
const _: () = assert!(MAX_STRING_FIELD_LENGTH > 0);
const _: () = assert!(HEALTH_TIMEOUT_MS > 0);

const _: () = assert!(DEFAULT_PAGE_SIZE <= MAX_PAGE_SIZE);
const _: () = assert!(MAX_PAGE_SIZE < MAX_PAGE_NUMBER);
const _: () = assert!(MAX_PARAMS_PER_QUERY < MAX_QUERY_LENGTH);
const _: () = assert!((MAX_RESULT_ROWS as u64) < MAX_PAGE_NUMBER);
const _: () = assert!(MAX_QUERY_LENGTH > MAX_STRING_FIELD_LENGTH);


#[test]
fn default_page_size_is_20() {
    assert_eq!(DEFAULT_PAGE_SIZE, 20);
}

#[test]
fn max_page_size_is_100() {
    assert_eq!(MAX_PAGE_SIZE, 100);
}

#[test]
fn max_page_number_is_one_million() {
    assert_eq!(MAX_PAGE_NUMBER, 1_000_000);
}

#[test]
fn max_query_length_is_100k() {
    assert_eq!(MAX_QUERY_LENGTH, 100_000);
}

#[test]
fn max_params_per_query_is_100() {
    assert_eq!(MAX_PARAMS_PER_QUERY, 100);
}

#[test]
fn max_result_rows_is_10k() {
    assert_eq!(MAX_RESULT_ROWS, 10_000);
}

#[test]
fn max_string_field_length_is_65535() {
    assert_eq!(MAX_STRING_FIELD_LENGTH, 65_535);
}

#[test]
fn health_timeout_ms_is_2000() {
    assert_eq!(HEALTH_TIMEOUT_MS, 2_000);
}


#[test]
fn max_string_field_length_equals_u16_max() {
    assert_eq!(MAX_STRING_FIELD_LENGTH, u16::MAX as usize);
}

#[test]
fn max_string_field_length_fits_in_u16() {
    assert!(u16::try_from(MAX_STRING_FIELD_LENGTH).is_ok());
}

#[test]
fn max_string_field_length_plus_one_overflows_u16() {
    assert!(u16::try_from(MAX_STRING_FIELD_LENGTH + 1).is_err());
}

#[test]
fn max_string_field_length_is_64_kib_minus_one() {
    assert_eq!(MAX_STRING_FIELD_LENGTH, 64 * 1024 - 1);
}


#[test]
fn default_page_size_strictly_less_than_max() {
    assert!(DEFAULT_PAGE_SIZE < MAX_PAGE_SIZE);
}

#[test]
fn default_page_size_is_reasonable() {
    assert!(DEFAULT_PAGE_SIZE >= 10);
    assert!(DEFAULT_PAGE_SIZE <= 50);
}

#[test]
fn max_page_size_is_reasonable_upper_bound() {
    assert!(MAX_PAGE_SIZE <= 1_000);
}

#[test]
fn max_page_size_less_than_max_page_number() {
    assert!(MAX_PAGE_SIZE < MAX_PAGE_NUMBER);
}

#[test]
fn default_page_size_divides_evenly() {
    assert_eq!(DEFAULT_PAGE_SIZE % 5, 0);
    assert_eq!(DEFAULT_PAGE_SIZE % 10, 0);
}


#[test]
fn health_timeout_constructs_duration() {
    let d = Duration::from_millis(HEALTH_TIMEOUT_MS);
    assert_eq!(d.as_millis(), u128::from(HEALTH_TIMEOUT_MS));
}

#[test]
fn health_timeout_at_least_100ms() {
    assert!(HEALTH_TIMEOUT_MS >= 100);
}

#[test]
fn health_timeout_at_most_5s() {
    assert!(HEALTH_TIMEOUT_MS <= 5_000);
}

#[test]
fn health_timeout_whole_seconds() {
    assert_eq!(HEALTH_TIMEOUT_MS % 1_000, 0);
}


#[test]
fn max_page_size_times_max_page_number_no_overflow_u64() {
    assert!(MAX_PAGE_SIZE.checked_mul(MAX_PAGE_NUMBER).is_some());
}

#[test]
fn default_page_size_times_max_page_number_no_overflow_u64() {
    assert!(DEFAULT_PAGE_SIZE.checked_mul(MAX_PAGE_NUMBER).is_some());
}

#[test]
fn max_offset_with_max_per_page_no_overflow_u64() {
    let max_offset = (MAX_PAGE_NUMBER - 1).checked_mul(MAX_PAGE_SIZE);
    assert!(max_offset.is_some());
}

#[test]
fn max_offset_is_under_200_million() {
    let max_offset = (MAX_PAGE_NUMBER - 1).saturating_mul(MAX_PAGE_SIZE);
    assert!(max_offset <= 200_000_000);
}

#[test]
fn max_addressable_rows_is_at_least_10_million() {
    let total = MAX_PAGE_SIZE.saturating_mul(MAX_PAGE_NUMBER);
    assert!(total >= 10_000_000, "max addressable rows = {total}");
}


#[test]
fn max_page_size_fits_u8() {
    assert!(u8::try_from(MAX_PAGE_SIZE).is_ok());
}

#[test]
fn max_params_per_query_fits_u8() {
    assert!(u8::try_from(MAX_PARAMS_PER_QUERY).is_ok());
}

#[test]
fn max_params_per_query_fits_u16() {
    assert!(u16::try_from(MAX_PARAMS_PER_QUERY).is_ok());
}

#[test]
fn max_query_length_fits_u32() {
    assert!(u32::try_from(MAX_QUERY_LENGTH).is_ok());
}

#[test]
fn max_result_rows_fits_u32() {
    assert!(u32::try_from(MAX_RESULT_ROWS).is_ok());
}

#[test]
fn max_page_number_fits_u32() {
    assert!(u32::try_from(MAX_PAGE_NUMBER).is_ok());
}


#[test]
fn max_query_length_greater_than_string_field_length() {
    assert!(MAX_QUERY_LENGTH > MAX_STRING_FIELD_LENGTH);
}

#[test]
fn max_result_rows_less_than_max_offset() {
    let max_offset = (MAX_PAGE_NUMBER - 1).saturating_mul(MAX_PAGE_SIZE);
    assert!((MAX_RESULT_ROWS as u64) < max_offset);
}

#[test]
fn default_page_size_less_than_max_params() {
    assert!(DEFAULT_PAGE_SIZE < MAX_PARAMS_PER_QUERY as u64);
}

#[test]
fn default_page_size_less_than_max_result_rows() {
    assert!((DEFAULT_PAGE_SIZE as usize) < MAX_RESULT_ROWS);
}


#[test]
fn default_page_size_differs_from_max_page_size() {
    assert_ne!(DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE);
}


#[test]
fn constants_usable_in_const_context() {
    const PAGE: u64 = MAX_PAGE_SIZE;
    const TIMEOUT: u64 = HEALTH_TIMEOUT_MS;
    const QUERY: usize = MAX_QUERY_LENGTH;
    const FIELD: usize = MAX_STRING_FIELD_LENGTH;
    const ROWS: usize = MAX_RESULT_ROWS;
    const PARAMS: usize = MAX_PARAMS_PER_QUERY;
    const DEFAULT: u64 = DEFAULT_PAGE_SIZE;
    const PAGES: u64 = MAX_PAGE_NUMBER;

    assert_eq!(PAGE, MAX_PAGE_SIZE);
    assert_eq!(TIMEOUT, HEALTH_TIMEOUT_MS);
    assert_eq!(QUERY, MAX_QUERY_LENGTH);
    assert_eq!(FIELD, MAX_STRING_FIELD_LENGTH);
    assert_eq!(ROWS, MAX_RESULT_ROWS);
    assert_eq!(PARAMS, MAX_PARAMS_PER_QUERY);
    assert_eq!(DEFAULT, DEFAULT_PAGE_SIZE);
    assert_eq!(PAGES, MAX_PAGE_NUMBER);
}


#[test]
fn all_constants_strictly_positive() {
    assert!(DEFAULT_PAGE_SIZE > 0);
    assert!(MAX_PAGE_SIZE > 0);
    assert!(MAX_PAGE_NUMBER > 0);
    assert!(MAX_QUERY_LENGTH > 0);
    assert!(MAX_PARAMS_PER_QUERY > 0);
    assert!(MAX_RESULT_ROWS > 0);
    assert!(MAX_STRING_FIELD_LENGTH > 0);
    assert!(HEALTH_TIMEOUT_MS > 0);
}

#[test]
fn all_ordering_invariants_hold() {
    assert!(DEFAULT_PAGE_SIZE < MAX_PAGE_SIZE);
    assert!(MAX_PAGE_SIZE < MAX_PAGE_NUMBER);
    assert!(MAX_PARAMS_PER_QUERY < MAX_QUERY_LENGTH);
    assert!((MAX_RESULT_ROWS as u64) < MAX_PAGE_NUMBER);
    assert!(MAX_QUERY_LENGTH > MAX_STRING_FIELD_LENGTH);
    assert!((DEFAULT_PAGE_SIZE as usize) < MAX_RESULT_ROWS);
}
