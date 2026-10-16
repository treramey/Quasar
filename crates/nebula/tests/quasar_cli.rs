use std::process::Command;

#[test]
fn command_names_share_version_protocol_and_settings_paths() {
    let tmp = tempfile::tempdir().unwrap();
    for args in [
        vec!["--version"],
        vec!["_protocol-version"],
        vec!["config", "path"],
    ] {
        let outputs: Vec<_> = [env!("CARGO_BIN_EXE_quasar"), env!("CARGO_BIN_EXE_nebula")]
            .iter()
            .map(|exe| {
                let out = Command::new(exe)
                    .args(&args)
                    .env("NEBULA_DATA_DIR", tmp.path())
                    .env_remove("NEBULA_IMPORT_BUNDLE")
                    .output()
                    .unwrap();
                assert!(out.status.success(), "{:?}", out);
                out.stdout
            })
            .collect();
        assert_eq!(outputs[0], outputs[1]);
        if args == ["--version"] {
            assert!(String::from_utf8_lossy(&outputs[0]).starts_with("quasar "));
        }
    }
}

#[test]
fn upgrade_does_not_replace_fork_with_upstream() {
    let out = Command::new(env!("CARGO_BIN_EXE_quasar"))
        .arg("upgrade")
        .env_remove("NEBULA_INSTALL_URL")
        .env_remove("NEBULA_IMPORT_BUNDLE")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("make install"));
}
