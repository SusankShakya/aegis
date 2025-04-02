//! Platform abstraction traits and utilities
//!
//! This module defines platform-agnostic traits that are implemented 
//! by platform-specific modules, enabling the Aegis framework to run
//! across various operating systems and environments.

use async_trait::async_trait;
use std::path::Path;
use crate::error::AegisResult;

/// Trait for file system operations
#[async_trait]
pub trait FileSystem: Send + Sync {
    /// Read the contents of a file as bytes
    async fn read_file(&self, path: &Path) -> AegisResult<Vec<u8>>;
    
    /// Write bytes to a file
    async fn write_file(&self, path: &Path, contents: &[u8]) -> AegisResult<()>;
    
    /// Check if a file exists
    async fn file_exists(&self, path: &Path) -> AegisResult<bool>;
    
    /// Create a directory and all parent directories if needed
    async fn create_dir_all(&self, path: &Path) -> AegisResult<()>;
    
    /// Remove a file
    async fn remove_file(&self, path: &Path) -> AegisResult<()>;
    
    /// Remove a directory and all its contents
    async fn remove_dir_all(&self, path: &Path) -> AegisResult<()>;
}

/// Trait for process management
#[async_trait]
pub trait ProcessManager: Send + Sync {
    /// Spawn a new process
    async fn spawn(&self, command: &str, args: &[&str]) -> AegisResult<ProcessHandle>;
    
    /// Get the current process ID
    fn get_pid(&self) -> AegisResult<u32>;
    
    /// Kill a process by its handle
    async fn kill(&self, handle: &ProcessHandle) -> AegisResult<()>;
}

/// Handle to a spawned process
#[derive(Debug, Clone)]
pub struct ProcessHandle {
    /// Process ID
    pub pid: u32,
    
    /// Process status
    pub status: ProcessStatus,
}

/// Process status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is running
    Running,
    /// Process has exited
    Exited(i32),
    /// Process was terminated by a signal
    Terminated(i32),
    /// Process status is unknown
    Unknown,
}

/// Trait for network operations
#[async_trait]
pub trait Network: Send + Sync {
    /// Open a TCP connection
    async fn connect_tcp(&self, host: &str, port: u16) -> AegisResult<Box<dyn TcpStream>>;
    
    /// Create a TCP listener
    async fn listen_tcp(&self, host: &str, port: u16) -> AegisResult<Box<dyn TcpListener>>;
    
    /// Resolve a hostname to IP addresses
    async fn resolve_host(&self, host: &str) -> AegisResult<Vec<String>>;
}

/// Trait for TCP stream
#[async_trait]
pub trait TcpStream: Send + Sync {
    /// Read data from the stream
    async fn read(&mut self, buf: &mut [u8]) -> AegisResult<usize>;
    
    /// Write data to the stream
    async fn write(&mut self, buf: &[u8]) -> AegisResult<usize>;
    
    /// Close the stream
    async fn close(&mut self) -> AegisResult<()>;
}

/// Trait for TCP listener
#[async_trait]
pub trait TcpListener: Send + Sync {
    /// Accept a new connection
    async fn accept(&self) -> AegisResult<Box<dyn TcpStream>>;
    
    /// Close the listener
    async fn close(&mut self) -> AegisResult<()>;
}

/// Trait for environment information
pub trait Environment: Send + Sync {
    /// Get environment variable value
    fn get_env(&self, name: &str) -> Option<String>;
    
    /// Get all environment variables
    fn get_all_env(&self) -> AegisResult<std::collections::HashMap<String, String>>;
    
    /// Get current working directory
    fn current_dir(&self) -> AegisResult<std::path::PathBuf>;
    
    /// Get system information
    fn system_info(&self) -> AegisResult<SystemInfo>;
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Operating system name
    pub os_name: String,
    
    /// Operating system version
    pub os_version: String,
    
    /// CPU architecture
    pub architecture: String,
    
    /// Number of CPU cores
    pub cpu_cores: u32,
    
    /// Total system memory in bytes
    pub total_memory: u64,
}

/// Platform factory trait for creating platform-specific implementations
pub trait PlatformFactory: Send + Sync {
    /// Create a file system implementation
    fn create_filesystem(&self) -> Box<dyn FileSystem>;
    
    /// Create a process manager implementation
    fn create_process_manager(&self) -> Box<dyn ProcessManager>;
    
    /// Create a network implementation
    fn create_network(&self) -> Box<dyn Network>;
    
    /// Create an environment implementation
    fn create_environment(&self) -> Box<dyn Environment>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // These tests will require mock implementations of the traits
    // They will be implemented when the actual platform-specific implementations are created
} 