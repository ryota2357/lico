use super::*;
use crate::object::RustFunction;

pub struct Module {
    executable: Executable,
    default_rfuns: Box<[(&'static str, RustFunction)]>,
    source_info: SourceInfo,
}

impl Module {
    pub const fn new(
        executable: Executable,
        default_rfuns: Box<[(&'static str, RustFunction)]>,
        source_info: SourceInfo,
    ) -> Self {
        Self {
            executable,
            default_rfuns,
            source_info,
        }
    }

    pub fn default_rfuncs(&self) -> &[(&'static str, RustFunction)] {
        &self.default_rfuns
    }

    pub fn executable(&self) -> &Executable {
        &self.executable
    }

    pub fn source_info(&self) -> &SourceInfo {
        &self.source_info
    }
}
