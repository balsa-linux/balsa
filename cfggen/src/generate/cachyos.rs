use serde::Serialize;

use crate::plan::{CachyNixpkgsPolicy, CachyVariant, CachyosCache};

/// Flake pin, overlay and substituter must agree or the user compiles a kernel
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CachyosSpec {
    /// Flake input URL, pinned to the `release` branch per upstream's README.
    pub flake_url: String,
    /// `pinned` or `default`, the two overlays upstream exposes.
    pub overlay_attr: String,
    pub packages_attr: String,
    pub substituter: String,
    pub trusted_public_key: String,
}

pub const FLAKE_URL: &str = "github:xddxdd/nix-cachyos-kernel/release";

/// Returns the spec plus any warnings for `GenerationResult.warnings`.
pub fn spec(
    variant: CachyVariant,
    policy: CachyNixpkgsPolicy,
    cache: &CachyosCache,
) -> (CachyosSpec, Vec<String>) {
    let mut warnings = Vec::new();

    // Lands in the target's nix.conf, not the one the installer builds with
    warnings.push(format!(
        "CachyOS kernel ({}) needs {} configured in the environment running the \
         install, not only in the generated system, or the install itself compiles \
         the kernel.",
        variant.attr_suffix(),
        cache.substituter
    ));

    if policy == CachyNixpkgsPolicy::SystemNixpkgs {
        warnings.push(format!(
            "CachyOS kernel ({}) uses the `default` overlay, so it is built against \
             the system nixpkgs rather than the revision {} was built with. The \
             kernel builds from source - expect a long first build.",
            variant.attr_suffix(),
            cache.substituter
        ));
    }

    let spec = CachyosSpec {
        flake_url: FLAKE_URL.to_string(),
        overlay_attr: match policy {
            CachyNixpkgsPolicy::Pinned => "pinned".to_string(),
            CachyNixpkgsPolicy::SystemNixpkgs => "default".to_string(),
        },
        packages_attr: format!(
            "pkgs.cachyosKernels.linuxPackages-cachyos-{}",
            variant.attr_suffix()
        ),
        substituter: cache.substituter.clone(),
        trusted_public_key: cache.trusted_public_key.clone(),
    };
    (spec, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_nixpkgs_overlay_warns_about_compiling() {
        let (spec, warnings) = spec(
            CachyVariant::Bore,
            CachyNixpkgsPolicy::SystemNixpkgs,
            &CachyosCache::default(),
        );
        assert_eq!(spec.overlay_attr, "default");
        assert_eq!(warnings.len(), 2);
        assert!(warnings[1].contains("builds from source"));
    }

    #[test]
    fn pinned_overlay_only_warns_about_the_install_environment() {
        let (spec, warnings) = spec(
            CachyVariant::Latest,
            CachyNixpkgsPolicy::Pinned,
            &CachyosCache::default(),
        );
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("running the install"));
        assert_eq!(spec.overlay_attr, "pinned");
        assert!(spec.flake_url.ends_with("/release"));
        assert_eq!(
            spec.packages_attr,
            "pkgs.cachyosKernels.linuxPackages-cachyos-latest"
        );
    }

    #[test]
    fn variant_attrs_match_upstream_names() {
        assert_eq!(CachyVariant::Lts.attr_suffix(), "lts");
        assert_eq!(CachyVariant::Bore.attr_suffix(), "bore");
    }
}
