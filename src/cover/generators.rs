#![allow(dead_code)]

use std::cmp;
use std::path::{Path, PathBuf};

use rand::RngExt;
use rand::distr::Uniform;
use rand::rngs::ThreadRng;
use rand::seq::IndexedRandom;
use rand_distr::{ChiSquared, Distribution, Exp};

/// 从指定字符集中随机生成固定长度的字符串。
pub(crate) fn gen_string_with_chars(rng: &mut ThreadRng, char_set: &str, length: u64) -> String {
    let chars: Vec<_> = char_set.chars().collect();
    let range = Uniform::new(0, chars.len()).expect("character set must not be empty");

    (0..length).map(|_| chars[rng.sample(range)]).collect()
}

/// 生成由小写十六进制字符组成的字符串。
pub(crate) fn gen_hex_string(rng: &mut ThreadRng, length: u64) -> String {
    gen_string_with_chars(rng, "0123456789abcdef", length)
}

/// 从候选列表随机选取若干元素，并用空格拼接。
pub(crate) fn gen_random_n_from_list_into_string(
    rng: &mut ThreadRng,
    list: &[&str],
    n: u64,
) -> String {
    let range = Uniform::new(0, list.len()).expect("candidate list must not be empty");

    (0..cmp::min(n, list.len() as u64))
        .fold(String::new(), |acc, _| acc + " " + list[rng.sample(range)])
}

/// 从候选文件中随机选择名称，并替换为指定扩展名。
pub(crate) fn gen_file_name_with_ext(
    rng: &mut ThreadRng,
    files: &[&str],
    extension: &str,
) -> String {
    let chosen_file = files.choose(rng).unwrap_or(&"");
    let path = Path::new(chosen_file).with_extension(extension);

    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// 随机组合候选文件名和扩展名。
pub(crate) fn gen_file_name(rng: &mut ThreadRng, files: &[&str], extensions: &[&str]) -> String {
    let chosen_file = files.choose(rng).unwrap_or(&"");
    let chosen_extension = extensions.choose(rng).unwrap_or(&"");
    let path = Path::new(chosen_file).with_extension(chosen_extension);

    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// 使用候选目录、文件和扩展名生成随机绝对路径。
pub(crate) fn gen_file_path<T: Clone + AsRef<Path>>(
    rng: &mut ThreadRng,
    files: &[&str],
    extensions: &[&str],
    dir_candidates: &[T],
) -> String {
    let path_length = rng.random_range(1..5);
    let range = Uniform::new(0, dir_candidates.len()).expect("directory list must not be empty");
    let mut path = PathBuf::from("/");

    for _ in 0..path_length {
        path.push(dir_candidates[rng.sample(range)].clone());
    }

    path.push(gen_file_name(rng, files, extensions));
    path.to_string_lossy().into_owned()
}

/// 生成类似 `1.12.4` 的随机软件包版本号。
pub(crate) fn gen_package_version(rng: &mut ThreadRng) -> String {
    let major_distribution = Exp::new(2.0).expect("valid exponential distribution");
    let component_distribution = ChiSquared::new(1.0).expect("valid chi-squared distribution");

    format!(
        "{major:.0}.{minor:.0}.{patch:.0}",
        major = major_distribution.sample(rng),
        minor = 10.0 * component_distribution.sample(rng),
        patch = 10.0 * component_distribution.sample(rng),
    )
}

#[cfg(test)]
mod tests {
    use super::{gen_hex_string, gen_package_version};

    #[test]
    fn hex_generator_uses_the_requested_length_and_alphabet() {
        let value = gen_hex_string(&mut rand::rng(), 32);

        assert_eq!(value.len(), 32);
        assert!(value.chars().all(|character| character.is_ascii_hexdigit()));
    }

    #[test]
    fn package_version_has_three_numeric_components() {
        let version = gen_package_version(&mut rand::rng());
        let components: Vec<_> = version.split('.').collect();

        assert_eq!(components.len(), 3);
        assert!(
            components
                .iter()
                .all(|component| component.parse::<u64>().is_ok())
        );
    }
}
