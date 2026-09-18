use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_balsa-configgen"))
        .args(args)
        .output()
        .unwrap()
}

fn out_dir(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("cli")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

#[test]
fn disko_subcommand_writes_the_requested_layout() {
    let out = out_dir("disko");
    let o = cli(&[
        "disko",
        "--disk",
        "/dev/nvme0n1",
        "--fs",
        "btrfs",
        "--luks-name",
        "balsa_root",
        "--swap",
        "partition",
        "--swap-size-gib",
        "6",
        "--legacy-bios",
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", stderr(&o));

    let nix = fs::read_to_string(out.join("disko-config.nix")).unwrap();
    for want in [
        r#"device = "/dev/nvme0n1";"#,
        r#"name = "balsa_root";"#,
        r#"size = "6G";"#,
        r#"type = "EF02";"#,
        r#"type = "btrfs";"#,
    ] {
        assert!(nix.contains(want), "missing {want} in:\n{nix}");
    }
}

#[test]
fn full_plan_invocation_still_works() {
    let out = out_dir("plan");
    let plan = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-plans/xfce-zen-ext4-noswap.toml"
    );
    let o = cli(&["--plan", plan, "--out", out.to_str().unwrap()]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(out.join("flake.nix").exists());
    assert!(out.join("disko-config.nix").exists());
}

#[test]
fn bad_disko_input_is_rejected() {
    let out = out_dir("rejected");
    let out = out.to_str().unwrap();
    let base = ["disko", "--disk", "/dev/sda", "--fs", "ext4", "--out", out];
    let cases: &[(&[&str], &str)] = &[
        (&["--swap", "partition"], "--swap-size-gib is required"),
        (
            &["--swap", "zram", "--swap-size-gib", "4"],
            "only applies to partition",
        ),
        (
            &["--swap", "none", "--luks-name", "has space"],
            "invalid luks_name",
        ),
        (&["--swap", "file", "--swap-size-gib", "0"], "at least 1"),
    ];
    for (extra, want) in cases {
        let args: Vec<&str> = base.iter().chain(extra.iter()).copied().collect();
        let o = cli(&args);
        assert!(!o.status.success(), "{extra:?} should fail");
        assert!(
            stderr(&o).contains(want),
            "{extra:?}: expected {want:?}, got:\n{}",
            stderr(&o)
        );
    }

    let xfs = cli(&[
        "disko", "--disk", "/dev/sda", "--fs", "xfs", "--swap", "none", "--out", out,
    ]);
    assert!(
        stderr(&xfs).contains("invalid value 'xfs'"),
        "{}",
        stderr(&xfs)
    );

    let relative = cli(&[
        "disko", "--disk", "sda", "--fs", "ext4", "--swap", "none", "--out", out,
    ]);
    assert!(stderr(&relative).contains("not an absolute path"));

    let mixed = cli(&["--plan", "x.toml", "disko", "--disk", "/dev/sda"]);
    assert!(
        !mixed.status.success(),
        "--plan with a subcommand should fail"
    );

    let bare = cli(&[]);
    assert!(!bare.status.success(), "no arguments should fail");
    assert!(stderr(&bare).contains("--plan"));
}
