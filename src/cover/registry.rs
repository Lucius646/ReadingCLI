use rand::seq::IndexedRandom;

use super::CoverModule;

pub type CoverModuleFactory = fn() -> Box<dyn CoverModule>;

struct RegisteredModule {
    name: &'static str,
    factory: CoverModuleFactory,
}

#[derive(Default)]
pub struct CoverRegistry {
    modules: Vec<RegisteredModule>,
}

impl CoverRegistry {
    /// 创建一个空的模块注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个模块名称及其构造函数。
    pub fn register(&mut self, name: &'static str, factory: CoverModuleFactory) {
        assert!(
            self.modules.iter().all(|module| module.name != name),
            "cover module `{name}` is already registered"
        );

        self.modules.push(RegisteredModule { name, factory });
    }

    /// 随机创建一个已注册模块的新实例。
    pub fn create_random(&self) -> Option<Box<dyn CoverModule>> {
        let mut rng = rand::rng();
        let module = self.modules.choose(&mut rng)?;

        Some((module.factory)())
    }

    /// 按注册顺序返回全部模块名称。
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.modules.iter().map(|module| module.name)
    }

    /// 判断注册表中是否没有模块。
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}
