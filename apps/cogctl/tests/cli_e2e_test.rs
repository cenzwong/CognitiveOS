use std::process::Command;

fn run_cog(args: &[&str], db_path: &str) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cog"))
        .arg("--db")
        .arg(db_path)
        .args(args)
        .output()
        .expect("Failed to execute cog binary");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

#[test]
fn test_cli_e2e_workflow() {
    let temp_dir = std::env::temp_dir().join(format!("cog_cli_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let db_path = temp_dir.join("cli.db");
    let db_str = db_path.to_str().unwrap();

    let (success, stdout, _) = run_cog(&["ps"], db_str);
    assert!(success);
    assert!(stdout.contains("No processes found."));

    let (success, stdout, _) = run_cog(
        &[
            "run",
            "Refactor caching layer",
            "--workspace",
            "/workspace/demo",
        ],
        db_str,
    );
    assert!(success);
    assert!(stdout.contains("Process created and admitted to READY pool."));
    assert!(stdout.contains("State:           ready"));

    let (success, stdout, _) = run_cog(&["--json", "ps"], db_str);
    assert!(success);
    let processes: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let proc_array = processes.as_array().unwrap();
    assert_eq!(proc_array.len(), 1);
    let proc_id = proc_array[0]["process_id"].as_str().unwrap();

    let (success, stdout, _) = run_cog(&["inspect", proc_id], db_str);
    assert!(success);
    assert!(stdout.contains("Refactor caching layer"));
    assert!(stdout.contains("PROCESS_CREATED"));
    assert!(stdout.contains("STATE_TRANSITION"));

    let (success, stdout, _) = run_cog(&["pause", proc_id], db_str);
    assert!(success);
    assert!(stdout.contains("paused"));

    let (success, stdout, _) = run_cog(&["--json", "inspect", proc_id], db_str);
    assert!(success);
    let inspect_json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(inspect_json["process"]["state"], "paused");
    assert_eq!(inspect_json["events"].as_array().unwrap().len(), 3);

    let (success, stdout, _) = run_cog(&["resume", proc_id], db_str);
    assert!(success);
    assert!(stdout.contains("resumed to READY"));

    let (success, stdout, _) = run_cog(&["cancel", proc_id], db_str);
    assert!(success);
    assert!(stdout.contains("cancelled"));

    let (success, _, stderr) = run_cog(&["resume", proc_id], db_str);
    assert!(!success);
    assert!(stderr.contains("cannot transition"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
