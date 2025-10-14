use common::pb::{MachineInfo, MachineDynamicInfo};
use sysinfo::System;
use sysinfo::Disks;

pub fn collect_dynamic_info(sys:&mut System, uuid: &str) -> MachineDynamicInfo {
    let cpus = sys.cpus();
    let mut cpu_usage = std::collections::HashMap::new();
    for cpu in cpus {
        cpu_usage.insert(
            cpu.name().to_string(),
            cpu.cpu_usage(),
        );
    }
    MachineDynamicInfo {
        uuid: uuid.to_string(),
        cpu_usage: cpu_usage,
    }
}

pub fn collect_static_info(uuid: Option<String>) -> MachineInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    let host_name = System::host_name();
    let system_name = System::name();
    let kernel_version = System::kernel_version();
    let os_version = System::os_version();

    let cpu_cores_num = System::physical_core_count()
        .unwrap_or_else(|| sys.cpus().len());

    // sysinfo returns KiB for memory APIs; convert to bytes for clarity
    let total_memory_bytes = sys.total_memory().saturating_mul(1024);
    let total_swap_bytes = sys.total_swap().saturating_mul(1024);

    let disks = Disks::new_with_refreshed_list();

    // count total space of all disks
    let mut total_space = 0;
    for disk in disks.list() {
        total_space += disk.total_space();
    }
    MachineInfo {
        uuid: uuid,
        host_name: host_name.unwrap(),
        system_name: system_name.unwrap(),
        ip_address: "".to_string(),
        kernel_version: kernel_version.unwrap(),
        os_version: os_version.unwrap(),
        cpu_cores: cpu_cores_num as u64,
        total_memory: total_memory_bytes,
        total_swap: total_swap_bytes,
        total_disk: total_space,
    }
}


