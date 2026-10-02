use super::*;
use crate::{
    cache_file::write_managed_text_atomically,
    ic_registry::{
        RegistryAcquisition, RegistryFetchError,
        proto::{
            HighCapacityRegistryDelta, HighCapacityRegistryGetChangesSinceResponse,
            HighCapacityRegistryValue, high_capacity_registry_value,
        },
        transport::{
            RegistryQueryCounter, SubnetCatalogProgress, SubnetCatalogProgressPhase,
            key_family::collect_key_family,
        },
    },
    test_support::temp_dir,
};
use prost::Message;
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};
use std::{
    fs,
    future::Future,
    sync::{Arc, Mutex},
    task::Context,
};

const PREFIX: &str = "canister_ranges_";
const ENDPOINT: &str = "https://a.example";

fn counter(root: &Path, endpoint: &str) -> RegistryQueryCounter {
    RegistryQueryCounter::with_acquisition(
        endpoint.to_string(),
        Arc::new(RegistryAcquisition::with_history_cache(
            root.to_path_buf(),
            None,
        )),
    )
}

fn response(
    version: u64,
    mutations: &[(u64, bool)],
) -> HighCapacityRegistryGetChangesSinceResponse {
    HighCapacityRegistryGetChangesSinceResponse {
        error: None,
        version,
        deltas: vec![HighCapacityRegistryDelta {
            key: b"canister_ranges_00".to_vec(),
            values: mutations
                .iter()
                .map(|&(version, present)| HighCapacityRegistryValue {
                    version,
                    timestamp_nanoseconds: version,
                    content: Some(if present {
                        high_capacity_registry_value::Content::Value(
                            b"payload is not retained".to_vec(),
                        )
                    } else {
                        high_capacity_registry_value::Content::DeletionMarker(true)
                    }),
                })
                .collect(),
        }],
    }
}

fn seed(root: &Path) {
    futures::executor::block_on(collect_key_family(
        PREFIX,
        3,
        &counter(root, ENDPOINT),
        |cursor| {
            assert_eq!(cursor, 0);
            std::future::ready(Ok(response(3, &[(1, true), (2, true), (3, false)])))
        },
    ))
    .expect("seed complete history");
}

fn rewrite_document(root: &Path, edit: impl FnOnce(&mut HistoryDocument)) {
    let cache = RegistryHistoryCache::new(root.to_path_buf());
    let mut document: HistoryDocument =
        serde_json::from_slice(&fs::read(&cache.path).expect("checkpoint bytes"))
            .expect("checkpoint JSON");
    edit(&mut document);
    document.digest = history_digest(&document).expect("checksum");
    write_managed_text_atomically(
        root,
        &cache.path,
        &serde_json::to_string(&document).expect("JSON"),
    )
    .expect("rewrite fixture");
}

#[test]
fn interrupted_batched_publication_restores_a_complete_prefix_with_bounded_rework() {
    for (completed_pages, durable_version) in [(3, 1), (9, 8)] {
        let root = temp_dir("ic-query-history-publication-batch");
        let source = counter(&root, ENDPOINT);
        let mut collection = Box::pin(collect_key_family(
            PREFIX,
            10,
            &source,
            |cursor| async move {
                if cursor == completed_pages {
                    std::future::pending().await
                } else {
                    Ok(response(10, &[(cursor + 1, true)]))
                }
            },
        ));
        let waker = futures::task::noop_waker();
        assert!(
            collection
                .as_mut()
                .poll(&mut Context::from_waker(&waker))
                .is_pending()
        );
        drop(collection);
        let cache = RegistryHistoryCache::new(root.clone());
        let (checkpoint, observation) = cache
            .load(ENDPOINT, PREFIX, 10)
            .expect("restore durable prefix");
        assert_eq!(
            observation.disposition,
            RegistryHistoryCacheDisposition::Reused
        );
        assert_eq!(
            checkpoint.expect("complete prefix").version,
            durable_version
        );
        let keys = futures::executor::block_on(collect_key_family(
            PREFIX,
            10,
            &counter(&root, ENDPOINT),
            |cursor| {
                assert_eq!(cursor, durable_version);
                std::future::ready(Ok(response(
                    10,
                    &((cursor + 1)..=10)
                        .map(|version| (version, true))
                        .collect::<Vec<_>>(),
                )))
            },
        ))
        .expect("resume without replaying durable history");
        assert_eq!(keys, ["canister_ranges_00"]);
        assert_eq!(
            cache
                .load(ENDPOINT, PREFIX, 10)
                .expect("restored final prefix")
                .0
                .expect("final checkpoint")
                .version,
            10
        );
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[test]
fn fresh_sources_restore_tombstones_and_resume_only_the_bound_endpoint_and_prefix() {
    let root = temp_dir("ic-query-history-source-isolation");
    seed(&root);
    futures::executor::block_on(async {
        let keys = collect_key_family(PREFIX, 4, &counter(&root, ENDPOINT), |cursor| {
            assert_eq!(cursor, 3);
            std::future::ready(Ok(response(4, &[(4, true)])))
        })
        .await
        .expect("resume history");
        assert_eq!(keys, ["canister_ranges_00"]);
        for (endpoint, prefix) in [("https://b.example", PREFIX), (ENDPOINT, "different_")] {
            collect_key_family(prefix, 4, &counter(&root, endpoint), |cursor| {
                assert_eq!(cursor, 0);
                std::future::ready(Ok(response(
                    4,
                    &[(1, true), (2, true), (3, false), (4, true)],
                )))
            })
            .await
            .expect("distinct identity replays from zero");
        }
        let keys = collect_key_family(PREFIX, 4, &counter(&root, ENDPOINT), |_| {
            panic!("same pin must not fetch any history pages");
            #[expect(unreachable_code)]
            std::future::ready(Ok(response(4, &[])))
        })
        .await
        .expect("same-pin reuse");
        assert_eq!(keys, ["canister_ranges_00"]);
    });
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn older_pins_replay_from_zero_without_regressing_the_disk_prefix() {
    let root = temp_dir("ic-query-history-older-pin");
    seed(&root);
    let cache = RegistryHistoryCache::new(root.clone());
    let original = fs::read(&cache.path).expect("checkpoint bytes");
    let keys = futures::executor::block_on(collect_key_family(
        PREFIX,
        1,
        &counter(&root, ENDPOINT),
        |cursor| {
            assert_eq!(cursor, 0);
            std::future::ready(Ok(response(3, &[(1, true), (2, true), (3, false)])))
        },
    ))
    .expect("older pin");
    assert_eq!(keys, ["canister_ranges_00"]);
    assert_eq!(
        fs::read(&cache.path).expect("checkpoint preserved"),
        original
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn replay_caps_boundary_pages_at_the_saved_pin_before_applying_later_deletions() {
    let root = temp_dir("ic-query-history-boundary-pin");
    let first = futures::executor::block_on(collect_key_family(
        PREFIX,
        1,
        &counter(&root, ENDPOINT),
        |_| std::future::ready(Ok(response(3, &[(1, true), (2, false), (3, true)]))),
    ))
    .expect("first pin");
    assert_eq!(first, ["canister_ranges_00"]);
    let second = futures::executor::block_on(collect_key_family(
        PREFIX,
        2,
        &counter(&root, ENDPOINT),
        |cursor| {
            assert_eq!(cursor, 1);
            std::future::ready(Ok(response(3, &[(2, false), (3, true)])))
        },
    ))
    .expect("second pin");
    assert_eq!(second, Vec::<String>::new());
    let third = futures::executor::block_on(collect_key_family(
        PREFIX,
        2,
        &counter(&root, ENDPOINT),
        |_| async {
            panic!("restoring the same pin must preserve its tombstone without live history");
        },
    ))
    .expect("restore capped transcript");
    assert_eq!(third, Vec::<String>::new());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn restored_history_preserves_cumulative_delta_resource_accounting() {
    let root = temp_dir("ic-query-history-resource-accounting");
    let mut page = response(1, &[(1, true)]);
    page.deltas
        .extend((1..100_000).map(|_| HighCapacityRegistryDelta {
            key: b"unrelated".to_vec(),
            values: Vec::new(),
        }));
    futures::executor::block_on(collect_key_family(
        PREFIX,
        1,
        &counter(&root, ENDPOINT),
        |_| std::future::ready(Ok(page.clone())),
    ))
    .expect("bounded complete prefix");
    let error = futures::executor::block_on(collect_key_family(
        PREFIX,
        2,
        &counter(&root, ENDPOINT),
        |cursor| {
            assert_eq!(cursor, 1);
            std::future::ready(Ok(response(2, &[(2, true)])))
        },
    ))
    .expect_err("resource counts survive process/source replacement");
    assert!(matches!(
        error,
        RegistryFetchError::RegistryKeyFamilyLimit {
            field: "delta_keys",
            maximum: 100_000,
            actual: 100_001
        }
    ));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn oversized_history_is_rejected_before_reading_and_repaired_by_authorized_collection() {
    let root = temp_dir("ic-query-history-oversized");
    seed(&root);
    let cache = RegistryHistoryCache::new(root.clone());
    fs::OpenOptions::new()
        .write(true)
        .open(&cache.path)
        .unwrap()
        .set_len(MAX_HISTORY_BYTES + 1)
        .unwrap();
    let (checkpoint, observation) = cache.load(ENDPOINT, PREFIX, 3).unwrap();
    assert!(checkpoint.is_none());
    assert_eq!(
        observation.disposition,
        RegistryHistoryCacheDisposition::Rejected
    );
    seed(&root);
    assert!(cache.load(ENDPOINT, PREFIX, 3).unwrap().0.is_some());
    assert!(fs::metadata(&cache.path).unwrap().len() < MAX_HISTORY_BYTES);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn rejected_history_and_failed_collection_leave_existing_bytes_untouched() {
    let root = temp_dir("ic-query-history-invalid-and-offline");
    seed(&root);
    let path = RegistryHistoryCache::new(root.clone()).path;
    write_managed_text_atomically(&root, &path, "invalid history").unwrap();
    let error = futures::executor::block_on(collect_key_family(
        PREFIX,
        4,
        &counter(&root, ENDPOINT),
        |cursor| {
            assert_eq!(cursor, 0);
            std::future::ready(Err(RegistryFetchError::AgentCall {
                method: "get_changes_since",
                reason: "fixture unavailable".to_string(),
            }))
        },
    ))
    .expect_err("transport failure");
    assert!(matches!(error, RegistryFetchError::AgentCall { .. }));
    assert_eq!(fs::read_to_string(&path).unwrap(), "invalid history");
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn history_identity_retention_is_bounded_without_failing_collection() {
    let root = temp_dir("ic-query-history-identity-limit");
    for index in 0..=MAX_HISTORY_CHECKPOINTS {
        futures::executor::block_on(collect_key_family(
            PREFIX,
            1,
            &counter(&root, &format!("https://{index}.example")),
            |cursor| {
                assert_eq!(cursor, 0);
                std::future::ready(Ok(response(1, &[(1, true)])))
            },
        ))
        .expect("collection succeeds with full retention");
    }
    let cache = RegistryHistoryCache::new(root.clone());
    let (document, rejection) = cache.read().unwrap();
    assert!(rejection.is_none());
    assert_eq!(document.checkpoints.len(), MAX_HISTORY_CHECKPOINTS);
    assert!(
        cache
            .load("https://8.example", PREFIX, 1)
            .unwrap()
            .0
            .is_none()
    );
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn competing_writer_lock_skips_publication_and_preserves_prior_history() {
    let root = temp_dir("ic-query-history-competing-writer");
    seed(&root);
    let cache = RegistryHistoryCache::new(root.clone());
    let original = fs::read(&cache.path).unwrap();
    with_refresh_lock(
        RefreshLockRequest {
            cache_root: &root,
            lock_path: &cache.lock_path,
            target_path: &cache.path,
            network: MAINNET_NETWORK,
            now_unix_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            lock_stale_after_seconds: 600,
        },
        operation,
        || {
            let keys = futures::executor::block_on(collect_key_family(
                PREFIX,
                4,
                &counter(&root, ENDPOINT),
                |cursor| {
                    assert_eq!(cursor, 3);
                    std::future::ready(Ok(response(4, &[(4, true)])))
                },
            ))
            .expect("competing writer does not prevent live collection");
            assert_eq!(keys, ["canister_ranges_00"]);
            assert_eq!(fs::read(&cache.path).unwrap(), original);
            Ok(())
        },
    )
    .expect("release fixture writer lock");
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn stale_writer_lock_remains_visible_and_requires_manual_cleanup() {
    let root = temp_dir("ic-query-history-stale-writer");
    seed(&root);
    let cache = RegistryHistoryCache::new(root.clone());
    let original = fs::read(&cache.path).unwrap();
    let lock = serde_json::json!({
        "schema_version": 1, "network": MAINNET_NETWORK, "pid": 123,
        "started_at_unix_ms": 0, "stale_after_seconds": 600,
        "target_path": cache.path.display().to_string(),
    });
    write_managed_text_atomically(&root, &cache.lock_path, &lock.to_string()).unwrap();
    let error = futures::executor::block_on(collect_key_family(
        PREFIX,
        4,
        &counter(&root, ENDPOINT),
        |cursor| {
            assert_eq!(cursor, 3);
            std::future::ready(Ok(response(4, &[(4, true)])))
        },
    ))
    .expect_err("stale lock");
    assert!(matches!(
        error,
        RegistryFetchError::HistoryCache(HostCacheError::Operation {
            source: CacheFileError::StaleRefreshLock { .. },
            ..
        })
    ));
    assert!(cache.lock_path.exists());
    assert_eq!(fs::read(&cache.path).unwrap(), original);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn cancellation_keeps_only_complete_pages_for_a_new_source() {
    let root = temp_dir("ic-query-history-cancelled");
    let first = counter(&root, ENDPOINT);
    let mut future = Box::pin(collect_key_family(PREFIX, 3, &first, |cursor| async move {
        if cursor == 0 {
            Ok(response(3, &[(1, true)]))
        } else {
            std::future::pending().await
        }
    }));
    let waker = futures::task::noop_waker();
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_pending()
    );
    drop(future);
    drop(first);
    let keys = futures::executor::block_on(collect_key_family(
        PREFIX,
        3,
        &counter(&root, ENDPOINT),
        |cursor| {
            assert_eq!(cursor, 1);
            std::future::ready(Ok(response(3, &[(2, true), (3, false)])))
        },
    ))
    .expect("resume cancelled prefix");
    assert_eq!(keys, Vec::<String>::new());
    assert!(!RegistryHistoryCache::new(root.clone()).lock_path.exists());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn invalid_live_pages_preserve_the_last_durable_prefix() {
    let root = temp_dir("ic-query-history-invalid-page");
    seed(&root);
    let path = RegistryHistoryCache::new(root.clone()).path;
    let original = fs::read(&path).expect("checkpoint bytes");
    let error = futures::executor::block_on(collect_key_family(
        PREFIX,
        5,
        &counter(&root, ENDPOINT),
        |cursor| {
            assert_eq!(cursor, 3);
            std::future::ready(Ok(response(5, &[(5, true)])))
        },
    ))
    .expect_err("missing version rejects page");
    assert!(matches!(
        error,
        RegistryFetchError::InvalidRegistryKeyFamily { .. }
    ));
    assert_eq!(fs::read(path).expect("durable prefix preserved"), original);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn corrupt_content_is_visible_and_replaced_only_after_a_valid_cold_page() {
    for invalid in [
        "invalid JSON",
        "checksum",
        "gap",
        "network",
        "registry",
        "watermark",
        "unknown",
        "schema",
    ] {
        let root = temp_dir("ic-query-history-corrupt");
        seed(&root);
        let cache = RegistryHistoryCache::new(root.clone());
        if invalid == "invalid JSON" {
            write_managed_text_atomically(&root, &cache.path, "not JSON").expect("corrupt JSON");
        } else {
            rewrite_document(&root, |document| match invalid {
                "checksum" => document.checkpoints[0].endpoint.push_str("/changed"),
                "gap" => {
                    let bytes = crate::hex::decode_lowercase_hex(
                        &document.checkpoints[0].pages[0].response_hex,
                    )
                    .unwrap();
                    let mut page =
                        HighCapacityRegistryGetChangesSinceResponse::decode(bytes.as_slice())
                            .unwrap();
                    page.deltas[0].values.retain(|value| value.version != 2);
                    document.checkpoints[0].pages[0].response_hex =
                        crate::hex::hex_bytes(&page.encode_to_vec());
                }
                "network" => document.checkpoints[0].network = "local".to_string(),
                "registry" => document.checkpoints[0].registry_canister = "aaaaa-aa".to_string(),
                "watermark" => document.checkpoints[0].pages[0].through_version = 0,
                "schema" => document.schema_version = 0,
                "unknown" => document.checkpoints[0].pages[0].response_hex = "xyz".to_string(),
                _ => unreachable!(),
            });
            if invalid == "checksum" {
                let mut json: serde_json::Value =
                    serde_json::from_slice(&fs::read(&cache.path).unwrap()).unwrap();
                json["digest"] = serde_json::json!("bad digest");
                write_managed_text_atomically(&root, &cache.path, &json.to_string()).unwrap();
            }
        }
        let events = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&events);
        let acquisition = RegistryAcquisition::with_history_cache(
            root.clone(),
            Some(Arc::new(move |event: SubnetCatalogProgress| {
                observed.lock().unwrap().push(event);
            })),
        );
        let counter =
            RegistryQueryCounter::with_acquisition(ENDPOINT.to_string(), Arc::new(acquisition));
        futures::executor::block_on(collect_key_family(PREFIX, 4, &counter, |cursor| {
            assert_eq!(cursor, 0, "{invalid}");
            std::future::ready(Ok(response(
                4,
                &[(1, true), (2, true), (3, false), (4, true)],
            )))
        }))
        .expect("recover with valid cold history");
        assert!(
            events.lock().unwrap().iter().any(|event| matches!(
                event.phase,
                SubnetCatalogProgressPhase::HistoryCache {
                    disposition: RegistryHistoryCacheDisposition::Rejected,
                    ..
                }
            )),
            "{invalid}"
        );
        assert!(cache.load(ENDPOINT, PREFIX, 4).unwrap().0.is_some());
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[cfg(unix)]
#[test]
fn confinement_and_permissions_remain_errors_without_cold_network_fallback() {
    for make_symlink in [true, false] {
        let root = temp_dir("ic-query-history-confinement");
        seed(&root);
        let path = RegistryHistoryCache::new(root.clone()).path;
        if make_symlink {
            let other = root.join("other.json");
            fs::rename(&path, &other).expect("move fixture");
            symlink(&other, &path).expect("link fixture");
        } else {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644))
                .expect("unsafe permissions");
        }
        let error = futures::executor::block_on(collect_key_family(
            PREFIX,
            4,
            &counter(&root, ENDPOINT),
            |_| {
                panic!("filesystem authority errors must not trigger cold collection");
                #[expect(unreachable_code)]
                std::future::ready(Ok(response(4, &[])))
            },
        ))
        .expect_err("unsafe checkpoint");
        assert!(matches!(
            error,
            RegistryFetchError::HistoryCache(HostCacheError::Operation {
                source: CacheFileError::Confinement { .. }
                    | CacheFileError::UnsafeManagedPermissions { .. },
                ..
            })
        ));
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[test]
fn separate_processes_resume_disk_history_without_shared_memory() {
    let root = temp_dir("ic-query-history-processes");
    for phase in ["seed", "resume"] {
        let output = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "ic_registry::transport::history_cache::tests::history_process_fixture_worker",
                "--nocapture",
            ])
            .env("ICQ_HISTORY_FIXTURE_ROOT", &root)
            .env("ICQ_HISTORY_FIXTURE_PHASE", phase)
            .output()
            .expect("run fixture process");
        assert!(
            output.status.success(),
            "{phase}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn history_process_fixture_worker() {
    let Some(root) = std::env::var_os("ICQ_HISTORY_FIXTURE_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    if std::env::var("ICQ_HISTORY_FIXTURE_PHASE").as_deref() == Ok("seed") {
        seed(&root);
        return;
    }
    let keys = futures::executor::block_on(collect_key_family(
        PREFIX,
        4,
        &counter(&root, ENDPOINT),
        |cursor| {
            assert_eq!(
                cursor, 3,
                "second process resumes the first process's complete prefix"
            );
            std::future::ready(Ok(response(4, &[(4, true)])))
        },
    ))
    .expect("cross-process resume");
    assert_eq!(keys, ["canister_ranges_00"]);
}
