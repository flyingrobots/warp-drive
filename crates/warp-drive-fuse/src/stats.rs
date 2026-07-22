// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Live mount diagnostics owned by the FUSE adapter.
//!
//! `MountStats` is the counter state behind `/.warp/stats`, and the label
//! pair (`GateLabel`, `RuntimeLabel`) behind both `/.warp/stats` and
//! `/.warp/runtime`'s `"gate"`/`"runtime"` fields.
//!
//! **Layer:** platform (FUSE glue; depends on `warp-drive-core` only).
//!
//! **Introduced at:** G3.

use std::sync::atomic::{AtomicU64, Ordering};

use warp_drive_core::WARP_DIAGNOSTICS_SCHEMA_VERSION;

/// Gate identity reported by live mount diagnostics.
///
/// Identifies which CLI `--gate`/fixture-construction path built this mount
/// — not necessarily the gate whose Echo payload provenance produced
/// `/echo/head.json`'s bytes (G3 reuses G2b's exact projection call
/// unmodified; only the surrounding metadata and counters are new).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateLabel {
    /// G1 in-memory POSIX-read gate.
    G1,
    /// G2a Echo coordinate gate.
    G2a,
    /// G2b Echo projected-file gate.
    G2b,
    /// G3 live-diagnostics gate.
    G3,
}

impl GateLabel {
    const fn as_str(self) -> &'static str {
        match self {
            Self::G1 => "G1",
            Self::G2a => "G2a",
            Self::G2b => "G2b",
            Self::G3 => "G3",
        }
    }
}

/// Runtime backend identity reported by live mount diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeLabel {
    /// Hardcoded in-memory fixture tree.
    InMemory,
    /// Embedded Echo rlib backend.
    EchoRlib,
}

impl RuntimeLabel {
    const fn as_str(self) -> &'static str {
        match self {
            Self::InMemory => "in-memory",
            Self::EchoRlib => "echo-rlib",
        }
    }
}

/// Live operation counters served by `/.warp/stats`.
///
/// Counters are per mount process; they reset on remount. Each counter is an
/// independent [`AtomicU64`] incremented with [`Ordering::Relaxed`] — there is
/// no ordering dependency between the fields, so a snapshot read across all
/// of them is not transactionally coherent (two fields read a moment apart
/// may reflect different instants under concurrent FUSE worker threads).
/// That's an accepted, documented property of this design, not a bug.
pub struct MountStats {
    gate: GateLabel,
    runtime: RuntimeLabel,
    lookup_count: AtomicU64,
    getattr_count: AtomicU64,
    readdir_count: AtomicU64,
    open_count: AtomicU64,
    read_count: AtomicU64,
    readlink_count: AtomicU64,
    runtime_observe_count: AtomicU64,
    runtime_observe_error_count: AtomicU64,
}

impl MountStats {
    /// Construct fresh, zeroed FUSE-side counters, seeded with the startup
    /// Echo-observation accounting the caller already performed (`0`/`0` for
    /// runtimes with no Echo involvement).
    #[must_use]
    pub const fn new(
        gate: GateLabel,
        runtime: RuntimeLabel,
        runtime_observe_count: u64,
        runtime_observe_error_count: u64,
    ) -> Self {
        Self {
            gate,
            runtime,
            lookup_count: AtomicU64::new(0),
            getattr_count: AtomicU64::new(0),
            readdir_count: AtomicU64::new(0),
            open_count: AtomicU64::new(0),
            read_count: AtomicU64::new(0),
            readlink_count: AtomicU64::new(0),
            runtime_observe_count: AtomicU64::new(runtime_observe_count),
            runtime_observe_error_count: AtomicU64::new(runtime_observe_error_count),
        }
    }

    pub(crate) fn record_lookup(&self) {
        self.lookup_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_getattr(&self) {
        self.getattr_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_readdir(&self) {
        self.readdir_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_open(&self) {
        self.open_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_read(&self) {
        self.read_count.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_readlink(&self) {
        self.readlink_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Serialize a fresh snapshot of every counter as `/.warp/stats`' JSON
    /// body.
    ///
    /// Every `u64` is right-aligned in a fixed 20-character field —
    /// insignificant whitespace before a JSON number is legal, so this keeps
    /// the document's byte length constant regardless of counter magnitude
    /// (no digit-growth/stale-cached-size race as counters cross decimal
    /// boundaries).
    pub(crate) fn snapshot_json(&self) -> Vec<u8> {
        format!(
            "{{\"gate\":\"{}\",\"runtime\":\"{}\",\"schema_version\":{},\
              \"lookup_count\":{:>20},\"getattr_count\":{:>20},\
              \"readdir_count\":{:>20},\"open_count\":{:>20},\
              \"read_count\":{:>20},\"readlink_count\":{:>20},\
              \"runtime_observe_count\":{:>20},\"runtime_observe_error_count\":{:>20}}}\n",
            self.gate.as_str(),
            self.runtime.as_str(),
            WARP_DIAGNOSTICS_SCHEMA_VERSION,
            self.lookup_count.load(Ordering::Relaxed),
            self.getattr_count.load(Ordering::Relaxed),
            self.readdir_count.load(Ordering::Relaxed),
            self.open_count.load(Ordering::Relaxed),
            self.read_count.load(Ordering::Relaxed),
            self.readlink_count.load(Ordering::Relaxed),
            self.runtime_observe_count.load(Ordering::Relaxed),
            self.runtime_observe_error_count.load(Ordering::Relaxed),
        )
        .into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::{GateLabel, MountStats, RuntimeLabel};

    fn json(stats: &MountStats) -> String {
        // `snapshot_json` only ever interpolates `Display` of integers and
        // `&'static str` labels via `format!`, so the result is always valid
        // UTF-8 — `from_utf8_lossy` is a no-op here, just one that doesn't
        // need `.unwrap()`/`.expect()` (denied workspace-wide).
        String::from_utf8_lossy(&stats.snapshot_json()).into_owned()
    }

    #[test]
    fn fresh_snapshot_is_all_zero_with_correct_labels() {
        let stats = MountStats::new(GateLabel::G3, RuntimeLabel::InMemory, 0, 0);
        let snapshot = json(&stats);

        assert!(snapshot.contains("\"gate\":\"G3\""));
        assert!(snapshot.contains("\"runtime\":\"in-memory\""));
        assert!(snapshot.contains("\"schema_version\":1"));
        for key in [
            "lookup_count",
            "getattr_count",
            "readdir_count",
            "open_count",
            "read_count",
            "readlink_count",
            "runtime_observe_count",
            "runtime_observe_error_count",
        ] {
            assert!(
                snapshot.contains(&format!("\"{key}\":{:>20}", 0)),
                "missing or non-zero {key} in {snapshot}"
            );
        }
    }

    #[test]
    fn record_methods_bump_only_their_own_counter() {
        let stats = MountStats::new(GateLabel::G1, RuntimeLabel::InMemory, 0, 0);
        stats.record_lookup();
        let snapshot = json(&stats);

        assert!(snapshot.contains(&format!("\"lookup_count\":{:>20}", 1)));
        for key in [
            "getattr_count",
            "readdir_count",
            "open_count",
            "read_count",
            "readlink_count",
            "runtime_observe_count",
            "runtime_observe_error_count",
        ] {
            assert!(snapshot.contains(&format!("\"{key}\":{:>20}", 0)));
        }
    }

    #[test]
    fn snapshot_has_all_eleven_keys() {
        let stats = MountStats::new(GateLabel::G2b, RuntimeLabel::EchoRlib, 2, 0);
        let snapshot = json(&stats);

        for key in [
            "gate",
            "runtime",
            "schema_version",
            "lookup_count",
            "getattr_count",
            "readdir_count",
            "open_count",
            "read_count",
            "readlink_count",
            "runtime_observe_count",
            "runtime_observe_error_count",
        ] {
            assert!(
                snapshot.contains(&format!("\"{key}\":")),
                "missing key {key}"
            );
        }
    }

    #[test]
    fn snapshot_byte_length_is_stable_across_decimal_boundaries() {
        let zero = MountStats::new(GateLabel::G3, RuntimeLabel::InMemory, 0, 0);
        let len_at_zero = zero.snapshot_json().len();

        for _ in 0..9 {
            zero.record_lookup();
        }
        assert_eq!(
            zero.snapshot_json().len(),
            len_at_zero,
            "length changed at 9"
        );

        zero.record_lookup();
        assert_eq!(
            zero.snapshot_json().len(),
            len_at_zero,
            "length changed at 10"
        );

        let max = MountStats::new(GateLabel::G3, RuntimeLabel::InMemory, u64::MAX, u64::MAX);
        assert_eq!(
            max.snapshot_json().len(),
            len_at_zero,
            "length changed at u64::MAX"
        );
    }

    #[test]
    fn exact_snapshot_string_for_fresh_g1_in_memory() {
        let stats = MountStats::new(GateLabel::G1, RuntimeLabel::InMemory, 0, 0);
        let expected = "{\"gate\":\"G1\",\"runtime\":\"in-memory\",\"schema_version\":1,\
            \"lookup_count\":                   0,\"getattr_count\":                   0,\
            \"readdir_count\":                   0,\"open_count\":                   0,\
            \"read_count\":                   0,\"readlink_count\":                   0,\
            \"runtime_observe_count\":                   0,\"runtime_observe_error_count\":                   0}\n";
        assert_eq!(json(&stats), expected);
    }
}
