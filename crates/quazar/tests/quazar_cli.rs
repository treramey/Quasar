use std::process::Command;

#[test]
fn command_names_share_version_protocol_and_settings_paths() {
    let tmp = tempfile::tempdir().unwrap();
    for args in [
        vec!["--version"],
        vec!["_protocol-version"],
        vec!["config", "path"],
    ] {
        let outputs: Vec<_> = [
            env!("CARGO_BIN_EXE_quazar"),
            env!("CARGO_BIN_EXE_quasar"),
            env!("CARGO_BIN_EXE_nebula"),
        ]
        .iter()
        .map(|exe| {
            let out = Command::new(exe)
                .args(&args)
                .env("QUAZAR_DATA_DIR", tmp.path())
                .env_remove("QUAZAR_IMPORT_BUNDLE")
                .output()
                .unwrap();
            assert!(out.status.success(), "{:?}", out);
            out.stdout
        })
        .collect();
        assert!(outputs.iter().all(|out| *out == outputs[0]));
        if args == ["--version"] {
            assert!(String::from_utf8_lossy(&outputs[0]).starts_with("quazar "));
        }
    }
}

#[test]
fn upgrade_does_not_replace_fork_with_upstream() {
    let out = Command::new(env!("CARGO_BIN_EXE_quazar"))
        .arg("upgrade")
        .env_remove("QUAZAR_INSTALL_URL")
        .env_remove("NEBULA_INSTALL_URL")
        .env_remove("QUAZAR_IMPORT_BUNDLE")
        .env_remove("NEBULA_IMPORT_BUNDLE")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("make install"));
}

#[test]
fn existing_settings_and_old_environment_overrides_remain_reachable() {
    let home = tempfile::tempdir().unwrap();
    let data = home.path().join("data");
    let legacy = data.join("nebula");
    let canonical = data.join("quazar");
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("config.json"), r#"{"theme":"ocean"}"#).unwrap();
    let invoke = |overrides: &[(&str, &std::path::Path)]| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_quazar"));
        cmd.env_clear()
            .env("HOME", home.path())
            .env("XDG_DATA_HOME", &data)
            .args(["config", "export"]);
        for (key, value) in overrides {
            cmd.env(key, value);
        }
        let out = cmd.output().unwrap();
        assert!(out.status.success(), "{out:?}");
        serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap()
    };
    assert_eq!(invoke(&[])["config"]["theme"], "ocean");
    std::fs::create_dir(&canonical).unwrap();
    std::fs::write(canonical.join("config.json"), r#"{"theme":"rose"}"#).unwrap();
    assert_eq!(invoke(&[])["config"]["theme"], "rose");
    assert_eq!(
        invoke(&[("NEBULA_DATA_DIR", &legacy)])["config"]["theme"],
        "ocean"
    );
    assert_eq!(
        invoke(&[
            ("NEBULA_DATA_DIR", &legacy),
            ("QUAZAR_DATA_DIR", &canonical)
        ])["config"]["theme"],
        "rose"
    );
    assert_eq!(
        std::fs::read_to_string(legacy.join("config.json")).unwrap(),
        r#"{"theme":"ocean"}"#
    );
}
