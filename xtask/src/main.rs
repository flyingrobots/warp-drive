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
//! | `acceptance`   | Build Docker image and run the G1 gate acceptance test    |

// xtask is a developer CLI — printing to stdout/stderr is intentional.
#![allow(clippy::print_stdout, clippy::print_stderr)]

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
    },

    /// Unmount the WARP DRIVE FUSE filesystem.
    Unmount {
        /// Mount point to unmount.
        #[arg(long, short)]
        path: PathBuf,
    },

    /// Build the Docker acceptance image and run the G1 gate test.
    ///
    /// Equivalent to:
    ///   docker build -t warp-drive-g1 .
    ///   docker run --rm --device /dev/fuse --cap-add SYS_ADMIN warp-drive-g1
    Acceptance {
        /// Docker image tag to use.
        #[arg(long, default_value = "warp-drive-g1")]
        tag: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Task::InstallDeps => install_deps(),
        Task::Mount { path } => mount(&path),
        Task::Unmount { path } => unmount(&path),
        Task::Acceptance { tag } => acceptance(&tag),
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

fn mount(path: &Path) -> Result<(), String> {
    let p = path_str(path)?;
    println!("Mounting WARP DRIVE at {p} (blocks until unmounted — Ctrl-C to stop)...");
    run(
        "cargo",
        &["run", "--package", "warp-drive-fuse", "--", "--mount", p],
    )
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

fn acceptance(tag: &str) -> Result<(), String> {
    println!("Building Docker image `{tag}`...");
    run("docker", &["build", "-t", tag, "."])?;
    println!("Running G1 acceptance test in Docker...");
    run(
        "docker",
        &[
            "run", "--rm",
            "--device", "/dev/fuse",
            "--cap-add", "SYS_ADMIN",
            tag,
        ],
    )
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
