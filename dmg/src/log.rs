#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    None = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
    Disassembly = 4,   
}

pub struct Logger {
    pub level: LogLevel,
}

impl Logger {

    pub fn log_info(&self, message: &str) {
        if self.level >= LogLevel::Info {
            println!("{}", message)
        }
    }

    pub fn log_warning(&self, message: &str) {
        if self.level >= LogLevel::Warning {
            println!("{}", message)
        }
    }

    pub fn log_error(&self, message: &str) {
        if self.level >= LogLevel::Error {
            println!("{}", message)
        }
    }

    pub fn log_disassembly(&self, message: &str) {
        if self.level >= LogLevel::Disassembly {
            println!("{}", message)
        }
    }

}



