pub mod cachyos;

use std::collections::BTreeMap;

use tera::{Context, Kwargs, State, Tera};

use crate::plan::{BalsaInstallPlan, Filesystem, Gpu, KernelChoice, PartitionScheme, SwapMode};
use crate::{Error, GeneratedConfig, GenerationResult};

// Fragments first: `include` is resolved when a template is added.
const TEMPLATES: &[(&str, &str)] = &[
    (
        "cachyos.nix.j2",
        include_str!("../templates/cachyos.nix.j2"),
    ),
    (
        "disko-fs.nix.j2",
        include_str!("../templates/disko-fs.nix.j2"),
    ),
    ("locale.nix.j2", include_str!("../templates/locale.nix.j2")),
    ("boot.nix.j2", include_str!("../templates/boot.nix.j2")),
    ("kernel.nix.j2", include_str!("../templates/kernel.nix.j2")),
    (
        "hardware.nix.j2",
        include_str!("../templates/hardware.nix.j2"),
    ),
    (
        "network.nix.j2",
        include_str!("../templates/network.nix.j2"),
    ),
    (
        "desktop.nix.j2",
        include_str!("../templates/desktop.nix.j2"),
    ),
    ("login.nix.j2", include_str!("../templates/login.nix.j2")),
    (
        "accounts.nix.j2",
        include_str!("../templates/accounts.nix.j2"),
    ),
    ("tuning.nix.j2", include_str!("../templates/tuning.nix.j2")),
    ("swap.nix.j2", include_str!("../templates/swap.nix.j2")),
    ("shell.nix.j2", include_str!("../templates/shell.nix.j2")),
    ("flake.nix.j2", include_str!("../templates/flake.nix.j2")),
    (
        "configuration.nix.j2",
        include_str!("../templates/configuration.nix.j2"),
    ),
    (
        "hardware-configuration.nix.j2",
        include_str!("../templates/hardware-configuration.nix.j2"),
    ),
    (
        "disko-config.nix.j2",
        include_str!("../templates/disko-config.nix.j2"),
    ),
];

/// Escapes for a Nix double-quoted string, `${` included, so text stays text
fn nix_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace("${", "\\${")
        .replace('\n', "\\n")
}

pub(crate) fn engine() -> Result<Tera, tera::Error> {
    let mut tera = Tera::new();
    tera.register_filter("nix", |s: &str, _: Kwargs, _: &State| nix_escape(s));
    for (name, body) in TEMPLATES {
        tera.add_raw_template(name, body)?;
    }
    Ok(tera)
}

/// Fragment composition leaves ragged blank lines; diffing against real configs suffers
pub(crate) fn tidy(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut blank_run = 0;
    for line in s.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            blank_run += 1;
            if blank_run > 1 {
                continue;
            }
        } else {
            blank_run = 0;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub fn generate(plan: &BalsaInstallPlan) -> Result<GenerationResult, Error> {
    plan.validate().map_err(Error::Validation)?;

    let mut warnings = Vec::new();
    let mut ctx = Context::from_serialize(plan).map_err(Error::Template)?;

    if let KernelChoice::CachyOs {
        variant,
        nixpkgs_policy,
        cache,
    } = &plan.kernel
    {
        let (spec, mut w) = cachyos::spec(*variant, *nixpkgs_policy, cache);
        warnings.append(&mut w);
        ctx.insert("cachyos", &spec);
    }

    if let PartitionScheme::Manual { .. } = &plan.disk.scheme {
        warnings.push(
            "Manual partitioning: the disko devices block is passed through unchanged and \
             is only checked by nix evaluation."
                .to_string(),
        );
    }

    if plan.disk.filesystem == Filesystem::Zfs {
        warnings.push(
            "ZFS: run `zpool export zroot` after installing. The pool is created under the \
             installer's hostId, and the installed system will not force-import it at boot."
                .to_string(),
        );
    }

    let has_nvidia = plan
        .hardware
        .gpu
        .iter()
        .any(|g| matches!(g, Gpu::NvidiaProprietary | Gpu::NvidiaOpen));

    ctx.insert("effective_loader", &plan.boot.effective_loader());
    ctx.insert("login", &plan.login_manager());
    ctx.insert("desktop_is_tiling", &plan.desktop.is_tiling());
    ctx.insert("needs_xserver", &plan.desktop.needs_xserver());
    ctx.insert("has_nvidia", &has_nvidia);
    ctx.insert("nvidia_open", &plan.hardware.gpu.contains(&Gpu::NvidiaOpen));
    ctx.insert(
        "allow_unfree",
        &(has_nvidia || plan.hardware.firmware == crate::plan::FirmwareMode::All),
    );

    if let SwapMode::File { size_gib } = plan.disk.swap {
        ctx.insert("swapfile_mib", &(u64::from(size_gib) * 1024));
        ctx.insert(
            "swapfile_path",
            &match plan.disk.filesystem {
                Filesystem::Btrfs => "/swap/swapfile",
                Filesystem::Ext4 | Filesystem::Xfs => "/swapfile",
                Filesystem::Zfs => unreachable!("validate() rejects file swap on ZFS"),
            },
        );
    }

    let tera = engine().map_err(Error::Template)?;
    let mut files = BTreeMap::new();
    for (out, tpl) in [
        ("flake.nix", "flake.nix.j2"),
        ("configuration.nix", "configuration.nix.j2"),
        (
            "hardware-configuration.nix",
            "hardware-configuration.nix.j2",
        ),
    ] {
        let body = tera.render(tpl, &ctx).map_err(Error::Template)?;
        files.insert(out.to_string(), tidy(&body));
    }

    files.insert(
        "disko-config.nix".to_string(),
        crate::disko::generate_disko_config(
            &plan.disk,
            plan.boot.legacy_bios,
            &plan.boot.esp_mount,
        )?,
    );

    Ok(GenerationResult {
        config: GeneratedConfig { files },
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::nix_escape;

    #[test]
    fn escapes_what_breaks_nix_strings() {
        assert_eq!(nix_escape(r#"a"b"#), r#"a\"b"#);
        assert_eq!(nix_escape(r"c:\x"), r"c:\\x");
        assert_eq!(nix_escape("${pkgs.bash}"), "\\${pkgs.bash}");
        assert_eq!(nix_escape("a\nb"), "a\\nb");
    }
}
