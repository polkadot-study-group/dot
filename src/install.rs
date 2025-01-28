use std::process::Command;
use std::fs;
use std::process::Stdio;
use std::path::Path;
use std::error::Error;
use crate::os_check;
use crate::chain_specs;

// MOCK COMMAND RUNNER  
// Define a trait for running commands
pub trait CommandRunner {
    fn run(&self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>>;
    fn run_with_input(&self, command: &str, args: &[&str], input: Stdio) -> Result<(), Box<dyn Error>>;
}

// Real command runner that executes system commands
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
        let status = Command::new(command).args(args).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Command {} failed with status {}", command, status).into())
        }
    }

    fn run_with_input(&self, command: &str, args: &[&str], input: Stdio) -> Result<(), Box<dyn Error>> {
        let status = Command::new(command).args(args).stdin(input).status()?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("Command {} failed with status {}", command, status).into())
        }
    }
}

pub fn install_polkadot<C: CommandRunner>(_runner: &C) -> Result<(), Box<dyn Error>>{
    println!("Installing Polkadot via curl");

    let url = "https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/scripts/getting-started.sh"; 
    
    // Run the curl command and pipe its output to bash
    let curl = Command::new("curl")
        .arg("--proto")
        .arg("=https")
        .arg("--tlsv1.2")
        .arg("-sSf")
        .arg(url)
        .stdout(Stdio::piped())
        .spawn()
    .expect("Failed to start curl");
    
    let status = Command::new("bash")
        .stdin(curl.stdout.unwrap())
        .status()
    .expect("Failed to run bash");

    if !status.success() {
        return Err(format!("Failed to run Polkadot-sdk").into());
    }

    println!("Polkadot-sdk is now installed.");
    Ok(()) 
}

pub fn create_binaries_dir() -> Result<(), Box<dyn Error>> {
    
    // Check if the 'binaries' directory exists, if not, create it
    let binaries_dir = Path::new("./binaries");
    if !binaries_dir.exists() {
        println!("'binaries' directory does not exist. Creating it...");
        if let Err(e) = fs::create_dir(binaries_dir) {
            return Err(format!("Failed to create 'binaries' directory: {}", e).into());
        }
    }
    Ok(())
}

pub fn check_binary( destination: &Path )  -> Result<(), Box<dyn Error>> {
    // Destination file path
    if destination.exists() {
        println!("Chain-spec-builder binary is available");
        return Ok(());
    }else {
        return Err(format!("Chain-spec-builder binary is not available").into());
    }
}

pub fn install_omni_node() -> Result<(), Box<dyn Error>> {
    println!("Installing polkadot-omni-node");

    // Determine the operating system and set the appropriate URL
    let os_info = os_check::get_os_info();
    let url ;
    if os_info.as_str() == "macos" {
        url = "https://drive.google.com/uc?export=download&id=1KjarMs-UOvdQgC4r6w5H0EGWeRGOwauo";
    } else {
        url = "https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2412/polkadot-omni-node";
    }

    // Destination file path
    let destination = Path::new("./binaries/polkadot-omni-node");
    if destination.exists() {
        println!("Omni-node binary is available");
        // return Ok(());
    }

    // Check if the 'binaries' directory exists, if not, create it
    let _ = create_binaries_dir();

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

pub fn run_download_script<C: CommandRunner>(runner: &C, destination: &Path) -> Result<(), Box<dyn Error>>{
    let url = "https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2412/asset_hub_westend_runtime.compact.compressed.wasm";

    if file_exists(destination) {
        println!("Wasm file is available");
        return Ok(());
    }

    let nodes_dir = Path::new("./nodes");
    ensure_directory_exists(nodes_dir)?;

    download_file(runner, url, destination)?;

    println!("Download successful: {:?}", destination);
    Ok(())
}


pub fn ensure_directory_exists(dir: &Path) -> Result<(), Box<dyn Error>> {
    if !dir.exists() {
        println!("Directory {:?} does not exist. Creating it...", dir);
        // fs::create_dir_all(dir).map_err(|e| format!("Failed to create directory {:?}: {}", dir, e).into())
        fs::create_dir_all(dir)?;
        Ok(())
    } else {
        Ok(())
    }
}

pub fn file_exists(file_path: &Path) -> bool {
    file_path.exists()
}

pub fn download_file<C: CommandRunner>(runner: &C, url: &str, destination: &Path) -> Result<(), Box<dyn Error>> {
    let args = &["-O", destination.to_str().unwrap(), url];
    runner.run("wget", args).map_err(|e| -> Box<dyn Error> {
        format!(
            "Failed to download file from URL {} to destination {:?}: {}",
            url, destination, e
        )
        .into()
    })?;
    Ok(())
}


/// =================================================================================================
/// Test Module
/// =================================================================================================
#[cfg(test)]
mod tests {
    use tempfile::{tempdir, TempDir};
    use super::*;
    use std::io;
    use std::env;
    use std::{fs, path::Path};
    use std::process::{Command, Output};
    use mockito::mock;
    use std::fs::{File, create_dir_all};
    use crate::install::{install_polkadot,  install_omni_node, run_download_script, create_binaries_dir, 
                        ensure_directory_exists, download_file, check_binary, CommandRunner};
    use crate::process::Stdio;
    use crate::os_check::{check_operating_system};
    use std::path::PathBuf;
    use std::error::Error;

    struct MockRunner {
        pub expected_calls: Vec<(String, Vec<String>)>,
        pub should_fail: bool,
    }

    impl MockRunner {
        pub fn new() -> Self {
            Self {
                expected_calls: vec![],
                should_fail: false,
            }
        }

        pub fn expect_call(&mut self, command: &str, args: &[&str]) {
            self.expected_calls.push((
                command.to_string(),
                args.iter().map(|&s| s.to_string()).collect(),
            ));
        }
    }

    impl CommandRunner for MockRunner {
        fn run(&self, command: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
            if self.should_fail {
                return Err(format!("Mocked failure for command: {}", command).into());
            }
            if !self.expected_calls.iter().any(|(cmd, exp_args)| {
                cmd == command && exp_args == &args.iter().map(|&s| s.to_string()).collect::<Vec<_>>()
            }) {
                return Err(format!("Unexpected command: {} {:?}", command, args).into());
            }
            if command == "wget" {
                let destination = args[1];
                fs::write(destination, "mock wasm content").expect("Failed to write mock WASM file");
            }
            Ok(())
        }
        
        fn run_with_input(&self, command: &str, args: &[&str], _input: Stdio) -> Result<(), Box<dyn Error>> {
            Ok(())
        }
    }


    #[test]
    fn test_ensure_directory_exists_creates_dir() {
        let temp_dir = tempdir().unwrap();
        let new_dir = temp_dir.path().join("new_dir");

        assert!(!new_dir.exists());
        ensure_directory_exists(&new_dir).unwrap();
        assert!(new_dir.exists());
    }

    #[test]
    fn test_download_file_success() {
        let temp_dir = tempdir().unwrap(); // Create a unique temporary directory for the test
        let nodes_dir = temp_dir.path().join("nodes");
        let destination = nodes_dir.join("asset_hub_westend_runtime.compact.compressed.wasm");

        // Ensure the nodes directory exists
        fs::create_dir_all(&nodes_dir).expect("Failed to create nodes directory");

        let url = "https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2412/asset_hub_westend_runtime.compact.compressed.wasm";

        // Set up the mock runner
        let mut mock_runner = MockRunner::new();
        mock_runner.expect_call(
            "wget",
            &[
                "-O",
                destination.to_str().unwrap(),
                url,
            ],
        );

        // Run the download function
        let result = download_file(&mock_runner, url, &destination);

        // Assertions
        assert!(result.is_ok(), "download_file failed: {:?}", result);
        assert!(
            destination.exists(),
            "Expected file at destination {:?} does not exist",
            destination
        );
    }

    #[test]
    fn test_run_download_script_skips_download_when_file_exists() {
        let temp_dir = tempdir().unwrap();
        let nodes_dir = temp_dir.path().join("nodes");
        let wasm_path = temp_dir.path().join("asset_hub_westend_runtime.compact.compressed.wasm");
        fs::create_dir_all(temp_dir.path().join("nodes")).unwrap();
        File::create(&wasm_path).unwrap();
        let wasm_path = nodes_dir.join("asset_hub_westend_runtime.compact.compressed.wasm");
        fs::write(&wasm_path, "mock wasm content").expect("Failed to write mock WASM file");
    

        let runner = MockRunner::new();
        assert!(run_download_script(&runner, &wasm_path).is_ok());
    }


    #[test]
    fn test_run_ensure_directory_exists() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let nodes_dir = temp_dir.path().join("nodes");
        fs::create_dir_all(&nodes_dir).expect("Failed to create nodes directory");
        let wasm_path = nodes_dir.join("asset_hub_westend_runtime.compact.compressed.wasm");
        fs::write(&wasm_path, "mock wasm content").expect("Failed to write mock WASM file");
    
        let runner = MockRunner::new();
        let result = run_download_script(&runner, &wasm_path);
    
        assert!(result.is_ok(), "run_download_script failed with {:?}", result.unwrap_err());
    }

    #[test]
    fn test_check_binary_exists() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let binary_path = temp_dir.path().join("chain-spec-builder");

        // Create a mock binary file
        if binary_path.exists() {
            fs::remove_file(&binary_path).expect("Failed to remove mock binary file");
            let result = check_binary(&binary_path);
            assert!(result.is_err(), "check_binary should have failed but didn't");
            assert_eq!(result.unwrap_err().to_string(), "Chain-spec-builder binary is not available");
        }else {
            fs::write(&binary_path, "mock binary content").expect("Failed to write mock binary file");
            let result = check_binary(&binary_path);
            assert!(result.is_ok(), "check_binary failed: {:?}", result.unwrap_err());
        }
    }

 

    // Test if 'binaries' directory is created when it doesn't exist
    #[test]
    fn test_create_binaries_directory() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let binaries_dir = temp_dir.path().join("binaries");
 
        // Ensure the directory doesn't exist at the start of the test
        assert!(!binaries_dir.exists());
 
        // Change the working directory for the test to the temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();
 
        // Run the function to check if it creates the 'binaries' directory
        create_binaries_dir().unwrap();
 
        // Check that the 'binaries' directory was created
        assert!(binaries_dir.exists());
    }
 
    #[test]
    fn test_install_omni_node() {
        install_omni_node();
        // Create a temporary directory for mock binaries
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let mock_dir = temp_dir.path();
    
        // Create mock `wget` binary
        let wget_path = mock_dir.join("wget");
        fs::write(&wget_path, "#!/bin/sh\necho Mock wget executed\nexit 0")
            .expect("Failed to write mock wget");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&wget_path, fs::Permissions::from_mode(0o755))
                .expect("Failed to make mock wget executable");
        }
    
        // Override PATH to use mock binaries
        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", mock_dir.to_str().unwrap(), original_path);
        env::set_var("PATH", new_path);
    
        let url = "https://github.com/paritytech/polkadot-sdk/releases/download/polkadot-stable2412/polkadot-omni-node";
        let destination = Path::new("./binaries/polkadot-omni-node");
    
        if destination.exists() {
            fs::remove_file(destination).expect("Failed to remove existing file");
        }
    
        // Simulate the successful download with mock `wget`
        let output = Command::new("wget")
            .arg("-O")
            .arg(destination)
            .arg(url)
            .output()
            .expect("Failed to execute wget");
    
        assert!(output.status.success(), "Omni-node installation failed");
    
        // Restore the original PATH
        env::set_var("PATH", original_path);
    }
    
    #[test]
    fn test_gen_chain_spec_failure_chmod_wasm() {
        // Simulate the failure of chmod when the file doesn't exist.
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mock_dir = temp_dir.path();
        let wasm_path = mock_dir.join("non_existent_wasm_file.wasm");
    
        // This will fail because the file doesn't exist
        let result = Command::new("chmod")
            .arg("+r")
            .arg(wasm_path.to_str().unwrap())
            .status()
            .expect("Failed to run chmod");
    
        assert!(!result.success(), "Expected chmod to fail because the file does not exist");
    }
 
    #[test]
    fn test_gen_chain_spec_failure_chmod_chain_spec_builder() {
        // Create a temporary directory for mock binaries
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let mock_dir = temp_dir.path();
    
        // Define the path to the non-existent chain-spec-builder binary
        let chain_spec_builder_path = mock_dir.join("non_existent_chain_spec_builder");
    
        // Attempt to set execute permissions on the non-existent file
        let result = Command::new("chmod")
            .arg("+x")
            .arg(chain_spec_builder_path.to_str().unwrap())
            .status()
            .expect("Failed to run chmod");
    
        // Assert that chmod fails (i.e., returns a non-zero exit status)
        assert!(!result.success(), "Expected chmod to fail due to non-existent file.");
    }

    
    /// ====================
    /// OS CHECK TESTS
    /// ====================
    #[cfg(target_os = "linux")]
    #[test]
    fn test_check_operating_system_linux() {
        // env::set_var("CARGO_CFG_TARGET_OS", "linux");
 
        let os_info = check_operating_system(env::consts::OS);
        assert_eq!(os_info, "linux");
    }
 
    #[cfg(target_os = "windows")]
    #[test]
    fn test_check_operating_system_windows_wsl() {
        env::set_var("CARGO_CFG_TARGET_OS", "windows");
        
        // Mock WSL behavior (we simulate WSL here)
        if mock_is_wsl() {
            let os_info = check_operating_system("windows");
            assert_eq!(os_info, "windows-wsl2");
        } else {
            let os_info = check_operating_system("windows");
            assert_eq!(os_info, "windows");
        }
    }
 
    #[cfg(target_os = "macos")]
    #[test]
    fn test_check_operating_system_macos() {
        env::set_var("CARGO_CFG_TARGET_OS", "macos");
        
        // Check if the OS info returns the correct result
        let os_info = check_operating_system("macos");
        assert_eq!(os_info, "macos");
    }

}
 
 
 