//! Generates every fixture plan and instantiates it with nix

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use balsa_flakegen::plan::{
    BalsaInstallPlan, DesktopChoice, Filesystem, KernelChoice, PartitionScheme, SwapMode,
};

fn plan_paths() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("test-plans");
    let mut paths: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "toml"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no fixtures in {}", dir.display());
    paths
}

fn load(path: &Path) -> BalsaInstallPlan {
    balsa_flakegen::load_plan(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.file_name().unwrap().to_string_lossy()))
}

const IMAGE: &str = "docker.io/nixos/nix:latest";
/// Named volume so /nix survives between runs; nixpkgs is fetched once.
const STORE_VOLUME: &str = "balsa-nix-store";

/// A rootless podman container holding the throwaway nix store.
struct NixSandbox {
    name: String,
}

impl NixSandbox {
    fn start(workdir: &Path) -> NixSandbox {
        let name = format!("balsa-flakegen-check-{}", std::process::id());
        let out = Command::new("podman")
            .args([
                "run",
                "-d",
                "--rm",
                "--name",
                &name,
                "-v",
                &format!("{STORE_VOLUME}:/nix"),
                "-v",
                &format!("{}:/work:Z", workdir.display()),
                IMAGE,
                "sleep",
                "infinity",
            ])
            .output()
            .unwrap_or_else(|e| panic!("could not run podman: {e}"));
        assert!(
            out.status.success(),
            "could not start the nix sandbox:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        NixSandbox { name }
    }

    /// Dry-run builds a fixture's system; the fixture name is also its hostname and flake attribute.
    fn instantiate(&self, fixture: &str) -> Result<(), String> {
        let out = Command::new("podman")
            .args([
                "exec",
                "-w",
                &format!("/work/{fixture}"),
                &self.name,
                "nix",
                "--extra-experimental-features",
                "nix-command flakes",
                "build",
                "--dry-run",
                "--no-link",
                &format!(".#nixosConfigurations.\"{fixture}\".config.system.build.toplevel"),
            ])
            .output()
            .map_err(|e| format!("could not run podman: {e}"))?;
        if out.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
}

impl Drop for NixSandbox {
    fn drop(&mut self) {
        let _ = Command::new("podman")
            .args(["rm", "-f", &self.name])
            .output();
    }
}

#[test]
fn every_fixture_generates_and_evaluates() {
    let paths = plan_paths();
    let workdir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("nix-check");
    fs::create_dir_all(&workdir).unwrap();

    let mut names = Vec::new();
    for path in &paths {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let plan = load(path);
        let result = balsa_flakegen::generate(&plan)
            .unwrap_or_else(|e| panic!("{name}: generation failed: {e}"));
        result.config.write_to(&workdir.join(&name)).unwrap();
        names.push(name);
    }

    let sandbox = NixSandbox::start(&workdir);
    let failures: Vec<String> = thread::scope(|scope| {
        let handles: Vec<_> = names
            .chunks(names.len().div_ceil(8))
            .map(|chunk| {
                let sandbox = &sandbox;
                scope.spawn(move || {
                    chunk
                        .iter()
                        .filter_map(|name| {
                            sandbox
                                .instantiate(name)
                                .err()
                                .map(|e| format!("{name}: {e}"))
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });

    assert!(
        failures.is_empty(),
        "{} of {} fixtures failed:\n\n{}",
        failures.len(),
        paths.len(),
        failures.join("\n\n")
    );
}

#[test]
fn fixtures_cover_every_desktop() {
    let seen: Vec<DesktopChoice> = plan_paths().iter().map(|p| load(p).desktop).collect();
    let missing: Vec<_> = DesktopChoice::ALL
        .iter()
        .filter(|d| !seen.contains(d))
        .collect();
    assert!(missing.is_empty(), "no fixture covers {missing:?}");
}

#[test]
fn fixtures_cover_every_kernel_filesystem_swap_and_encryption() {
    let plans: Vec<_> = plan_paths().iter().map(|p| load(p)).collect();

    for want in ["default", "lts", "zen", "xanmod"] {
        let found = plans.iter().any(|p| match &p.kernel {
            KernelChoice::Default => want == "default",
            KernelChoice::Lts => want == "lts",
            KernelChoice::Zen => want == "zen",
            KernelChoice::Xanmod => want == "xanmod",
        });
        assert!(found, "no fixture uses kernel {want}");
    }

    for fs in balsa_flakegen::plan::Filesystem::ALL {
        assert!(
            plans.iter().any(|p| p.disk.filesystem == fs),
            "no fixture uses {fs:?}"
        );
    }

    for want in ["none", "partition", "zram"] {
        let found = plans.iter().any(|p| {
            matches!(
                (&p.disk.swap, want),
                (SwapMode::None, "none")
                    | (SwapMode::Partition { .. }, "partition")
                    | (SwapMode::Zram, "zram")
            )
        });
        assert!(found, "no fixture uses swap mode {want}");
    }

    for want in [true, false] {
        assert!(
            plans.iter().any(|p| p.boot.legacy_bios == want),
            "no fixture has legacy_bios = {want}"
        );
    }

    assert!(
        plans.iter().any(|p| p.disk.encryption.is_some()),
        "no encrypted fixture"
    );
    assert!(
        plans.iter().any(|p| p.disk.encryption.is_none()),
        "no plaintext fixture"
    );
}

#[test]
fn only_manual_disko_and_zfs_warn() {
    for path in plan_paths() {
        let plan = load(&path);
        let expected = matches!(plan.disk.scheme, PartitionScheme::Manual { .. })
            || plan.disk.filesystem == Filesystem::Zfs;
        let warned = !balsa_flakegen::generate(&plan).unwrap().warnings.is_empty();
        assert_eq!(
            warned,
            expected,
            "{}: warned = {warned}, expected {expected}",
            path.file_stem().unwrap().to_string_lossy()
        );
    }
}

#[test]
fn zfs_requires_a_well_formed_host_id() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("test-plans/plasma-lts-zfs-encrypted.toml");
    let mut plan = load(&path);
    assert!(plan.validate().is_ok());

    plan.network.host_id = None;
    let errs = plan.validate().unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("needs network.host_id")),
        "{errs:?}"
    );

    plan.network.host_id = Some("not-hex!".to_string());
    let errs = plan.validate().unwrap_err();
    assert!(errs.iter().any(|e| e.contains("8 hex digits")), "{errs:?}");
}
