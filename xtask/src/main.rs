// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! WARP DRIVE developer task runner.
//!
//! Invoke with `cargo xtask <command>` (alias defined in `.cargo/config.toml`).
//!
//! | Command        | What it does                                              |
//! |----------------|-----------------------------------------------------------|
//! | `install-deps` | Install macFUSE via Homebrew (macOS only, no-op elsewhere)|
//! | `mount`        | Build and mount the WARP DRIVE FUSE filesystem            |
//! | `unmount`      | Unmount the WARP DRIVE FUSE filesystem                    |
//! | `acceptance`   | Run the selected gate acceptance test                     |

// xtask is a developer CLI — printing to stdout/stderr is intentional.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};

/// WARP DRIVE developer task runner.
///
/// Run with `cargo xtask <command>`.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// The task to run.
    #[command(subcommand)]
    command: Task,
}

/// Available xtask commands.
#[derive(Debug, Subcommand)]
enum Task {
    /// Install macFUSE via Homebrew (macOS only; no-op on other platforms).
    InstallDeps,

    /// Build and mount the WARP DRIVE FUSE filesystem.
    ///
    /// Blocks in the foreground while the filesystem is mounted.
    /// Unmount from another terminal with `cargo xtask unmount --path <dir>`,
    /// or press Ctrl-C to unmount and exit.
    Mount {
        /// Existing directory to use as the mount point.
        #[arg(long, short)]
        path: PathBuf,

        /// Runtime back-end to mount.
        #[arg(long, value_enum, default_value = "in-memory")]
        runtime: Runtime,

        /// Gate to mount. Defaults to G1 for in-memory, G2a for echo-rlib.
        #[arg(long, value_enum)]
        gate: Option<Gate>,
    },

    /// Unmount the WARP DRIVE FUSE filesystem.
    Unmount {
        /// Mount point to unmount.
        #[arg(long, short)]
        path: PathBuf,
    },

    /// Run the selected acceptance test.
    ///
    /// For `in-memory`, equivalent to:
    ///   docker build -t warp-drive-g1 .
    ///   docker run --rm --device /dev/fuse --cap-add SYS_ADMIN \
    ///     --security-opt apparmor=unconfined warp-drive-g1
    Acceptance {
        /// Docker image tag to use.
        #[arg(long, default_value = "warp-drive-g1")]
        tag: String,

        /// Gate to test.
        #[arg(long, value_enum)]
        gate: Option<Gate>,

        /// Runtime back-end to test.
        #[arg(long, value_enum, default_value = "in-memory")]
        runtime: Runtime,
    },
}

/// Runtime back-ends known to developer tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Runtime {
    /// Hardcoded in-memory fixture tree. G1 gate target.
    #[value(name = "in-memory")]
    InMemory,
    /// Embedded Echo rlib backend for local Echo gates.
    #[value(name = "echo-rlib")]
    EchoRlib,
}

/// Acceptance gates known to developer tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Gate {
    /// POSIX translation over the in-memory fixture tree.
    #[value(name = "g1")]
    G1,
    /// Echo coordinate metadata mount.
    #[value(name = "g2a")]
    G2a,
    /// First Echo-projected regular-file bytes.
    #[value(name = "g2b")]
    G2b,
    /// Live `/.warp/` diagnostics and operation counters.
    #[value(name = "g3")]
    G3,
}

impl Gate {
    const fn as_str(self) -> &'static str {
        match self {
            Self::G1 => "g1",
            Self::G2a => "g2a",
            Self::G2b => "g2b",
            Self::G3 => "g3",
        }
    }
}

/// Resolve the effective gate for a `(Runtime, Option<Gate>)` request.
///
/// All ten combinations (2 runtimes × {none, G1, G2a, G2b, G3}) are
/// enumerated explicitly below — an invalid combination is a matched error,
/// never a silent wildcard fallback. Shared by `mount` and `acceptance` so
/// the two commands can never disagree about what's valid.
const fn resolve_gate(runtime: Runtime, gate: Option<Gate>) -> Result<Gate, GateMismatch> {
    match (runtime, gate) {
        (Runtime::InMemory, None) => Ok(Gate::G1),
        (Runtime::InMemory, Some(Gate::G1)) => Ok(Gate::G1),
        (Runtime::InMemory, Some(Gate::G3)) => Ok(Gate::G3),
        (Runtime::InMemory, Some(Gate::G2a)) => Err(GateMismatch {
            runtime: Runtime::InMemory,
            gate: Gate::G2a,
        }),
        (Runtime::InMemory, Some(Gate::G2b)) => Err(GateMismatch {
            runtime: Runtime::InMemory,
            gate: Gate::G2b,
        }),
        (Runtime::EchoRlib, None) => Ok(Gate::G2a),
        (Runtime::EchoRlib, Some(Gate::G2a)) => Ok(Gate::G2a),
        (Runtime::EchoRlib, Some(Gate::G2b)) => Ok(Gate::G2b),
        (Runtime::EchoRlib, Some(Gate::G3)) => Ok(Gate::G3),
        (Runtime::EchoRlib, Some(Gate::G1)) => Err(GateMismatch {
            runtime: Runtime::EchoRlib,
            gate: Gate::G1,
        }),
    }
}

/// An explicitly-rejected `(Runtime, Gate)` combination.
#[derive(Debug, PartialEq, Eq)]
struct GateMismatch {
    runtime: Runtime,
    gate: Gate,
}

impl GateMismatch {
    fn into_message(self) -> String {
        let alternative = match self.runtime {
            Runtime::InMemory => "echo-rlib",
            Runtime::EchoRlib => "in-memory",
        };
        format!(
            "runtime {:?} does not support gate {}; use --runtime {alternative}",
            self.runtime,
            self.gate.as_str()
        )
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Task::InstallDeps => install_deps(),
        Task::Mount {
            path,
            runtime,
            gate,
        } => mount(&path, runtime, gate),
        Task::Unmount { path } => unmount(&path),
        Task::Acceptance { tag, gate, runtime } => acceptance(&tag, runtime, gate),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask error: {e}");
            ExitCode::FAILURE
        }
    }
}

// ── install-deps ─────────────────────────────────────────────────────────────

fn install_deps() -> Result<(), String> {
    install_deps_impl()
}

#[cfg(target_os = "macos")]
fn install_deps_impl() -> Result<(), String> {
    println!("Installing macFUSE via Homebrew...");
    run("brew", &["install", "--cask", "macfuse"])
}

#[cfg(not(target_os = "macos"))]
fn install_deps_impl() -> Result<(), String> {
    println!("install-deps: nothing to do on this platform (macFUSE is macOS-only).");
    Ok(())
}

// ── mount ────────────────────────────────────────────────────────────────────

fn mount(path: &Path, runtime: Runtime, gate: Option<Gate>) -> Result<(), String> {
    let resolved_gate = resolve_gate(runtime, gate).map_err(GateMismatch::into_message)?;
    let p = path_str(path)?;
    println!(
        "Mounting WARP DRIVE at {p} with {runtime:?} gate {} (blocks until unmounted — Ctrl-C to stop)...",
        resolved_gate.as_str()
    );
    match runtime {
        Runtime::InMemory => run(
            "cargo",
            &[
                "run",
                "--package",
                "warp-drive-fuse",
                "--",
                "--gate",
                resolved_gate.as_str(),
                "--mount",
                p,
            ],
        ),
        Runtime::EchoRlib => run(
            "cargo",
            &[
                "run",
                "--manifest-path",
                "crates/warp-drive-fuse-echo/Cargo.toml",
                "--target-dir",
                "target/echo-rlib",
                "--",
                "--runtime",
                "echo-rlib",
                "--gate",
                resolved_gate.as_str(),
                "--mount",
                p,
            ],
        ),
    }
}

// ── unmount ──────────────────────────────────────────────────────────────────

fn unmount(path: &Path) -> Result<(), String> {
    unmount_impl(path)
}

#[cfg(target_os = "macos")]
fn unmount_impl(path: &Path) -> Result<(), String> {
    let p = path_str(path)?;
    println!("Unmounting {p} (macOS)...");
    run("umount", &[p])
}

#[cfg(target_os = "linux")]
fn unmount_impl(path: &Path) -> Result<(), String> {
    let p = path_str(path)?;
    println!("Unmounting {p} (Linux)...");
    // fuse3 ships fusermount3; fuse2 ships fusermount — try both.
    run("fusermount3", &["-u", p]).or_else(|_| run("fusermount", &["-u", p]))
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn unmount_impl(_path: &Path) -> Result<(), String> {
    Err("unmount is not supported on this platform".to_owned())
}

// ── acceptance ───────────────────────────────────────────────────────────────

fn acceptance(tag: &str, runtime: Runtime, gate: Option<Gate>) -> Result<(), String> {
    let resolved = resolve_gate(runtime, gate).map_err(GateMismatch::into_message)?;
    match (runtime, resolved) {
        (Runtime::InMemory, Gate::G1) => acceptance_in_memory(tag),
        (Runtime::InMemory, Gate::G3) => acceptance_in_memory_g3(tag),
        (Runtime::EchoRlib, Gate::G2a | Gate::G2b | Gate::G3) => acceptance_echo_rlib(resolved),
        (runtime, gate) => Err(format!(
            "internal error: resolve_gate produced an unexpected combination runtime={runtime:?} gate={}",
            gate.as_str()
        )),
    }
}

fn acceptance_in_memory(tag: &str) -> Result<(), String> {
    println!("Building Docker image `{tag}`...");
    run("docker", &["build", "-t", tag, "."])?;
    println!("Running G1 acceptance test in Docker...");
    run(
        "docker",
        &[
            "run",
            "--rm",
            "--device",
            "/dev/fuse",
            "--cap-add",
            "SYS_ADMIN",
            "--security-opt",
            "apparmor=unconfined",
            tag,
        ],
    )
}

fn acceptance_in_memory_g3(tag: &str) -> Result<(), String> {
    println!("Building Docker image `{tag}`...");
    run("docker", &["build", "-t", tag, "."])?;
    println!("Running G3 in-memory acceptance test in Docker...");
    run(
        "docker",
        &[
            "run",
            "--rm",
            "--device",
            "/dev/fuse",
            "--cap-add",
            "SYS_ADMIN",
            "--security-opt",
            "apparmor=unconfined",
            tag,
            "bash",
            "scripts/acceptance-g3.sh",
        ],
    )
}

fn acceptance_echo_rlib(gate: Gate) -> Result<(), String> {
    let script = match gate {
        Gate::G1 => {
            return Err(
                "runtime echo-rlib does not support gate g1; use --runtime in-memory".to_owned(),
            );
        }
        Gate::G2a => "scripts/acceptance-g2.sh",
        Gate::G2b => "scripts/acceptance-g2b.sh",
        Gate::G3 => "scripts/acceptance-g3-echo.sh",
    };

    if env::var_os("WARP_DRIVE_ACCEPTANCE_IN_CONTAINER").is_none() {
        return acceptance_echo_rlib_copyin_docker(gate);
    }
    assert_sanitized_container_checkout()?;

    println!("Building local-only warp-drive-fuse Echo binary...");
    run(
        "cargo",
        &[
            "build",
            "--manifest-path",
            "crates/warp-drive-fuse-echo/Cargo.toml",
            "--target-dir",
            "target/echo-rlib",
        ],
    )?;
    println!("Running {} echo-rlib acceptance script...", gate.as_str());
    let target_debug = env::current_dir()
        .map_err(|e| format!("failed to read current directory: {e}"))?
        .join("target")
        .join("echo-rlib")
        .join("debug");
    let path = prepend_path(target_debug)?;
    // Always invoke the script through bash explicitly rather than executing
    // it directly — no gate script (new or existing) should depend on its
    // Git-tracked executable bit surviving a checkout/copy.
    run_with_path("bash", &[script], &path)
}

fn assert_sanitized_container_checkout() -> Result<(), String> {
    let warp_root =
        env::current_dir().map_err(|e| format!("failed to read current directory: {e}"))?;
    let repo_parent = warp_root
        .parent()
        .ok_or_else(|| format!("repo path `{}` has no parent", warp_root.display()))?;
    let echo_root = repo_parent.join("echo-warp-drive");

    println!("Copy-in acceptance isolation:");
    let forbidden = [
        warp_root.join(".git"),
        warp_root.join(".gitmodules"),
        echo_root.join(".git"),
        echo_root.join(".gitmodules"),
    ];
    for path in forbidden {
        if path.exists() {
            return Err(format!(
                "unsafe acceptance checkout: `{}` exists; Echo acceptance must run from sanitized copy-in Docker source, not a live Git checkout",
                path.display()
            ));
        }
    }
    if env::var_os("GIT_DIR").is_some() || env::var_os("GIT_WORK_TREE").is_some() {
        return Err("unsafe acceptance environment: GIT_DIR/GIT_WORK_TREE is set".to_owned());
    }
    println!("  PASS no git metadata in copied repos");
    Ok(())
}

fn acceptance_echo_rlib_copyin_docker(gate: Gate) -> Result<(), String> {
    let repo_root = env::current_dir().map_err(|e| format!("failed to read current dir: {e}"))?;
    let repo_parent = repo_root
        .parent()
        .ok_or_else(|| format!("repo path `{}` has no parent", repo_root.display()))?;
    let echo_root = repo_parent.join("echo-warp-drive");
    if !echo_root.join("crates").join("warp-wasm").exists() {
        return Err(format!(
            "expected sibling Echo checkout at `{}`",
            echo_root.display()
        ));
    }

    let stage = copyin_stage_dir(gate)?;
    let image_tag = copyin_image_tag(gate)?;
    let result = (|| {
        let warp_copy = stage.join("warp-drive");
        let echo_copy = stage.join("echo-warp-drive");
        copy_repo_for_docker(&repo_root, &warp_copy)?;
        copy_repo_for_docker(&echo_root, &echo_copy)?;
        assert_sanitized_repo_copy(&warp_copy)?;
        assert_sanitized_repo_copy(&echo_copy)?;
        write_copyin_dockerfile(&stage)?;

        println!("Building Docker image `{image_tag}` from sanitized copies (no bind mounts)...");
        let dockerfile = stage.join("Dockerfile.echo-acceptance");
        let dockerfile_arg = path_str(&dockerfile)?;
        let stage_arg = path_str(&stage)?;
        run(
            "docker",
            &[
                "build",
                "--no-cache",
                "-t",
                &image_tag,
                "-f",
                dockerfile_arg,
                stage_arg,
            ],
        )?;

        println!(
            "Running {} echo-rlib acceptance in copy-in Docker container...",
            gate.as_str()
        );
        run(
            "docker",
            &[
                "run",
                "--rm",
                "--device",
                "/dev/fuse",
                "--cap-add",
                "SYS_ADMIN",
                "--security-opt",
                "apparmor=unconfined",
                &image_tag,
                "/usr/local/cargo/bin/cargo",
                "xtask",
                "acceptance",
                "--gate",
                gate.as_str(),
                "--runtime",
                "echo-rlib",
            ],
        )
    })();

    let stage_cleanup = fs::remove_dir_all(&stage).map_err(|e| {
        format!(
            "failed to remove copy-in staging dir `{}`: {e}",
            stage.display()
        )
    });
    let _ = run("docker", &["rmi", "-f", &image_tag]);

    match (result, stage_cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(err), Ok(())) => Err(err),
        (Ok(()), Err(cleanup_err)) => Err(cleanup_err),
        (Err(err), Err(cleanup_err)) => Err(format!("{err}; additionally, {cleanup_err}")),
    }
}

fn copyin_stage_dir(gate: Gate) -> Result<PathBuf, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system clock error: {e}"))?
        .as_millis();
    let stage = env::temp_dir().join(format!(
        "warp-drive-{}-copyin-{}-{now}",
        gate.as_str(),
        std::process::id()
    ));
    fs::create_dir_all(&stage).map_err(|e| {
        format!(
            "failed to create copy-in staging dir `{}`: {e}",
            stage.display()
        )
    })?;
    Ok(stage)
}

fn copyin_image_tag(gate: Gate) -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system clock error: {e}"))?
        .as_millis();
    Ok(format!(
        "warp-drive-{}-echo-copyin-{}-{now}",
        gate.as_str(),
        std::process::id()
    ))
}

fn copy_repo_for_docker(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| {
        format!(
            "failed to create sanitized repo copy `{}`: {e}",
            destination.display()
        )
    })?;
    let source_arg = format!("{}/", path_str(source)?);
    let destination_arg = path_str(destination)?;
    let status = Command::new("rsync")
        .arg("-a")
        .arg("--delete")
        .arg("--exclude")
        .arg(".git")
        .arg("--exclude")
        .arg(".gitmodules")
        .arg("--exclude")
        .arg("target")
        .arg("--exclude")
        .arg(".DS_Store")
        .arg(&source_arg)
        .arg(destination_arg)
        .status()
        .map_err(|e| format!("failed to spawn `rsync`: {e}"))?;

    if status.success() {
        return Ok(());
    }

    let code = status.code().unwrap_or(-1);
    Err(format!("`rsync` exited with status {code}"))
}

fn assert_sanitized_repo_copy(repo: &Path) -> Result<(), String> {
    for forbidden in [repo.join(".git"), repo.join(".gitmodules")] {
        if forbidden.exists() {
            return Err(format!(
                "unsafe copy-in staging tree: `{}` exists before Docker build context creation",
                forbidden.display()
            ));
        }
    }
    Ok(())
}

fn write_copyin_dockerfile(stage: &Path) -> Result<(), String> {
    let dockerfile = r#"FROM rust:1.90
ENV DEBIAN_FRONTEND=noninteractive
ENV PATH="/usr/local/cargo/bin:${PATH}"
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        fuse3 \
        libssl-dev \
        pkg-config \
        ripgrep \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /work
COPY warp-drive /work/warp-drive
COPY echo-warp-drive /work/echo-warp-drive
# Acceptance runs against copied source trees only. Strip VCS metadata so the
# container has no remote to fetch from, push to, or mutate.
RUN rm -rf \
        /work/warp-drive/.git \
        /work/warp-drive/.gitmodules \
        /work/echo-warp-drive/.git \
        /work/echo-warp-drive/.gitmodules \
    && test ! -d /work/warp-drive/.git \
    && test ! -e /work/warp-drive/.gitmodules \
    && test ! -d /work/echo-warp-drive/.git \
    && test ! -e /work/echo-warp-drive/.gitmodules
WORKDIR /work/warp-drive
ENV WARP_DRIVE_ACCEPTANCE_IN_CONTAINER=1
"#;
    let path = stage.join("Dockerfile.echo-acceptance");
    fs::write(&path, dockerfile)
        .map_err(|e| format!("failed to write Dockerfile `{}`: {e}", path.display()))
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Borrow `path` as a `&str`, returning an error if it contains non-UTF-8 bytes.
fn path_str(path: &Path) -> Result<&str, String> {
    path.to_str()
        .ok_or_else(|| format!("path `{}` contains non-UTF-8 bytes", path.display()))
}

/// Spawn `program args…`, inherit stdio, wait for exit, and return an error if
/// the process exits with a non-zero status.
fn run(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("failed to spawn `{program}`: {e}"))?;

    if status.success() {
        return Ok(());
    }

    let code = status.code().unwrap_or(-1);
    Err(format!("`{program}` exited with status {code}"))
}

fn run_with_path(program: &str, args: &[&str], path: &std::ffi::OsStr) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .env("PATH", path)
        .status()
        .map_err(|e| format!("failed to spawn `{program}`: {e}"))?;

    if status.success() {
        return Ok(());
    }

    let code = status.code().unwrap_or(-1);
    Err(format!("`{program}` exited with status {code}"))
}

fn prepend_path(first: PathBuf) -> Result<std::ffi::OsString, String> {
    let existing = env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![first];
    paths.extend(env::split_paths(&existing));
    env::join_paths(paths).map_err(|e| format!("failed to build PATH: {e}"))
}

#[cfg(test)]
mod tests {
    use super::{Gate, Runtime, resolve_gate};

    #[test]
    fn resolve_gate_covers_all_ten_combinations() {
        // in-memory
        assert_eq!(resolve_gate(Runtime::InMemory, None), Ok(Gate::G1));
        assert_eq!(
            resolve_gate(Runtime::InMemory, Some(Gate::G1)),
            Ok(Gate::G1)
        );
        assert!(resolve_gate(Runtime::InMemory, Some(Gate::G2a)).is_err());
        assert!(resolve_gate(Runtime::InMemory, Some(Gate::G2b)).is_err());
        assert_eq!(
            resolve_gate(Runtime::InMemory, Some(Gate::G3)),
            Ok(Gate::G3)
        );

        // echo-rlib
        assert_eq!(resolve_gate(Runtime::EchoRlib, None), Ok(Gate::G2a));
        assert!(resolve_gate(Runtime::EchoRlib, Some(Gate::G1)).is_err());
        assert_eq!(
            resolve_gate(Runtime::EchoRlib, Some(Gate::G2a)),
            Ok(Gate::G2a)
        );
        assert_eq!(
            resolve_gate(Runtime::EchoRlib, Some(Gate::G2b)),
            Ok(Gate::G2b)
        );
        assert_eq!(
            resolve_gate(Runtime::EchoRlib, Some(Gate::G3)),
            Ok(Gate::G3)
        );
    }
}
