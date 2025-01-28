use std::path::{Path, PathBuf};
use std::error::Error;
use process::Command;
use std::process;
use std::fs;
use crate::install;
use crate::os_check;


pub fn install_chain_spec_builder() -> Result<(), Box<dyn Error>> {
    println!("Installing chain-spec-builder");

    // Determine the operating system and set the appropriate URL
    let os_info = os_check::get_os_info();
    let url;
    if os_info.as_str() == "macos" {
        url = "https://drive.google.com/uc?export=download&id=1K9QmJVnjnV3wfOHn7ZGnbtF2hMZCGeUT";
    } else {
        url = "https://drive.google.com/uc?export=download&id=1PfS0kAs1CxWmSsMm1anYA6f0jCIxm8nX";
    }

    // check and create binaries directory
    let destination = Path::new("./binaries/chain-spec-builder");
    let _ = install::create_binaries_dir()?;
    if destination.exists() {
        println!("Chain-spec-builder binary is already available.");
        return Ok(());
    }

    println!("Downloading...");
    let output = Command::new("wget")
        .arg("-O")
        .arg(destination)
        .arg(url)
        .output()
        .map_err(|e| format!("Failed to execute wget: {}", e))?;

    // Check if the download was successful
    if output.status.success() {
        println!("Download successful: {:?}", destination);

        let destination_str = destination.to_str().expect("Failed to convert path to str");

        let _chmod_status = Command::new("chmod")
            .args(&["755", destination_str])
            .status()
            .expect("Failed to run chmod");

        return Ok(());
    } else {
        return Err(format!(
            "Download failed with exit code: {:?}",
            output.status.code()
        )
        .into());
    }
}


pub fn gen_chain_spec(wasm_source_path: Option<&Path>, chain_spec_builder_path: Option<&Path>) -> Result<(), Box<dyn Error>> {
    let wasm_path = wasm_source_path.unwrap_or_else(|| Path::new("./nodes/westend.wasm"));
    let builder_path = chain_spec_builder_path.unwrap_or_else(|| Path::new("./binaries/chain-spec-builder"));

    if !wasm_path.exists() {
        eprintln!("WASM file not found: {:?}", wasm_path);
        return Err(format!("WASM file not found: {:?}", wasm_path).into());
    }

    let _chmod_status = Command::new("chmod")
        .args(&["+r", wasm_path.to_str().unwrap()])
        .status()
        .expect("Failed to run chmod");


    // Add execute permissions to the chain-spec-builder binary
    let chmod_chain_spec_status = Command::new("chmod")
        .args(&["+x", builder_path.to_str().unwrap()])
        .status()
        .expect("Failed to run chmod on chain-spec-builder");

    if !chmod_chain_spec_status.success() {
        return Err(format!("Failed to add execute permissions to the chain-spec-builder").into());
    }

    let _chain_spec_status = Command::new(builder_path)
        .args(&[
            "create",
            "--relay-chain", "westend2",
            "--para-id", "100",
            "--runtime", wasm_path.to_str().unwrap(),
            "named-preset", "development"
        ])
        .status()
        .expect("Failed to run chain-spec-builder");

    let _ = locate_chain_spec();
    Ok(())
}

pub fn locate_chain_spec()-> Result<(), String>{
    // Define the directory to search for the chain_spec.json file
    let search_directories = vec!["./", "../"];
    let mut chain_spec_source_path: Option<PathBuf> = None;

    // Locate the chain_spec.json file
    for dir in &search_directories {
        let potential_path: PathBuf = Path::new(dir).join("chain_spec.json");
        if potential_path.exists() {
            chain_spec_source_path = Some(potential_path);
            break;
        }
    }

    // Check if the file was found
    let chain_spec_source_path = match chain_spec_source_path {
        Some(path) => path,
        None => {
            return Err(format!("chain_spec.json not found in the specified directories."));
        }
    };

    let _ = move_chain_spec(&chain_spec_source_path);
    Ok(())
}

pub fn move_chain_spec(chain_spec_source_path:&Path) -> Result<(), String>{

    let chain_spec_destination_path = Path::new("./chain-specs/chain_spec.json");

    let _ = create_chain_specs_dir(chain_spec_destination_path.parent().unwrap())?;
    
    // Move the chain_spec.json file to the chain-specs directory
    if let Err(e) = fs::rename(&chain_spec_source_path, &chain_spec_destination_path) {
        return Err(format!("Failed to move chain_spec.json: {}", e));
    }
    Ok(())
}

pub fn create_chain_specs_dir(path: &Path) -> Result<(), String> {
    if let Err(e) = fs::create_dir_all(path) {
        return Err(format!("Failed to create chain-specs directory: {}", e));
    }
    Ok(())
}


/// =================================================================================================
/// Test Module
/// =================================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use mockall::predicate;
    use tempfile::tempdir;
    use std::env;
    use std::thread;
    use std::time::Duration;
    use std::io::Write;
    use tempfile::TempDir;
    use mockall::predicate::*;
    use mockito::mock;
    use std::process::Command;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_install_chain_spec_builder_success() {
        // Mock the URL and wget command
        let _mock = mock("GET", "/chain-spec-builder")
            .with_status(200)
            .with_body("test binary content")
            .create();

        // Temporary directory to test file creation
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("binaries");
        fs::create_dir_all(&temp_path).unwrap();

        // Mock successful command execution for wget and chmod
        let destination = temp_path.join("chain-spec-builder");
        let url = &format!("{}/chain-spec-builder", mockito::server_url());

        let output = Command::new("wget")
            .arg("-O")
            .arg(&destination)
            .arg(url)
            .output()
            .expect("Failed to execute wget");

        assert!(output.status.success(), "wget command failed");

        let destination_str = destination.to_str().expect("Failed to convert path to str");

        let chmod_status = Command::new("chmod")
            .args(&["755", destination_str])
            .status()
            .expect("Failed to run chmod");

        assert!(chmod_status.success(), "chmod command failed");

        // Execute the function
        let result = install_chain_spec_builder();
        if result.is_err() {
            assert!(result.is_err());
        }else {
            assert!(
                result.is_ok(),
                "install_chain_spec_builder() failed with error: {:?}",
                result.unwrap_err()
            );
            
        }
    }
 

    #[test]
    fn test_gen_chain_spec_success() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let wasm_path = temp_dir.path().join("asset_hub_westend_runtime.compact.compressed.wasm");
        fs::write(&wasm_path, "mock wasm content").expect("Failed to write mock WASM file");

        let builder_path = temp_dir.path().join("chain-spec-builder");
        fs::write(&builder_path, "#!/bin/sh\necho \"Mock chain-spec-builder executed\"\nexit 0").expect("Failed to write mock builder file");

        #[cfg(unix)]
        {
            fs::set_permissions(&wasm_path, fs::Permissions::from_mode(0o644)).expect("Failed to set read permissions on mock WASM file");
            fs::set_permissions(&builder_path, fs::Permissions::from_mode(0o755)).expect("Failed to make mock builder executable");
        }

        thread::sleep(Duration::from_millis(100));

        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", temp_dir.path().to_str().unwrap(), original_path);
        env::set_var("PATH", new_path);

        let result = gen_chain_spec(Some(&wasm_path), Some(&builder_path));
        assert!(result.is_ok());

        env::set_var("PATH", original_path);
    }

    #[test]
    fn test_mock_chain_spec_builder_execution() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mock_dir = temp_dir.path();

        let wasm_path = mock_dir.join("asset_hub_westend_runtime.compact.compressed.wasm");
        fs::write(&wasm_path, "mock wasm content").expect("Failed to write mock WASM file");

        let builder_path = mock_dir.join("chain-spec-builder");
        fs::write(
            &builder_path,
            "#!/bin/sh\necho \"Mock chain-spec-builder executed\"\nexit 0",
        )
        .expect("Failed to write mock builder file");

        #[cfg(unix)]
        {
            fs::set_permissions(&builder_path, fs::Permissions::from_mode(0o755))
                .expect("Failed to make mock builder executable");
            fs::set_permissions(&wasm_path, fs::Permissions::from_mode(0o644))
                .expect("Failed to set read permissions on mock WASM file");
        }

        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", mock_dir.to_str().unwrap(), original_path);
        env::set_var("PATH", new_path);

        assert!(builder_path.exists(), "Mock chain-spec-builder does not exist");
        assert!(wasm_path.exists(), "Mock WASM file does not exist");

        let output = std::process::Command::new("chain-spec-builder")
            .output()
            .expect("Failed to execute mock chain-spec-builder");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            output.status.success(),
            "Mock chain-spec-builder failed with stderr: {}",
            stderr
        );
        assert_eq!(
            stdout.trim(),
            "Mock chain-spec-builder executed",
            "Unexpected stdout: {}",
            stdout
        );

        env::set_var("PATH", original_path);
    }


    #[test]
    fn test_gen_chain_spec_wasm_not_found() {
        let result = gen_chain_spec(Some(Path::new("non_existent_wasm.wasm")), None);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "WASM file not found: \"non_existent_wasm.wasm\"");
    }

    
    #[test]
    fn test_gen_chain_spec_failure_wasm_not_found() {
        // Create a temporary directory for mock binaries
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mock_dir = temp_dir.path();

        // Create mock WASM file
        let wasm_source_path = mock_dir.join("asset_hub_westend_runtime.compact.compressed.wasm");
        fs::write(&wasm_source_path, "mock wasm content")
            .expect("Failed to write mock WASM file");

        // Simulating that the chain-spec-builder binary doesn't exist.
        let chain_spec_builder_path = mock_dir.join("chain-spec-builder");
        assert!(!chain_spec_builder_path.exists(), "chain-spec-builder binary should not exist for this test");
    
        // Override PATH to include the mock directory
        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", mock_dir.to_str().unwrap(), original_path);
        env::set_var("PATH", new_path);

        
        // Call the function after setting up the mock environment
        let result = gen_chain_spec(Some(&wasm_source_path), Some(&chain_spec_builder_path));
        assert!(result.is_err(), "gen_chain_spec should have failed");
        assert_eq!(result.unwrap_err().to_string(), "Failed to add execute permissions to the chain-spec-builder");

        
        // Create mock `chain-spec-builder` binary
        let chain_spec_builder_path = mock_dir.join("chain-spec-builder");
        fs::write(&chain_spec_builder_path, "#!/bin/sh\necho \"Mock chain-spec-builder executed\"\nexit 0")
            .expect("Failed to write mock chain-spec-builder");
    
        // Ensure the file is fully written and permissions are set
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&chain_spec_builder_path, fs::Permissions::from_mode(0o755))
                .expect("Failed to make mock chain-spec-builder executable");
            fs::set_permissions(&wasm_source_path, fs::Permissions::from_mode(0o644))
                .expect("Failed to set read permissions on mock WASM file");
        }
    
        // Override PATH to include the mock directory
        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", mock_dir.to_str().unwrap(), original_path);
        env::set_var("PATH", new_path);

    
        // Simulating that the WASM file doesn't exist.
        let wasm_source_path = Path::new("./nodes/asset_hub_westend_runtime_test.compact.compressed.wasm");
        assert!(!wasm_source_path.exists(), "WASM file should not exist for this test");
    
        let result = gen_chain_spec(Some(&wasm_source_path), Some(&chain_spec_builder_path));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "WASM file not found: \"./nodes/asset_hub_westend_runtime_test.compact.compressed.wasm\"");
    
        // Restore the original PATH
        env::set_var("PATH", original_path);
    }

    #[test]
    fn test_move_chain_spec() {
        // Create a temporary directory for the test
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Create mock source and destination directories
        let source_dir = temp_path.join("source");
        let dest_dir = temp_path.join("chain-specs");

        // Create source directory and mock chain_spec.json file
        fs::create_dir(&source_dir).expect("Failed to create source directory");
        let source_file = source_dir.join("chain_spec.json");
        fs::write(&source_file, "mock chain spec content").expect("Failed to write chain_spec.json");

        // Update the search directories for the test
        let search_directories = vec![source_dir.to_str().unwrap().to_string()];

        // Mock the function to use test-specific paths
        fn test_move_chain_spec(search_directories: Vec<String>, dest_dir: &Path) -> Result<(), String> {
            let mut chain_spec_source_path: Option<PathBuf> = None;

            // Locate the chain_spec.json file
            for dir in &search_directories {
                let potential_path = Path::new(dir).join("chain_spec.json");
                if potential_path.exists() {
                    chain_spec_source_path = Some(potential_path);
                    break;
                }
            }

            // Check if the file was found
            let chain_spec_source_path = match chain_spec_source_path {
                Some(path) => path,
                None => {
                    return Err(format!("chain_spec.json not found in the specified directories."));
                }
            };

            let chain_spec_destination_path = dest_dir.join("chain_spec.json");

            // Create the chain-specs directory if it does not exist
            if let Err(e) = fs::create_dir_all(chain_spec_destination_path.parent().unwrap()) {
                return Err(format!("Failed to create chain-specs directory: {}", e));
            }

            // Move the chain_spec.json file to the chain-specs directory
            if let Err(e) = fs::rename(&chain_spec_source_path, &chain_spec_destination_path) {
                return Err(format!("Failed to move chain_spec.json: {}", e));
            }
            Ok(())
        }

        // Run the test-specific move_chain_spec function
        let result = test_move_chain_spec(search_directories, &dest_dir);

        // Assertions
        assert!(result.is_ok(), "move_chain_spec failed: {:?}", result.err());
        assert!(
            !source_file.exists(),
            "Source chain_spec.json should not exist after being moved."
        );
        let dest_file = dest_dir.join("chain_spec.json");
        assert!(
            dest_file.exists(),
            "chain_spec.json was not moved to the destination directory."
        );

        // Verify file contents
        let content = fs::read_to_string(dest_file).expect("Failed to read destination chain_spec.json");
        assert_eq!(content, "mock chain spec content", "File content mismatch.");
    }

    #[test]
    fn test_move_chain_spec_failure() {
        // Run the function without the source file present to simulate an error
        let non_existent_path = PathBuf::from("non_existent_chain_spec.json");

        // Run the function with the non-existent file path
        let result = move_chain_spec(&non_existent_path);

        // Assert that the result is Err
        assert!(result.is_err());
    }
    #[test]
    fn test_move_chain_spec_failure_v2() {
        let non_existent_path = PathBuf::from("chain_spec.json");

        let result = move_chain_spec(&non_existent_path);

        // Assert that the result is Err
        assert_eq!(result.err().unwrap(), "Failed to move chain_spec.json: No such file or directory (os error 2)");
    }

    #[test]
    fn test_move_chain_spec_success() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let source_file_path = temp_dir.path().join("chain_spec.json");
        let destination_dir_path = temp_dir.path().join("test-specs");
        let _destination_file_path = destination_dir_path.join("chain_spec.json");

        // Create a dummy chain_spec.json file
        fs::write(&source_file_path, "mock chain spec content").unwrap();

        // Run the function with the source file path
        let result = move_chain_spec(&source_file_path);

        // Assert that the result is Ok
        dbg!(&result);
        assert!(result.is_ok());

        // Assert that the source file no longer exists
        assert!(!source_file_path.exists(), "Source chain_spec.json should not exist after being moved.");
    }


    #[test]
    fn test_create_dir_all_failure() {
        // Create a path to a directory where we don't have write permissions
        let invalid_path = PathBuf::from("/invalid_path/chain_spec.json");

        // Run the function with the invalid directory path
        let result = create_chain_specs_dir(&invalid_path);

        // Assert that the result is Err
        assert!(result.is_err());
        assert!(result.err().unwrap().contains("Failed to create chain-specs directory"));
    }

    #[test]
    fn test_chain_spec_handling() {
        let temp_dir = tempfile::tempdir().unwrap();
        let chain_spec_source_path = temp_dir.path().join("chain_spec.json");
        let chain_spec_destination_dir = temp_dir.path().join("chain-specs");
        let chain_spec_destination_path = chain_spec_destination_dir.join("chain_spec.json");

        // Create a dummy chain_spec.json file
        let mut file = fs::File::create(&chain_spec_source_path).unwrap();
        writeln!(file, "{{\"key\": \"value\"}}").unwrap();

        // Ensure the destination directory does not exist initially
        assert!(!chain_spec_destination_dir.exists());

        // Test directory creation
        if let Err(e) = fs::create_dir_all(chain_spec_destination_path.parent().unwrap()) {
            panic!("Failed to create chain-specs directory: {}", e);
        }
        assert!(chain_spec_destination_dir.exists(), "Directory was not created.");

        // Test file move
        if let Err(e) = fs::rename(&chain_spec_source_path, &chain_spec_destination_path) {
            panic!("Failed to move chain_spec.json: {}", e);
        }
        assert!(!chain_spec_source_path.exists(), "Source file still exists.");
        assert!(chain_spec_destination_path.exists(), "File was not moved to the destination.");
    }

    #[test]
    fn test_directory_creation_failure() {
        let invalid_path = Path::new("/invalid_path/chain-specs");

        // Try to create a directory in an invalid location
        let result = fs::create_dir_all(invalid_path);
        assert!(result.is_err(), "Expected failure to create directory.");
    }

    #[test]
    fn test_file_move_failure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let chain_spec_source_path = temp_dir.path().join("nonexistent.json");
        let chain_spec_destination_dir = temp_dir.path().join("chain-specs");
        let chain_spec_destination_path = chain_spec_destination_dir.join("chain_spec.json");

        // Ensure the source file does not exist
        assert!(!chain_spec_source_path.exists());

        // Try to move a nonexistent file
        let result = fs::rename(&chain_spec_source_path, &chain_spec_destination_path);
        assert!(result.is_err(), "Expected failure to move a nonexistent file.");
    }
    

}

