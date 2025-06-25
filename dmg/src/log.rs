use crate::gb::GameBoy;
#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    None = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
    Disassembly = 4,   
}

impl GameBoy {

    pub fn log_info(&self, message: &str) {
        if self.log_level >= LogLevel::Info {
            println!("{}", message)
        }
    }

    pub fn log_warning(&self, message: &str) {
        if self.log_level >= LogLevel::Warning {
            println!("{}", message)
        }
    }

    pub fn log_error(&self, message: &str) {
        if self.log_level >= LogLevel::Error {
            println!("{}", message)
        }
    }

    pub fn log_disassembly(&self, message: &str) {
        if self.log_level >= LogLevel::Disassembly {
            println!("{}", message)
        }
    }

}



