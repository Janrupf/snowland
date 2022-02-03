use std::env::VarError;

const PLATFORM_FEATURE_LINUX: &str = "snowland_workflow_linux_build";
const PLATFORM_FEATURE_WINDOWS: &str = "snowland_workflow_win_build";

fn main() {
    // We need to be able to invoke cargo in the workflow crate
    bake_environment_variable("CARGO");
    bake_environment_variable("CARGO_MANIFEST_DIR");

    let var_res = std::env::var("SNOWLAND_TARGET_PLATFORM");

    let target_feature = match var_res.as_ref().map(|s| s.as_str()) {
        Ok("linux") => PLATFORM_FEATURE_LINUX,
        Ok("windows") => PLATFORM_FEATURE_WINDOWS,
        Ok(v) => panic!("Unknown target platform {}", v),

        #[cfg(target_os = "windows")]
        Err(err) if err == &VarError::NotPresent => PLATFORM_FEATURE_WINDOWS,

        #[cfg(target_os = "linux")]
        Err(err) if err == &VarError::NotPresent => PLATFORM_FEATURE_LINUX,

        Err(err) => panic!(
            "Failed to read environment variable SNOWLAND_TARGET_PLATFORM: {}",
            err
        ),
    };

    set_cfg(target_feature, None);
}

fn bake_environment_variable(key: &str) {
    let value =
        std::env::var(key).unwrap_or_else(|_| panic!("Missing environment variable {}", key));
    set_environment_variable(&format!("SNOWLAND_WORKFLOW_{}", key), &value);
}

fn set_environment_variable(key: &str, value: &str) {
    println!("cargo:rustc-env={}={}", key, value);
}

fn set_cfg(key: &str, value: Option<&str>) {
    if let Some(v) = value {
        println!("cargo:rustc-cfg={}={}", key, v);
    } else {
        println!("cargo:rustc-cfg={}", key);
    }
}
