use sysinfo::System;

#[derive(Debug)]
pub struct Sys {
    pub hostname: String,
    pub cpu:      CPU,
    pub memory:   Memory,
}

#[derive(Debug)]
pub struct CPU {
    cores:          usize,
    physical_cores: usize,
}

#[derive(Debug)]
pub struct Memory {
    pub total:     u64,
    pub free:      u64,
    pub available: u64,
}

impl Sys {
    pub fn get_info() -> Self {
        let mut sys = System::new_all();

        // First we update all information of our `System` struct.
        sys.refresh_all();

        dbg!(&sys);

        let unknown = || "Unknown".to_string();

        Self {
            hostname: System::host_name().unwrap_or_else(unknown),
            cpu:      CPU {
                cores:          sys.cpus().len(),
                physical_cores: sysinfo::System::physical_core_count().unwrap_or_default(),
            },
            memory:   Memory {
                total:     sys.total_memory(),
                free:      sys.free_memory(),
                available: sys.available_memory(),
            },
        }
    }
}

#[cfg(test)]
mod test {
    use sysinfo::System;
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::Sys;

    #[wasm_bindgen_test(unsupported = test)]
    fn test_sysinfo() {
        Sys::get_info();

        let mut sys = System::new_all();
        sys.refresh_all();

        println!("--- Device Information ---");

        // Hostname
        println!(
            "Hostname:          {:?}",
            System::host_name().unwrap_or_else(|| "Unknown".into())
        );

        // OS Info
        println!(
            "OS Name:           {:?}",
            System::name().unwrap_or_else(|| "Unknown".into())
        );
        println!(
            "OS Version:        {:?}",
            System::os_version().unwrap_or_else(|| "Unknown".into())
        );
        println!(
            "Kernel Version:    {:?}",
            System::kernel_version().unwrap_or_else(|| "Unknown".into())
        );

        // CPU Info
        // physical_core_count gives actual hardware cores
        println!("Total CPUs:        {}", sys.cpus().len());
        if let Some(cores) = sysinfo::System::physical_core_count() {
            println!("Physical Cores:    {}", cores);
        }

        // RAM Info (Returned in bytes, converting to GB)
        let total_ram = sys.total_memory();
        println!(
            "Total RAM:         {:.2} GB",
            total_ram as f64 / 1024.0 / 1024.0 / 1024.0
        );
        println!(
            "Used RAM:          {:.2} GB",
            sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0
        );

        // Device Name (Note: on some OSs this is the same as hostname)
        println!(
            "System Name:       {:?}",
            System::long_os_version().unwrap_or_else(|| "Unknown".into())
        );

        dbg!(&Sys::get_info());
    }
}
