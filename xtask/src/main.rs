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
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

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
    /// Embedded Echo rlib coordinate metadata over G1 fixture bytes. G2a target.
    #[value(name = "echo-rlib")]
    EchoRlib,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Task::InstallDeps => install_deps(),
        Task::Mount { path, runtime } => mount(&path, runtime),
        Task::Unmount { path } => unmount(&path),
        Task::Acceptance { tag, runtime } => acceptance(&tag, runtime),
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

fn mount(path: &Path, runtime: Runtime) -> Result<(), String> {
    let p = path_str(path)?;
    println!(
        "Mounting WARP DRIVE at {p} with {runtime:?} (blocks until unmounted — Ctrl-C to stop)..."
    );
    match runtime {
        Runtime::InMemory => run(
            "cargo",
            &[
                "run",
                "--package",
                "warp-drive-fuse",
                "--",
                "--runtime",
                "in-memory",
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

fn acceptance(tag: &str, runtime: Runtime) -> Result<(), String> {
    match runtime {
        Runtime::InMemory => acceptance_in_memory(tag),
        Runtime::EchoRlib => acceptance_echo_rlib(),
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

fn acceptance_echo_rlib() -> Result<(), String> {
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
    println!("Running G2a echo-rlib acceptance script...");
    let target_debug = env::current_dir()
        .map_err(|e| format!("failed to read current directory: {e}"))?
        .join("target")
        .join("echo-rlib")
        .join("debug");
    let path = prepend_path(target_debug)?;
    run_with_path("scripts/acceptance-g2.sh", &[], &path)
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
