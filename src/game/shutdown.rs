#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShutdownStage {
    #[default]
    None,
    Requested,
    Finalizing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShutdownFlow {
    pub stage: ShutdownStage,
}

impl ShutdownFlow {
    pub fn request_close(&mut self) {
        if self.stage == ShutdownStage::None {
            self.stage = ShutdownStage::Requested;
        }
    }

    pub fn mark_finalizing(&mut self) {
        if self.stage == ShutdownStage::Requested {
            self.stage = ShutdownStage::Finalizing;
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    unsafe extern "C" {
        pub fn mq_shutdown_game();
    }
}

pub fn shutdown_game() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        wasm::mq_shutdown_game();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::process::exit(0);
    }
}
