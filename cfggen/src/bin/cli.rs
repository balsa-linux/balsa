use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use balsa_flakegen::disko::generate_disko_config;
use balsa_flakegen::plan::{DiskoConfig, EncryptionConfig, Filesystem, PartitionScheme, SwapMode};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "balsa-configgen",
    about = "Generate a NixOS config from a Balsa install plan",
    // Keeps `balsa-configgen --plan … --out …` working alongside subcommands.
    subcommand_negates_reqs = true,
    args_conflicts_with_subcommands = true
)]
struct Args {
    /// Install plan in TOML form.
    #[arg(long, required = true)]
    plan: Option<PathBuf>,
    /// Directory to write the generated config into.
    #[arg(long, required = true)]
    out: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Write only disko-config.nix, without a full install plan.
    Disko(DiskoArgs),
}

#[derive(clap::Args)]
struct DiskoArgs {
    /// Target disk, e.g. /dev/sda or /dev/nvme0n1.
    #[arg(long)]
    disk: String,
    #[arg(long = "fs", value_enum)]
    filesystem: FsArg,
    /// Enables LUKS full-disk encryption under /dev/mapper/<NAME>.
    #[arg(long, value_name = "NAME")]
    luks_name: Option<String>,
    #[arg(long, value_enum)]
    swap: SwapArg,
    /// Required for partition and file swap.
    #[arg(long)]
    swap_size_gib: Option<u32>,
    /// Adds the BIOS boot partition GRUB needs on legacy firmware.
    #[arg(long)]
    legacy_bios: bool,
    #[arg(long, default_value = "/boot")]
    esp_mount: String,
    /// Directory to write disko-config.nix into.
    #[arg(long)]
    out: PathBuf,
}

#[derive(Clone, ValueEnum)]
enum FsArg {
    Btrfs,
    Ext4,
}

#[derive(Clone, ValueEnum)]
enum SwapArg {
    Partition,
    File,
    Zram,
    None,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let result = match args.command {
        Some(Command::Disko(disko)) => run_disko(disko),
        None => run_plan(
            &args
                .plan
                .expect("clap requires --plan without a subcommand"),
            &args.out.expect("clap requires --out without a subcommand"),
        ),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run_plan(plan_path: &Path, out: &Path) -> Result<(), String> {
    let plan = balsa_flakegen::load_plan(plan_path).map_err(|e| e.to_string())?;
    let result = balsa_flakegen::generate(&plan).map_err(|e| e.to_string())?;
    result
        .config
        .write_to(out)
        .map_err(|e| format!("could not write to {}: {e}", out.display()))?;

    for w in &result.warnings {
        eprintln!("warning: {w}");
    }
    for name in result.config.files.keys() {
        println!("{}", out.join(name).display());
    }
    Ok(())
}

fn run_disko(args: DiskoArgs) -> Result<(), String> {
    let swap = match (args.swap, args.swap_size_gib) {
        (SwapArg::Partition, Some(size_gib)) => SwapMode::Partition { size_gib },
        (SwapArg::File, Some(size_gib)) => SwapMode::File { size_gib },
        (SwapArg::Zram, None) => SwapMode::Zram,
        (SwapArg::None, None) => SwapMode::None,
        (SwapArg::Partition | SwapArg::File, None) => {
            return Err("--swap-size-gib is required for partition and file swap".to_string());
        }
        (SwapArg::Zram | SwapArg::None, Some(_)) => {
            return Err("--swap-size-gib only applies to partition and file swap".to_string());
        }
    };
    let cfg = DiskoConfig {
        disk: args.disk,
        scheme: PartitionScheme::Guided,
        filesystem: match args.filesystem {
            FsArg::Btrfs => Filesystem::Btrfs,
            FsArg::Ext4 => Filesystem::Ext4,
        },
        swap,
        encryption: args
            .luks_name
            .map(|luks_name| EncryptionConfig { luks_name }),
    };

    let body = generate_disko_config(&cfg, args.legacy_bios, &args.esp_mount)
        .map_err(|e| e.to_string())?;
    fs::create_dir_all(&args.out)
        .map_err(|e| format!("could not create {}: {e}", args.out.display()))?;
    let path = args.out.join("disko-config.nix");
    fs::write(&path, body).map_err(|e| format!("could not write {}: {e}", path.display()))?;
    println!("{}", path.display());
    Ok(())
}
