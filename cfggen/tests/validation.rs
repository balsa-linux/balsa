//! Generates every fixture plan and instantiates it with nix

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use balsa_flakegen::plan::{BalsaInstallPlan, BootLoader, DesktopChoice, KernelChoice, SwapMode};

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

    /// Instantiates the generated system without building it. If fails, returns output
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
                ".#nixosConfigurations.test.config.system.build.toplevel",
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

    for want in [
        "default",
        "lts",
        "zen",
        "xanmod",
        "cachy-os-latest",
        "cachy-os-lts",
        "cachy-os-bore",
    ] {
        let found = plans.iter().any(|p| match &p.kernel {
            KernelChoice::Default => want == "default",
            KernelChoice::Lts => want == "lts",
            KernelChoice::Zen => want == "zen",
            KernelChoice::Xanmod => want == "xanmod",
            KernelChoice::CachyOs { variant, .. } => {
                want == format!("cachy-os-{}", variant.attr_suffix())
            }
        });
        assert!(found, "no fixture uses kernel {want}");
    }

    for fs in balsa_flakegen::plan::Filesystem::ALL {
        assert!(
            plans.iter().any(|p| p.disk.filesystem == fs),
            "no fixture uses {fs:?}"
        );
    }

    for want in ["none", "partition", "file", "zram"] {
        let found = plans.iter().any(|p| {
            matches!(
                (&p.disk.swap, want),
                (SwapMode::None, "none")
                    | (SwapMode::Partition { .. }, "partition")
                    | (SwapMode::File { .. }, "file")
                    | (SwapMode::Zram, "zram")
            )
        });
        assert!(found, "no fixture uses swap mode {want}");
    }

    for loader in BootLoader::ALL {
        assert!(
            plans.iter().any(|p| p.boot.effective_loader() == loader),
            "no fixture boots with {loader:?}"
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
fn cachyos_fixtures_warn_and_others_stay_quiet() {
    let mut compiles = Vec::new();
    let mut cachyos = Vec::new();
    let mut warns_at_all = Vec::new();
    for path in plan_paths() {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let plan = load(&path);
        let result = balsa_flakegen::generate(&plan).unwrap();
        if matches!(plan.kernel, KernelChoice::CachyOs { .. }) {
            cachyos.push(name.clone());
        }
        if !result.warnings.is_empty() {
            warns_at_all.push(name.clone());
        }
        if result
            .warnings
            .iter()
            .any(|w| w.contains("builds from source"))
        {
            compiles.push(name);
        }
    }

    assert_eq!(compiles, vec!["gaming-cachyos-hyprland".to_string()]);
    // Every CachyOS pick warns; the only other warning paths are manual disko and ZFS.
    for name in &cachyos {
        assert!(warns_at_all.contains(name), "{name} produced no warning");
    }
    let mut expected = cachyos;
    expected.push("plasma-manual-scheme-ext4".to_string());
    expected.push("plasma-lts-zfs-encrypted".to_string());
    expected.sort();
    warns_at_all.sort();
    assert_eq!(warns_at_all, expected);
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
