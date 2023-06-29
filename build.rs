extern crate winreg;
#[macro_use] extern crate failure;

use failure::Error;
use std::{
    env::var,
    path::{Path, PathBuf},
};
use winreg::{enums::*, RegKey};

fn get_windows_kits_dir() -> Result<PathBuf, Error> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = r"SOFTWARE\Microsoft\Windows Kits\Installed Roots";
    let dir: String = hklm.open_subkey(key)?.get_value("KitsRoot10")?;

    Ok(dir.into())
}

fn get_km_dir(windows_kits_dir: &Path) -> Result<PathBuf, Error> {
    let readdir = Path::new(windows_kits_dir).join("lib").read_dir()?;

    let max_libdir = readdir
        .filter_map(|dir| dir.ok())
        .map(|dir| dir.path())
        .filter(|dir| {
            dir.components()
                .last()
                .and_then(|c| c.as_os_str().to_str())
                .map(|c| c.starts_with("10.") && dir.join("km").is_dir())
                .unwrap_or(false)
        })
        .max()
        .ok_or_else(|| format_err!("Can not find a valid km dir in `{:?}`", windows_kits_dir))?;

    Ok(max_libdir.join("km"))
}

fn internal_link_search() {
    let windows_kits_dir = get_windows_kits_dir().unwrap();

    let km_dir = get_km_dir(&windows_kits_dir).unwrap();

    let target = var("TARGET").unwrap();

    let arch = if target.contains("x86_64") {
        "x64"
    } else if target.contains("i686") {
        "x86"
    } else if target.contains("aarch64") {
        "arm64"
    } else {
        panic!("Only support x86_64, i686 and aarch64!");
    };

    let lib_dir = km_dir.join(arch);
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_str().unwrap()
    );
}

fn extra_link_search() {}

fn main() {
    if var(format!(
        "CARGO_FEATURE_{}",
        "extra_link_search".to_uppercase()
    ))
    .is_ok()
    {
        extra_link_search()
    } else {
        internal_link_search()
    }
}
