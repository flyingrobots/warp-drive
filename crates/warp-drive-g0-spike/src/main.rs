// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! G0 gate spike — proves warp-wasm can be embedded in a native Rust binary
//! outside the echo workspace and that a single `observe` call round-trips.
//!
//! Exit 0 = G0 PASS.  Exit 1 = G0 FAIL.

use echo_wasm_abi::kernel_port::{
    ObservationAt, ObservationCoordinate, ObservationFrame, ObservationProjection,
    ObservationRequest, OkEnvelope, ObservationArtifact,
};

fn main() {
    println!("=== G0 spike: embedding warp-wasm outside echo workspace ===");
    println!();

    // ── Step 1: initialize the embedded engine kernel ──────────────────────
    print!("step 1  init_embedded() ... ");
    let handle = match warp_wasm::init_embedded() {
        Ok(h) => h,
        Err(e) => {
            println!("FAIL");
            eprintln!("  error {}: {}", e.code, e.message);
            std::process::exit(1);
        }
    };
    println!("ok");
    println!(
        "        worldline_tick = {}",
        handle.head.worldline_tick.0
    );
    println!(
        "        state_root     = {} bytes (non-zero: {})",
        handle.head.state_root.len(),
        handle.head.state_root.iter().any(|&b| b != 0),
    );
    println!(
        "        commit_id      = {} bytes",
        handle.head.commit_id.len(),
    );

    // ── Step 2: build an ObservationRequest for the default worldline ──────
    print!("step 2  build ObservationRequest ... ");
    let request = match ObservationRequest::builtin_one_shot(
        ObservationCoordinate {
            worldline_id: handle.worldline_id,
            at: ObservationAt::Frontier,
        },
        ObservationFrame::CommitBoundary,
        ObservationProjection::Head,
    ) {
        Ok(r) => r,
        Err(e) => {
            println!("FAIL");
            eprintln!("  {e:?}");
            std::process::exit(1);
        }
    };
    println!("ok");

    // ── Step 3: encode request → CBOR bytes ───────────────────────────────
    print!("step 3  encode request to CBOR ... ");
    let request_bytes = match echo_wasm_abi::encode_cbor(&request) {
        Ok(b) => b,
        Err(e) => {
            println!("FAIL");
            eprintln!("  {e:?}");
            std::process::exit(1);
        }
    };
    println!("ok ({} bytes)", request_bytes.len());

    // ── Step 4: observe_cbor round-trip ───────────────────────────────────
    print!("step 4  observe_cbor() ... ");
    let response_bytes = warp_wasm::observe_cbor(&request_bytes);

    // ── Step 5: decode response ────────────────────────────────────────────
    let artifact: OkEnvelope<ObservationArtifact> =
        match echo_wasm_abi::decode_cbor(&response_bytes) {
            Ok(env) => env,
            Err(e) => {
                println!("FAIL");
                eprintln!("  decode error: {e:?}");
                // Try to decode as error envelope for better diagnostics
                if let Ok(err) = echo_wasm_abi::decode_cbor::<echo_wasm_abi::kernel_port::ErrEnvelope>(
                    &response_bytes,
                ) {
                    eprintln!("  kernel error {}: {}", err.code, err.message);
                }
                std::process::exit(1);
            }
        };
    println!("ok");
    println!(
        "        artifact_hash  = {} bytes",
        artifact.data.artifact_hash.len(),
    );
    println!(
        "        resolved_tick  = {}",
        artifact.data.resolved.resolved_worldline_tick.0,
    );

    // ── Result ─────────────────────────────────────────────────────────────
    println!();
    println!("=== G0: PASS ===");
    println!();
    println!("Finding: warp-wasm embeds as an rlib and observe round-trips cleanly.");
    println!("Finding: the wasm32-unknown-unknown build uses wasm-bindgen ABI and");
    println!("         requires a JS host — wasmtime cannot load it without one.");
    println!("         The rlib path IS the correct embedded surface for WARP DRIVE.");
}
