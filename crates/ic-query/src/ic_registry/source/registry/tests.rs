use super::*;

#[test]
fn certificate_time_must_be_close_to_the_caller_observation_time() {
    let request = MainnetRegistryFetchRequest {
        endpoint: "https://icp-api.io".to_string(),
        fetched_at: "2026-08-05T12:00:00Z".to_string(),
        fetched_by: "test".to_string(),
    };
    let fetched_at = parse_utc_timestamp_secs(&request.fetched_at).expect("timestamp");

    for certificate_time in [
        fetched_at - crate::certification::MAX_CERTIFICATE_TIME_SKEW_SECONDS,
        fetched_at,
        fetched_at + crate::certification::MAX_CERTIFICATE_TIME_SKEW_SECONDS,
    ] {
        validate_live_certificate_time(&request, certificate_time * 1_000_000_000)
            .expect("accepted skew boundary");
    }

    let error = validate_live_certificate_time(
        &request,
        (fetched_at + crate::certification::MAX_CERTIFICATE_TIME_SKEW_SECONDS + 1) * 1_000_000_000,
    )
    .expect_err("future certificate outside skew");
    assert!(matches!(
        error,
        RegistryFetchError::InvalidCertifiedRegistry { reason }
            if reason.contains("outside the accepted")
    ));
}
