use byte_unit::Byte;

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

#[derive(Debug)]
pub struct System {
    pub hostname:    String,
    pub os:          String,
    pub os_version:  String,
    pub system_name: String,
    pub cpu:         CPU,
    pub memory:      Memory,
}

impl System {
    pub fn get_info() -> Self {
        let mut sys = sysinfo::System::new_all();

        // First we update all information of our `System` struct.
        sys.refresh_all();

        dbg!(&sys);

        let unknown = || "Unknown".to_string();

        Self {
            hostname:    sysinfo::System::host_name().unwrap_or_else(unknown),
            os:          sysinfo::System::name().unwrap_or_else(unknown),
            os_version:  sysinfo::System::os_version().unwrap_or_else(unknown),
            system_name: sysinfo::System::long_os_version().unwrap_or_else(unknown),
            cpu:         CPU {
                cores:          sys.cpus().len(),
                physical_cores: sysinfo::System::physical_core_count().unwrap_or_default(),
            },
            memory:      Memory {
                total:     sys.total_memory(),
                free:      sys.free_memory(),
                available: sys.available_memory(),
            },
        }
    }

    pub fn dump(&self) -> String {
        format!(
            r"
Hostname: {}
OS: {} {}
System: {}
CPU cores: {}/{}
Memory: total - {}, free - {}, available - {}
        ",
            self.hostname,
            self.os,
            self.os_version,
            self.system_name,
            self.cpu.cores,
            self.cpu.physical_cores,
            display_size(self.memory.total),
            display_size(self.memory.free),
            display_size(self.memory.available)
        )
    }
}

fn display_size(size: u64) -> String {
    let bytes = Byte::from_u64(size);

    let adjusted_byte = bytes.get_appropriate_unit(byte_unit::UnitType::Decimal);

    format!("{adjusted_byte:.2}")
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::System;

    #[wasm_bindgen_test(unsupported = test)]
    fn test_sysinfo() {
        println!("{}", System::get_info().dump());
    }
}
