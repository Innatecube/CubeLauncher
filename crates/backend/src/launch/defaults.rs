use schema::{
    backend_config::BackendConfig,
    instance::{
        InstanceConfiguration, InstanceJvmFlagsConfiguration, InstanceMemoryConfiguration,
    },
};

/// Detects total physical system memory in megabytes (MB).
pub fn get_system_total_memory_mb() -> Option<u32> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut status = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        };
        if unsafe { GlobalMemoryStatusEx(&mut status) }.is_ok() {
            return Some((status.ullTotalPhys / (1024 * 1024)) as u32);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    if let Some(kb) = rest.split_whitespace().next().and_then(|s| s.parse::<u64>().ok()) {
                        return Some((kb / 1024) as u32);
                    }
                }
            }
        }
    }
    None
}

/// Computes the safe default RAM allocation cap in MB.
/// Safely capped between 4GB (4096MB) and 6GB (6144MB) depending on system specs,
/// preventing long GC pauses caused by massive heap bloat.
pub fn get_smart_memory_limit_mb() -> u32 {
    if let Some(total_mb) = get_system_total_memory_mb() {
        if total_mb >= 14000 {
            // >= 16 GB system RAM -> safe 6 GB cap (6144 MB)
            6144
        } else if total_mb >= 7000 {
            // 8 GB - 12 GB system RAM -> safe 4 GB cap (4096 MB)
            4096
        } else {
            // Low memory system -> 3 GB - 4 GB
            total_mb.saturating_sub(2048).clamp(2048, 4096)
        }
    } else {
        4096
    }
}

pub fn apply_global_launch_defaults(instance: &mut InstanceConfiguration, global: &BackendConfig) {
    let instance_memory_enabled = instance.memory.as_ref().is_some_and(|memory| memory.enabled);
    if !instance_memory_enabled {
        if let Some(memory) = &global.memory
            && memory.enabled
        {
            instance.memory = Some(*memory);
        } else {
            let safe_max = get_smart_memory_limit_mb();
            instance.memory = Some(InstanceMemoryConfiguration {
                enabled: true,
                min: 1024,
                max: safe_max,
            });
        }
    }

    let instance_binary_enabled = instance.jvm_binary.as_ref().is_some_and(|binary| binary.enabled);
    if !instance_binary_enabled {
        if let Some(jvm_binary) = &global.jvm_binary
            && jvm_binary.enabled
        {
            instance.jvm_binary = Some(jvm_binary.clone());
        }
    }

    let global_flags = global
        .jvm_flags
        .as_ref()
        .filter(|flags| flags.enabled && !flags.flags.trim_ascii().is_empty());
    let instance_flags = instance
        .jvm_flags
        .as_ref()
        .filter(|flags| flags.enabled && !flags.flags.trim_ascii().is_empty());

    match (global_flags, instance_flags) {
        (Some(global), Some(instance_flags)) => {
            instance.jvm_flags = Some(InstanceJvmFlagsConfiguration {
                enabled: true,
                flags: format!("{} {}", global.flags, instance_flags.flags).into(),
            });
        },
        (Some(global), None) => {
            instance.jvm_flags = Some(global.clone());
        },
        _ => {},
    }
}

/// Checks if a Minecraft version string corresponds to version 1.21, 1.20.5+, or newer.
pub fn is_version_1_21_or_above(id: &str) -> bool {
    if let Some(rest) = id.strip_prefix("1.") {
        let parts: Vec<&str> = rest.split('.').collect();
        if let Some(minor_str) = parts.get(0) {
            if let Ok(minor) = minor_str.parse::<u32>() {
                if minor > 20 {
                    return true;
                }
                if minor == 20 {
                    if let Some(patch_str) = parts.get(1) {
                        if let Ok(patch) = patch_str.parse::<u32>() {
                            return patch >= 5;
                        }
                    }
                }
            }
        }
    }
    // Snapshots: e.g. 24w..., 25w...
    if id.len() >= 3 && id.as_bytes()[2] == b'w' {
        if let Ok(year) = id[..2].parse::<u32>() {
            return year >= 24;
        }
    }
    false
}

/// Checks if a Minecraft version string corresponds to version 1.20 or newer.
pub fn is_version_1_20_or_above(id: &str) -> bool {
    if let Some(rest) = id.strip_prefix("1.") {
        if let Some(minor_str) = rest.split('.').next() {
            if let Ok(minor) = minor_str.parse::<u32>() {
                return minor >= 20;
            }
        }
    }
    // Snapshots: e.g. 23w..., 24w..., 25w...
    if id.len() >= 3 && id.as_bytes()[2] == b'w' {
        if let Ok(year) = id[..2].parse::<u32>() {
            return year >= 23;
        }
    }
    false
}