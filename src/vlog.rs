//! # vlog.rs
//!
//! An agile log tool that avoid third-parties dependencies.
//!
//! pub use vlog_debug as debug;
//! pub use vlog_info as info;
//! pub use vlog_warn as warn;
//! pub use vlog_error as error;
//! pub use vlog_success as success;

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
    Silent = 4,  // Higher than Error, suppresses all logging
}

static CURRENT_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Silent as u8);

pub fn set_level(level: LogLevel) {
    CURRENT_LEVEL.store(level as u8, Ordering::Relaxed);
}

pub fn should_log(level: LogLevel) -> bool {
    let current = CURRENT_LEVEL.load(Ordering::Relaxed);
    (level as u8) >= current
}

#[macro_export]
macro_rules! vlog_set_level {
    ($($arg:tt)*) => {
        $crate::vlog::set_level($($arg)*);
    };
}

#[macro_export]
macro_rules! vlog_debug {
    ($($arg:tt)*) => {
        if $crate::vlog::should_log($crate::vlog::LogLevel::Debug) {
            println!("🔍 [DEBUG] {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! vlog_info {
    ($($arg:tt)*) => {
        if $crate::vlog::should_log($crate::vlog::LogLevel::Info) {
            println!("ℹ️  [INFO]  {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! vlog_warn {
    ($($arg:tt)*) => {
        if $crate::vlog::should_log($crate::vlog::LogLevel::Warn) {
            eprintln!("⚠️  [WARN]  {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! vlog_error {
    ($($arg:tt)*) => {
        if $crate::vlog::should_log($crate::vlog::LogLevel::Error) {
            eprintln!("❌ [ERROR] {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! vlog_success {
    ($($arg:tt)*) => {
        if $crate::vlog::should_log($crate::vlog::LogLevel::Info) {
            println!("✅ [OK]    {}", format!($($arg)*));
        }
    };
}