// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Echo rlib-backed projection adapter for G2 gates.
//!
//! G2a deliberately does not claim Echo-projected file bytes. It initializes a
//! real embedded Echo kernel, performs one `observe_cbor()` head observation on
//! the main thread, and bakes that coordinate metadata into the `.warp/` files
//! of the existing fixture tree.
//!
//! G2b adds one normal projected file, `/echo/head.json`, whose bytes come from
//! an Echo `QueryView` observation returning `ObservationPayload::QueryBytes`.
//!
//! G3 adds no new Echo call — `init_g3()` performs the exact same two
//! observations as `init_g2b()` — but reports real
//! [`RuntimeObservationStats`] accounting instead of a downstream-hardcoded
//! constant, and stamps `/.warp/runtime` with the G3 diagnostics shape.

use std::error::Error;
use std::fmt;

use echo_wasm_abi::kernel_port::{
    ErrEnvelope, ObservationArtifact, ObservationAt, ObservationCoordinate, ObservationFrame,
    ObservationPayload, ObservationProjection, ObservationRequest, OkEnvelope, WorldlineId,
};
use warp_drive_core::FixtureTree;

/// Startup observation accounting produced by [`EchoBackend`].
///
/// Fields are private and only readable through the documented getters below
/// — the downstream FUSE binary consumes this accounting, it does not get to
/// alter the backend's claim about what it actually did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RuntimeObservationStats {
    observe_count: u64,
    observe_error_count: u64,
}

impl RuntimeObservationStats {
    fn record_success(&mut self) {
        self.observe_count += 1;
    }

    /// Number of successful calls to `warp_wasm::observe_cbor` this backend
    /// issued.
    #[must_use]
    pub const fn observe_count(self) -> u64 {
        self.observe_count
    }

    /// Always `0` for any successfully constructed [`EchoBackend`] — an
    /// observation error aborts `init_*()` via `?` before a backend exists to
    /// report from. Not a durable count of failed startup attempts.
    #[must_use]
    pub const fn observe_error_count(self) -> u64 {
        self.observe_error_count
    }
}

/// Backend that owns the cached fixture tree served by the FUSE adapter.
pub struct EchoBackend {
    tree: FixtureTree,
    observation_stats: RuntimeObservationStats,
}

impl EchoBackend {
    /// Initialize Echo, observe its current head, and build a cached G2a tree.
    ///
    /// # Errors
    ///
    /// Returns [`EchoBackendError`] if Echo initialization, request encoding,
    /// kernel observation, or response decoding fails.
    pub fn init() -> Result<Self, EchoBackendError> {
        let handle = warp_wasm::init_embedded()
            .map_err(|err| EchoBackendError::Init(format!("{}: {}", err.code, err.message)))?;
        let mut observation_stats = RuntimeObservationStats::default();
        let artifact = observe_head(handle.worldline_id)?;
        observation_stats.record_success();
        let metadata = EchoCoordinateMetadata::from_artifact(&artifact)?;
        let tree = FixtureTree::with_warp_metadata(
            metadata.coordinate_json("G2a").into_bytes(),
            metadata.runtime_json("G2a").into_bytes(),
        )
        .map_err(|err| EchoBackendError::FixtureTree(err.to_string()))?;

        Ok(Self {
            tree,
            observation_stats,
        })
    }

    /// Initialize Echo, observe coordinate metadata, project `/echo/head.json`,
    /// and build a cached G2b tree.
    ///
    /// # Errors
    ///
    /// Returns [`EchoBackendError`] if Echo initialization, request encoding,
    /// kernel observation, projection decoding, or fixture construction fails.
    pub fn init_g2b() -> Result<Self, EchoBackendError> {
        let handle = warp_wasm::init_embedded()
            .map_err(|err| EchoBackendError::Init(format!("{}: {}", err.code, err.message)))?;
        let mut observation_stats = RuntimeObservationStats::default();
        let artifact = observe_head(handle.worldline_id)?;
        observation_stats.record_success();
        let metadata = EchoCoordinateMetadata::from_artifact(&artifact)?;
        let echo_head_json = observe_echo_head_file(handle.worldline_id, &metadata)?;
        observation_stats.record_success();
        let tree = FixtureTree::with_warp_metadata_and_echo_head_file(
            metadata.coordinate_json("G2b").into_bytes(),
            metadata.runtime_json("G2b").into_bytes(),
            echo_head_json,
        )
        .map_err(|err| EchoBackendError::FixtureTree(err.to_string()))?;

        Ok(Self {
            tree,
            observation_stats,
        })
    }

    /// Initialize Echo and build a cached G3 tree.
    ///
    /// Performs the exact same two observations as [`Self::init_g2b`] (head +
    /// query-projected `/echo/head.json`) — G3 adds no new Echo call, only
    /// live FUSE-side diagnostics and a new `/.warp/runtime` shape.
    ///
    /// # Errors
    ///
    /// Returns [`EchoBackendError`] if Echo initialization, request encoding,
    /// kernel observation, projection decoding, or fixture construction fails.
    pub fn init_g3() -> Result<Self, EchoBackendError> {
        let handle = warp_wasm::init_embedded()
            .map_err(|err| EchoBackendError::Init(format!("{}: {}", err.code, err.message)))?;
        let mut observation_stats = RuntimeObservationStats::default();
        let artifact = observe_head(handle.worldline_id)?;
        observation_stats.record_success();
        let metadata = EchoCoordinateMetadata::from_artifact(&artifact)?;
        let echo_head_json = observe_echo_head_file(handle.worldline_id, &metadata)?;
        observation_stats.record_success();
        let tree = FixtureTree::with_warp_metadata_and_echo_head_file(
            metadata.coordinate_json("G3").into_bytes(),
            metadata.runtime_json_g3().into_bytes(),
            echo_head_json,
        )
        .map_err(|err| EchoBackendError::FixtureTree(err.to_string()))?;

        Ok(Self {
            tree,
            observation_stats,
        })
    }

    /// Consume the backend and return the cached fixture tree.
    #[must_use]
    pub fn into_tree(self) -> FixtureTree {
        self.tree
    }

    /// Consume the backend, returning its fixture tree and the startup
    /// observation accounting it actually performed.
    #[must_use]
    pub fn into_parts(self) -> (FixtureTree, RuntimeObservationStats) {
        (self.tree, self.observation_stats)
    }
}

/// Errors produced by the Echo metadata backend.
#[derive(Debug)]
pub enum EchoBackendError {
    /// `warp_wasm::init_embedded()` failed.
    Init(String),
    /// Constructing the observation request failed.
    BuildRequest(String),
    /// Encoding the request to CBOR failed.
    EncodeRequest(String),
    /// Echo returned an error envelope.
    Kernel(String),
    /// Decoding the Echo response failed.
    DecodeResponse(String),
    /// Echo returned the wrong payload kind for the request.
    UnexpectedPayload {
        /// The payload kind that was expected.
        expected: &'static str,
    },
    /// Echo returned an artifact whose head and resolved coordinate disagree.
    InconsistentArtifact {
        /// Name of the field that disagreed.
        field: &'static str,
        /// Value from the observation's `head`.
        head: String,
        /// Value from the observation's `resolved` coordinate.
        resolved: String,
    },
    /// Cached fixture tree construction failed.
    FixtureTree(String),
}

impl fmt::Display for EchoBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Init(message) => write!(f, "Echo init failed: {message}"),
            Self::BuildRequest(message) => {
                write!(f, "Echo observation request construction failed: {message}")
            }
            Self::EncodeRequest(message) => {
                write!(f, "Echo observation request encoding failed: {message}")
            }
            Self::Kernel(message) => write!(f, "Echo observation failed: {message}"),
            Self::DecodeResponse(message) => {
                write!(f, "Echo observation response decoding failed: {message}")
            }
            Self::UnexpectedPayload { expected } => {
                write!(f, "Echo observation returned a non-{expected} payload")
            }
            Self::InconsistentArtifact {
                field,
                head,
                resolved,
            } => write!(
                f,
                "Echo observation artifact has inconsistent {field}: head={head}, resolved={resolved}"
            ),
            Self::FixtureTree(message) => {
                write!(f, "Echo metadata fixture construction failed: {message}")
            }
        }
    }
}

impl Error for EchoBackendError {}

struct EchoCoordinateMetadata {
    worldline: String,
    frontier: String,
    state_root: String,
    tick: u64,
    artifact_hash: String,
}

impl EchoCoordinateMetadata {
    fn from_artifact(artifact: &ObservationArtifact) -> Result<Self, EchoBackendError> {
        let ObservationPayload::Head { head } = &artifact.payload else {
            return Err(EchoBackendError::UnexpectedPayload { expected: "head" });
        };

        let frontier = hex(&head.commit_id);
        let resolved_frontier = hex(&artifact.resolved.commit_hash);
        ensure_artifact_field("frontier", &frontier, &resolved_frontier)?;

        let state_root = hex(&head.state_root);
        let resolved_state_root = hex(&artifact.resolved.state_root);
        ensure_artifact_field("state_root", &state_root, &resolved_state_root)?;

        let tick = head.worldline_tick.as_u64();
        let resolved_tick = artifact.resolved.resolved_worldline_tick.as_u64();
        ensure_artifact_field(
            "worldline_tick",
            &tick.to_string(),
            &resolved_tick.to_string(),
        )?;

        Ok(Self {
            worldline: hex(artifact.resolved.worldline_id.as_bytes()),
            frontier,
            state_root,
            tick,
            artifact_hash: hex(&artifact.artifact_hash),
        })
    }

    fn coordinate_json(&self, gate: &str) -> String {
        format!(
            "{{\"worldline\":\"{}\",\"frontier\":\"{}\",\"state_root\":\"{}\",\"tick\":{},\"artifact_hash\":\"{}\",\"backend\":\"echo-rlib\",\"gate\":\"{}\"}}\n",
            self.worldline, self.frontier, self.state_root, self.tick, self.artifact_hash, gate
        )
    }

    /// `/.warp/runtime` content for `init()` (G2a) and `init_g2b()` (G2b).
    ///
    /// Frozen — their acceptance scripts assert on this exact `"kind"`-shaped
    /// output. G3 uses [`Self::runtime_json_g3`] instead, never this method.
    fn runtime_json(&self, gate: &str) -> String {
        format!(
            "{{\"kind\":\"echo-rlib\",\"driver\":\"warp-wasm\",\"gate\":\"{}\",\"worldline\":\"{}\"}}\n",
            gate, self.worldline
        )
    }

    /// `/.warp/runtime` content for `init_g3()`, per the G3 design doc's
    /// required shape. Does not touch or replace [`Self::runtime_json`].
    fn runtime_json_g3(&self) -> String {
        format!(
            "{{\"gate\":\"G3\",\"runtime\":\"echo-rlib\",\"driver\":\"warp-wasm\",\
              \"build_mode\":\"{}\",\"stats\":\"live\",\"schema_version\":{},\
              \"worldline\":\"{}\"}}\n",
            warp_drive_core::build_mode(),
            warp_drive_core::WARP_DIAGNOSTICS_SCHEMA_VERSION,
            self.worldline
        )
    }
}

fn ensure_artifact_field(
    field: &'static str,
    head: &str,
    resolved: &str,
) -> Result<(), EchoBackendError> {
    if head == resolved {
        return Ok(());
    }

    Err(EchoBackendError::InconsistentArtifact {
        field,
        head: head.to_owned(),
        resolved: resolved.to_owned(),
    })
}

fn observe_head(worldline_id: WorldlineId) -> Result<ObservationArtifact, EchoBackendError> {
    let request = ObservationRequest::builtin_one_shot(
        ObservationCoordinate {
            worldline_id,
            at: ObservationAt::Frontier,
        },
        ObservationFrame::CommitBoundary,
        ObservationProjection::Head,
    )
    .map_err(|err| EchoBackendError::BuildRequest(format!("{err:?}")))?;

    let request_bytes = echo_wasm_abi::encode_cbor(&request)
        .map_err(|err| EchoBackendError::EncodeRequest(format!("{err:?}")))?;
    let response_bytes = warp_wasm::observe_cbor(&request_bytes);

    match echo_wasm_abi::decode_cbor::<OkEnvelope<ObservationArtifact>>(&response_bytes) {
        Ok(envelope) => Ok(envelope.data),
        Err(decode_error) => {
            if let Ok(error) = echo_wasm_abi::decode_cbor::<ErrEnvelope>(&response_bytes) {
                return Err(EchoBackendError::Kernel(format!(
                    "{}: {}",
                    error.code, error.message
                )));
            }
            Err(EchoBackendError::DecodeResponse(format!(
                "{decode_error:?}"
            )))
        }
    }
}

fn observe_echo_head_file(
    worldline_id: WorldlineId,
    metadata: &EchoCoordinateMetadata,
) -> Result<Vec<u8>, EchoBackendError> {
    let request = ObservationRequest::builtin_one_shot(
        ObservationCoordinate {
            worldline_id,
            at: ObservationAt::Frontier,
        },
        ObservationFrame::QueryView,
        ObservationProjection::Query {
            query_id: warp_wasm::experimental_warp_drive_g2b::HEAD_QUERY_ID,
            vars_bytes: warp_wasm::experimental_warp_drive_g2b::HEAD_QUERY_VARS.to_vec(),
        },
    )
    .map_err(|err| EchoBackendError::BuildRequest(format!("{err:?}")))?;

    let artifact = observe(request)?;
    ensure_artifact_field(
        "query_worldline",
        &hex(artifact.resolved.worldline_id.as_bytes()),
        &metadata.worldline,
    )?;
    ensure_artifact_field(
        "query_frontier",
        &hex(&artifact.resolved.commit_hash),
        &metadata.frontier,
    )?;
    ensure_artifact_field(
        "query_state_root",
        &hex(&artifact.resolved.state_root),
        &metadata.state_root,
    )?;

    let ObservationPayload::QueryBytes { data } = artifact.payload else {
        return Err(EchoBackendError::UnexpectedPayload {
            expected: "query bytes",
        });
    };
    Ok(data)
}

fn observe(request: ObservationRequest) -> Result<ObservationArtifact, EchoBackendError> {
    let request_bytes = echo_wasm_abi::encode_cbor(&request)
        .map_err(|err| EchoBackendError::EncodeRequest(format!("{err:?}")))?;
    let response_bytes = warp_wasm::observe_cbor(&request_bytes);

    match echo_wasm_abi::decode_cbor::<OkEnvelope<ObservationArtifact>>(&response_bytes) {
        Ok(envelope) => Ok(envelope.data),
        Err(decode_error) => {
            if let Ok(error) = echo_wasm_abi::decode_cbor::<ErrEnvelope>(&response_bytes) {
                return Err(EchoBackendError::Kernel(format!(
                    "{}: {}",
                    error.code, error.message
                )));
            }
            Err(EchoBackendError::DecodeResponse(format!(
                "{decode_error:?}"
            )))
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}
