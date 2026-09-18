use tera::Context;

use crate::Error;
use crate::generate::{engine, tidy};
use crate::plan::DiskoConfig;

/// Renders disko-config.nix alone; `legacy_bios` adds GRUB's BIOS partition, `esp_mount` places the ESP.
pub fn generate_disko_config(
    disk: &DiskoConfig,
    legacy_bios: bool,
    esp_mount: &str,
) -> Result<String, Error> {
    disk.validate().map_err(Error::Validation)?;

    let mut ctx = Context::new();
    ctx.insert("disk", disk);
    ctx.insert("legacy_bios", &legacy_bios);
    ctx.insert("esp_mount", esp_mount);
    let body = engine()
        .map_err(Error::Template)?
        .render("disko-config.nix.j2", &ctx)
        .map_err(Error::Template)?;
    Ok(tidy(&body))
}
