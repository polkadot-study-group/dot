use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::os_check;

pub struct EthRpcInstaller {
    url: String,
    binaries_dir: PathBuf,
    binary_name: String,
}

impl EthRpcInstaller {
    pub fn new( binaries_dir: &str, binary_name: &str) -> Self {
        let os_info = os_check::get_os_info();
        let url;
        if os_info.as_str() == "macos"  {
            url = "https://drive.google.com/uc?export=download&id=1JVJbzaLVgpaSl4-DygPncJpUOg_9OPxX";
        }else {
            url = "https://drive.google.com/uc?export=download&id=18mN2cIJfLVzEWiGdzV8E6jjUZP-wt8Hi";
        }
        Self {
            url: url.to_string(),
            binaries_dir: PathBuf::from(binaries_dir),
            binary_name: binary_name.to_string(),
        }
    }

    pub fn install(&self) -> Result<(), Box<dyn Error>> {
        println!("Installing eth-rpc");

        let destination = self.binaries_dir.join(&self.binary_name);
        if destination.exists() {
            println!("Eth-rpc binary is available");
            return Ok(());
        }

        self.create_binaries_dir()?;

        println!("Downloading...");
        self.download_binary(&destination)?;

        self.set_executable_permissions(&destination)?;

        Ok(())
    }

    fn create_binaries_dir(&self) -> Result<(), Box<dyn Error>> {
        if !self.binaries_dir.exists() {
            fs::create_dir_all(&self.binaries_dir)?;
            println!("Created binaries directory: {:?}", self.binaries_dir);
        }
        Ok(())
    }

    fn download_binary(&self, destination: &Path) -> Result<(), Box<dyn Error>> {
        let output = Command::new("wget")
            .arg("-O")
            .arg(destination)
            .arg(&self.url)
            .output()
            .map_err(|e| format!("Failed to execute wget: {}", e))?;

        if output.status.success() {
            println!("Download successful: {:?}", destination);
            Ok(())
        } else {
            Err(format!(
                "Download failed with exit code: {:?}",
                output.status.code()
            )
            .into())
        }
    }

    fn set_executable_permissions(&self, destination: &Path) -> Result<(), Box<dyn Error>> {
        Command::new("chmod")
            .args(&["755", destination.to_str().unwrap()])
            .status()
            .map_err(|e| format!("Failed to set permissions: {}", e))?;

        println!("Set executable permissions for: {:?}", destination);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_install_eth_rpc_success() {
        let test_dir = env::temp_dir().join("test_binaries");
        let binary_name = "eth-rpc-test";

        // Ensure a clean state
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }

        let installer = EthRpcInstaller::new( test_dir.to_str().unwrap(), binary_name);

        // Mock the Command executions (e.g., wget and chmod) using tools like `assert_cmd` or by wrapping Command calls
        let result = installer.install();

        assert!(result.is_ok(), "Installation should succeed");
        assert!(test_dir.join(binary_name).exists(), "Binary should be downloaded");

        // Cleanup
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
    }
}
