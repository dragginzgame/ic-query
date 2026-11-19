use super::*;
use ic_platform::certificate::PreparedRootKey;
use std::{cell::Cell, future::ready};

fn id() -> Principal {
    Principal::from_slice(&[1])
}

fn observation(time: u64) -> StateTimeObservation {
    StateTimeObservation {
        subnet_id: id(),
        certified_time_nanoseconds: time,
        delegation_time_nanoseconds: None,
        observed_at_unix_nanoseconds: 0,
        endpoint: mainnet::ENDPOINT,
        source_path: format!("/api/v3/subnet/{}/read_state", id()),
        state_tree_path: "/time",
        maximum_age_nanoseconds: MAX_AGE_NS,
        maximum_future_skew_nanoseconds: FUTURE_SKEW_NS,
        state_certificate_verified: true,
        original_certificate_bytes_retained: false,
    }
}

#[tokio::test(start_paused = true)]
async fn progress_order_extremes_and_clock_reversal() {
    for (a, b, expected) in [
        (
            0,
            u64::MAX,
            StateTimeChange::Advanced {
                by_nanoseconds: u64::MAX,
            },
        ),
        (7, 7, StateTimeChange::Unchanged),
        (8, 3, StateTimeChange::Regressed),
    ] {
        let calls = Cell::new(0);
        let start = Instant::now();
        let report = sample(id(), 60, || {
            let n = calls.get();
            calls.set(n + 1);
            if n == 1 {
                assert_eq!(start.elapsed(), Duration::from_secs(60));
            }
            let mut value = observation(if n == 0 { a } else { b });
            value.observed_at_unix_nanoseconds = if n == 0 { 100 } else { 1 };

            ready(Ok(value))
        })
        .await
        .unwrap();
        assert!(report.collection_complete);
        assert_eq!(calls.get(), 2);
        assert!(matches!(&report.after, LaterObservation::Read{change,..} if *change==expected));
        let json = serde_json::to_value(report).unwrap();
        assert_eq!(json["before"]["certified_time_nanoseconds"], a);
    }
}

#[tokio::test(start_paused = true)]
async fn progress_failures_and_identity_admission() {
    let calls = Cell::new(0);
    for interval in [0, 61] {
        assert!(matches!(
            sample(id(), interval, || {
                calls.set(99);
                ready(Ok(observation(1)))
            })
            .await,
            Err(ProgressError::InvalidInterval)
        ));
    }
    assert_eq!(calls.get(), 0);
    assert!(
        sample(id(), 1, || {
            calls.set(calls.get() + 1);
            ready(Err(StateTimeError::Read(ReadError::Deadline)))
        })
        .await
        .is_err()
    );
    assert_eq!(calls.get(), 1);
    for wrong_identity in [false, true] {
        calls.set(0);
        let report = sample(id(), 1, || {
            let n = calls.get();
            calls.set(n + 1);

            ready(if n == 0 {
                Ok(observation(1))
            } else if wrong_identity {
                let mut v = observation(2);
                v.subnet_id = Principal::anonymous();

                Ok(v)
            } else {
                Err(StateTimeError::Read(ReadError::Deadline))
            })
        })
        .await
        .unwrap();
        assert!(!report.collection_complete);
        assert_eq!(report.before.certified_time_nanoseconds, 1);
        let json = serde_json::to_value(report).unwrap();
        assert_eq!(
            json["after"]["error"],
            if wrong_identity {
                "source_identity"
            } else {
                "read_deadline"
            }
        );
    }
    let mut unverified = observation(1);
    unverified.state_certificate_verified = false;
    assert!(admit(unverified, id()).is_err());
}

#[tokio::test(start_paused = true)]
async fn progress_cancellation_starts_no_later_work() {
    for stop in [5, 15, 25] {
        let calls = Cell::new(0);
        let work = sample(id(), 10, || {
            calls.set(calls.get() + 1);

            async {
                tokio::time::sleep(Duration::from_secs(10)).await;

                Ok(observation(1))
            }
        });
        assert!(
            tokio::time::timeout(Duration::from_secs(stop), work)
                .await
                .is_err()
        );
        let expected = if stop < 20 { 1 } else { 2 };
        assert_eq!(calls.get(), expected);
        tokio::time::sleep(Duration::from_secs(100)).await;
        assert_eq!(calls.get(), expected);
    }
}

#[test]
fn local_supplier_accepts_agent_certificate_identity_and_rejects_wrong_authority() {
    use ic_platform::certificate::{CertificateDecodeLimits, decode_certificate};
    let root = PreparedRootKey::from_der(include_bytes!("fixtures/root.der")).unwrap();
    for (bytes, time, subnet, root_binding) in [
        (
            include_bytes!("fixtures/direct.cbor").as_slice(),
            1_620_328_630_000_000_005,
            "aaaaa-aa",
            true,
        ),
        (
            include_bytes!("fixtures/delegated.cbor").as_slice(),
            1_620_328_630_000_000_008,
            "sgier-p6fyz-ejpwm-jghgc-4ypgv-6ft53-5hns6-ojhae-wvlic-ifcah-eqe",
            false,
        ),
    ] {
        let decoded =
            decode_certificate(bytes, CertificateDecodeLimits { bytes: 1024 * 1024 }).unwrap();
        let certificate = decoded.unverified();
        let subnet = subnet.parse().unwrap();
        let binding = root_binding.then_some(subnet);
        let window = TimeWindow {
            reference_ns: time,
            max_age_ns: 0,
            max_future_skew_ns: 0,
        };
        let policy = TimePolicy {
            certificate: window,
            delegation: window,
        };
        let evidence = authenticate_subnet_time_from_certificate(
            certificate,
            &root,
            subnet,
            binding,
            policy,
            LIMITS,
        )
        .unwrap();
        assert_eq!(evidence.time_ns(), time);
        assert!(evidence.original_bytes().is_none());
        assert!(std::ptr::eq(evidence.certificate().unwrap(), certificate));
        let wrong = "2vxsx-fae".parse().unwrap();
        assert_eq!(
            authenticate_subnet_time_from_certificate(
                certificate,
                &root,
                wrong,
                binding,
                policy,
                LIMITS
            )
            .unwrap_err(),
            VerifyError::SubnetMismatch
        );
        let mut broken = certificate.clone();
        broken.signature[0] ^= 1;
        assert!(
            authenticate_subnet_time_from_certificate(
                &broken, &root, subnet, binding, policy, LIMITS
            )
            .is_err()
        );
    }
}
