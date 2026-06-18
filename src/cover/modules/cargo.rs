use std::time::{Duration, Instant};

use rand::RngExt;
use rand::seq::IndexedRandom;

use crate::cover::data::PACKAGES_LIST;
use crate::cover::generators::gen_package_version;
use crate::cover::{CoverContext, CoverFrame, CoverModule, CoverOp};

struct Package {
    name: &'static str,
    version: String,
}

enum CargoStage {
    Downloading,
    Compiling,
    Finished,
    Done,
}

pub struct CargoModule {
    packages: Vec<Package>,
    stage: CargoStage,
    package_index: usize,
    started_at: Instant,
}

impl Default for CargoModule {
    fn default() -> Self {
        Self::new()
    }
}

impl CargoModule {
    /// 随机选择软件包和版本，并从下载阶段开始执行。
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let package_count = rng.random_range(10..100).min(PACKAGES_LIST.len());
        let packages = PACKAGES_LIST
            .sample(&mut rng, package_count)
            .map(|&name| Package {
                name,
                version: gen_package_version(&mut rng),
            })
            .collect();

        Self {
            packages,
            stage: CargoStage::Downloading,
            package_index: 0,
            started_at: Instant::now(),
        }
    }

    fn package_frame(&mut self, stage: &str) -> CoverFrame {
        let package = &self.packages[self.package_index];
        let line = format!("{stage:>12} {} v{}", package.name, package.version);
        self.package_index += 1;

        let delay = rand::rng().random_range(100..2000);
        CoverFrame::new(
            vec![CoverOp::Write(line), CoverOp::NewLine],
            Duration::from_millis(delay),
        )
    }
}

impl CoverModule for CargoModule {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn signature(&self) -> String {
        "cargo run".to_string()
    }

    fn next_frame(&mut self, _context: &CoverContext) -> Option<CoverFrame> {
        loop {
            match self.stage {
                CargoStage::Downloading if self.package_index < self.packages.len() => {
                    return Some(self.package_frame("Downloading"));
                }
                CargoStage::Downloading => {
                    self.stage = CargoStage::Compiling;
                    self.package_index = 0;
                }
                CargoStage::Compiling if self.package_index < self.packages.len() => {
                    return Some(self.package_frame("Compiling"));
                }
                CargoStage::Compiling => {
                    self.stage = CargoStage::Finished;
                    self.package_index = 0;
                }
                CargoStage::Finished => {
                    self.stage = CargoStage::Done;
                    let elapsed = self.started_at.elapsed().as_secs_f32();
                    let line = format!(
                        "{:>12} release [optimized] target(s) in {elapsed:.2} secs",
                        "Finished"
                    );

                    return Some(CoverFrame::new(
                        vec![CoverOp::Write(line), CoverOp::NewLine],
                        Duration::ZERO,
                    ));
                }
                CargoStage::Done => return None,
            }
        }
    }
}
