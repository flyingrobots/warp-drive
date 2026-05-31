// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Echo rlib-backed metadata adapter for the G2a gate.
//!
//! G2a deliberately does not claim Echo-projected file bytes. It initializes a
//! real embedded Echo kernel, performs one `observe_cbor()` head observation on
//! the main thread, and bakes that coordinate metadata into the `.warp/` files
//! of the existing fixture tree.

use std::error::Error;
use std::fmt;

use echo_wasm_abi::kernel_port::{
    ErrEnvelope, ObservationArtifact, ObservationAt, ObservationCoordinate, ObservationFrame,
    ObservationPayload, ObservationProjection, ObservationRequest, OkEnvelope, WorldlineId,
};
use warp_drive_core::FixtureTree;

/// Backend that owns the cached fixture tree served by the FUSE adapter.
pub struct EchoBackend {
    tree: FixtureTree,
}

impl EchoBackend {
    /// Initialize Echo, observe its current head, and build a cached tree.
    ///
    /// # Errors
    ///
    /// Returns [`EchoBackendError`] if Echo initialization, request encoding,
    /// kernel observation, or response decoding fails.
    pub fn init() -> Result<Self, EchoBackendError> {
        let handle = warp_wasm::init_embedded()
            .map_err(|err| EchoBackendError::Init(format!("{}: {}", err.code, err.message)))?;
        let artifact = observe_head(handle.worldline_id)?;
        let metadata = EchoCoordinateMetadata::from_artifact(&artifact)?;
        let tree = FixtureTree::with_warp_metadata(
            metadata.coordinate_json().into_bytes(),
            metadata.runtime_json().into_bytes(),
            metadata.stats_json().into_bytes(),
        )
        .map_err(|err| EchoBackendError::FixtureTree(err.to_string()))?;

        Ok(Self { tree })
    }

    /// Consume the backend and return the cached fixture tree.
    #[must_use]
    pub fn into_tree(self) -> FixtureTree {
        self.tree
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
    /// Echo returned a non-head payload for a head request.
    UnexpectedPayload,
    /// Echo returned an artifact whose head and resolved coordinate disagree.
    InconsistentArtifact {
        field: &'static str,
        head: String,
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
            Self::UnexpectedPayload => f.write_str("Echo observation returned a non-head payload"),
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
            return Err(EchoBackendError::UnexpectedPayload);
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

    fn coordinate_json(&self) -> String {
        format!(
            "{{\"worldline\":\"{}\",\"frontier\":\"{}\",\"state_root\":\"{}\",\"tick\":{},\"artifact_hash\":\"{}\",\"backend\":\"echo-rlib\",\"gate\":\"G2a\"}}\n",
            self.worldline, self.frontier, self.state_root, self.tick, self.artifact_hash
        )
    }

    fn runtime_json(&self) -> String {
        format!(
            "{{\"kind\":\"echo-rlib\",\"driver\":\"warp-wasm\",\"gate\":\"G2a\",\"worldline\":\"{}\"}}\n",
            self.worldline
        )
    }

    fn stats_json(&self) -> String {
        "{\"gate\":\"G2a\",\"status\":\"static-placeholder\",\"note\":\"live counters arrive at G3\",\"lookup_count\":0,\"getattr_count\":0,\"readdir_count\":0,\"open_count\":0,\"read_count\":0,\"readlink_count\":0}\n"
            .to_owned()
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

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    out
}
