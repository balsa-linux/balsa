use serde::{Deserialize, Serialize};

/// The complete description of an install. Interface between UI and generator
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BalsaInstallPlan {
    pub locale: LocaleConfig,
    pub disk: DiskoConfig,
    pub boot: BootConfig,
    pub hardware: HardwareConfig,
    pub network: NetworkConfig,
    pub kernel: KernelChoice,
    pub desktop: DesktopChoice,
    /// None means default
    #[serde(default)]
    pub login_manager: Option<LoginManager>,
    pub accounts: AccountsConfig,
    pub tuning_profile: TuningProfile,
    #[serde(default = "default_nixpkgs_ref")]
    pub nixpkgs_ref: String,
    /// Balsa commit whose nixosModules the system imports; fixtures use None, having no pushed commit.
    #[serde(default)]
    pub balsa_ref: Option<String>,
    #[serde(default = "default_state_version")]
    pub state_version: String,
}

fn default_nixpkgs_ref() -> String {
    "nixos-unstable".to_string()
}

fn default_state_version() -> String {
    "25.05".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocaleConfig {
    pub locale: String,
    pub keymap: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiskoConfig {
    /// e.g. "/dev/sda", "/dev/nvme0n1"
    pub disk: String,
    pub scheme: PartitionScheme,
    pub filesystem: Filesystem,
    pub swap: SwapMode,
    /// `None` means no full-disk encryption
    #[serde(default)]
    pub encryption: Option<EncryptionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncryptionConfig {
    /// Device-mapper name, i.e. /dev/mapper/<luks_name>
    pub luks_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PartitionScheme {
    Guided,
    /// The body of disko.devices, written by the caller
    Manual {
        devices: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Filesystem {
    Btrfs,
    Ext4,
    Xfs,
    /// Single-disk pool named `zroot`. Needs `network.host_id`.
    Zfs,
}

impl Filesystem {
    pub const ALL: [Filesystem; 4] = [
        Filesystem::Btrfs,
        Filesystem::Ext4,
        Filesystem::Xfs,
        Filesystem::Zfs,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SwapMode {
    None,
    Partition { size_gib: u32 },
    File { size_gib: u32 },
    Zram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BootConfig {
    pub loader: BootLoader,
    #[serde(default)]
    pub dual_boot: bool,
    #[serde(default)]
    pub legacy_bios: bool,
    #[serde(default = "default_esp")]
    pub esp_mount: String,
}

fn default_esp() -> String {
    "/boot".to_string()
}

impl BootConfig {
    /// systemd-boot handles neither, so it falls back to GRUB; explicit picks stand
    pub fn effective_loader(&self) -> BootLoader {
        match self.loader {
            BootLoader::SystemdBoot if self.dual_boot || self.legacy_bios => BootLoader::Grub,
            other => other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BootLoader {
    SystemdBoot,
    Grub,
    /// Only offered for btrfs installs since Limine lets you reap the benefits of btrfs
    Limine,
}

impl BootLoader {
    pub const ALL: [BootLoader; 3] = [
        BootLoader::SystemdBoot,
        BootLoader::Grub,
        BootLoader::Limine,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HardwareConfig {
    pub arch: Arch,
    pub cpu: CpuVendor,
    pub gpu: Vec<Gpu>,
    pub firmware: FirmwareMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Arch {
    #[serde(rename = "x86_64-linux")]
    X86_64Linux,
    #[serde(rename = "aarch64-linux")]
    Aarch64Linux,
}

impl Arch {
    pub fn is_x64(self) -> bool {
        self == Arch::X86_64Linux
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CpuVendor {
    Intel,
    Amd,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Gpu {
    Amd,
    Intel,
    NvidiaProprietary,
    NvidiaOpen,
    Nouveau,
    Virtio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FirmwareMode {
    /// Redistributable blobs only
    Auto,
    /// Everything, including unfree blobs
    All,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    pub hostname: String,
    pub backend: NetworkBackend,
    /// `networking.hostId`, 8 hex digits. Required by ZFS.
    #[serde(default)]
    pub host_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkBackend {
    NetworkManager,
    SystemdNetworkd,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum KernelChoice {
    /// linuxPackages_latest
    Default,
    /// linuxPackages
    Lts,
    Zen,
    Xanmod,
    CachyOs {
        /// cachyos kernels have attributes that require extra configuration
        variant: CachyVariant,
        #[serde(default)]
        nixpkgs_policy: CachyNixpkgsPolicy,
        #[serde(default)]
        cache: CachyosCache,
    },
}

impl KernelChoice {
    pub fn is_x64_only(&self) -> bool {
        !matches!(self, KernelChoice::Default | KernelChoice::Lts)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CachyVariant {
    Latest,
    Lts,
    Bore, // based on latest but uses BORE scheduler
}

impl CachyVariant {
    /// Attribute suffix under pkgs.cachyosKernels.linuxPackages-cachyos-yaddayadda.
    pub fn attr_suffix(self) -> &'static str {
        match self {
            CachyVariant::Latest => "latest",
            CachyVariant::Lts => "lts",
            CachyVariant::Bore => "bore",
        }
    }
}

/// Which of upstream's two overlays to use
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CachyNixpkgsPolicy {
    /// Upstream's nixpkgs revision, the cache hits, but nixpkgs.config misses the kernel
    #[default]
    Pinned,
    /// balsa nixpkgs: nixpkgs.config applies, but the kernel may be compiled
    SystemNixpkgs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CachyosCache {
    pub substituter: String,
    pub trusted_public_key: String,
}

impl Default for CachyosCache {
    fn default() -> Self {
        // From the xddxdd/nix-cachyos-kernel README and its flake nixConfig
        Self {
            substituter: "https://xddxdd.cachix.org".to_string(),
            trusted_public_key: "xddxdd.cachix.org-1:ay1HJyNDYmlSwj5NXQG065C8LfoqqKaTNCyzeixGjf8="
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopChoice {
    Plasma,
    Gnome,
    Xfce,
    Cinnamon,
    Mate,
    Cosmic,
    Wayfire,
    Openbox,
    Budgie,
    Lxqt,
    Fluxbox,
    Enlightenment,
    Labwc,
    Pantheon,
    Niri,
    Mango,
    Hyprland,
    I3,
    Sway,
    Bspwm,
    Xmonad,
    Dwm,
}

impl DesktopChoice {
    /// every option as of right now
    pub const ALL: [DesktopChoice; 22] = {
        use DesktopChoice::*;
        [
            Plasma,
            Gnome,
            Xfce,
            Cinnamon,
            Mate,
            Cosmic,
            Wayfire,
            Openbox,
            Budgie,
            Lxqt,
            Fluxbox,
            Enlightenment,
            Labwc,
            Pantheon,
            Niri,
            Mango,
            Hyprland,
            I3,
            Sway,
            Bspwm,
            Xmonad,
            Dwm,
        ]
    };

    pub fn is_tiling(self) -> bool {
        use DesktopChoice::*;
        matches!(
            self,
            Niri | Mango | Hyprland | I3 | Sway | Bspwm | Xmonad | Dwm
        )
    }

    /// True when the pick runs on X11 and needs services.xserver.enable.
    pub fn needs_xserver(self) -> bool {
        use DesktopChoice::*;
        matches!(
            self,
            Xfce | Cinnamon
                | Mate
                | Openbox
                | Fluxbox
                | Enlightenment
                | Pantheon
                | I3
                | Bspwm
                | Xmonad
                | Dwm
                | Budgie
                | Lxqt
        )
    }

    pub fn default_login_manager(self) -> LoginManager {
        use DesktopChoice::*;
        match self {
            Plasma => LoginManager::Sddm,
            Gnome | Pantheon | Budgie => LoginManager::Gdm,
            Cosmic => LoginManager::CosmicGreeter,
            other if other.is_tiling() => LoginManager::Regreet,
            _ => LoginManager::Sddm,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoginManager {
    Sddm,
    Gdm,
    PlasmaLoginManager,
    Greetd,
    Ly,
    Regreet,
    Lemurs,
    CosmicGreeter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountsConfig {
    pub primary: UserAccount,
    #[serde(default)]
    pub root_enabled: bool,
    /// Required when root enabled.
    #[serde(default)]
    pub root_hashed_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserAccount {
    pub username: String,
    pub full_name: String,
    /// Output of mkpasswd -m yescrypt, written to hashedPassword.
    pub hashed_password: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TuningProfile {
    Standard,
    Laptop,
    Gaming,
    Development,
    Creative,
    Lightweight,
}

impl BalsaInstallPlan {
    pub fn login_manager(&self) -> LoginManager {
        self.login_manager
            .unwrap_or_else(|| self.desktop.default_login_manager())
    }

    /// Catches Nix that would evaluate but be wrong, plus what the scaffold forbids
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();

        if self.kernel.is_x64_only() && !self.hardware.arch.is_x64() {
            errs.push(format!(
                "kernel {:?} is x86_64-only but arch is {:?}",
                self.kernel, self.hardware.arch
            ));
        }
        if self.network.hostname.is_empty()
            || !self
                .network
                .hostname
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            errs.push(format!("invalid hostname {:?}", self.network.hostname));
        }
        if self.accounts.primary.username.is_empty()
            || !self
                .accounts
                .primary
                .username
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        {
            errs.push(format!(
                "invalid username {:?}",
                self.accounts.primary.username
            ));
        }
        if self.accounts.primary.hashed_password.is_empty() {
            errs.push("primary account has an empty hashed_password".to_string());
        }
        if self.accounts.root_enabled && self.accounts.root_hashed_password.is_none() {
            errs.push("root_enabled is set but root_hashed_password is missing".to_string());
        }
        if self.boot.effective_loader() == BootLoader::Limine
            && self.disk.filesystem != Filesystem::Btrfs
        {
            errs.push(format!(
                "Limine is only offered for btrfs installs, not {:?}",
                self.disk.filesystem
            ));
        }
        match &self.network.host_id {
            Some(id) if id.len() != 8 || !id.chars().all(|c| c.is_ascii_hexdigit()) => {
                errs.push(format!("host_id {id:?} must be exactly 8 hex digits"));
            }
            None if self.disk.filesystem == Filesystem::Zfs => {
                errs.push("ZFS needs network.host_id (networking.hostId)".to_string());
            }
            _ => {}
        }
        if let Err(mut disk_errs) = self.disk.validate() {
            errs.append(&mut disk_errs);
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

impl DiskoConfig {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();

        if let PartitionScheme::Manual { devices } = &self.scheme
            && devices.trim().is_empty()
        {
            errs.push("manual partitioning selected with an empty devices block".to_string());
        }
        if let Some(enc) = &self.encryption
            && (enc.luks_name.is_empty()
                || !enc
                    .luks_name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        {
            errs.push(format!("invalid luks_name {:?}", enc.luks_name));
        }
        if self.filesystem == Filesystem::Zfs && matches!(self.swap, SwapMode::File { .. }) {
            errs.push("ZFS cannot host a swap file; use partition or zram swap".to_string());
        }
        if let SwapMode::Partition { size_gib: 0 } | SwapMode::File { size_gib: 0 } = self.swap {
            errs.push("swap size_gib must be at least 1".to_string());
        }
        if !self.disk.starts_with('/') {
            errs.push(format!("disk {:?} is not an absolute path", self.disk));
        }

        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}
