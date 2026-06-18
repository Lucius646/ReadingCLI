mod cargo;
mod download;
mod weblog;

pub use cargo::CargoModule;
pub use download::DownloadModule;
pub use weblog::WeblogModule;

use super::CoverRegistry;

/// 创建并注册第一版支持的全部 Cover 模块。
pub fn default_registry() -> CoverRegistry {
    let mut registry = CoverRegistry::new();

    registry.register("cargo", || Box::new(CargoModule::new()));
    registry.register("weblog", || Box::new(WeblogModule::new()));
    registry.register("download", || Box::new(DownloadModule::new()));

    registry
}
