mod build_support;
use build_support::{cargo, skia};

/// Environment variables used by this build script.
mod env {
    use crate::build_support::cargo;
    use std::path::PathBuf;

    /// The path to the Skia source directory.
    pub fn offline_source_dir() -> Option<PathBuf> {
        cargo::env_var("SKIA_OFFLINE_SOURCE_DIR").map(PathBuf::from)
    }

    /// The full path of the ninja command to run. Only relevent when SKIA_OFFLINE_SOURCE_DIR is set.
    pub fn offline_ninja_command() -> Option<PathBuf> {
        cargo::env_var("SKIA_OFFLINE_NINJA_COMMAND").map(PathBuf::from)
    }

    pub fn offline_gn_command() -> Option<PathBuf> {
        cargo::env_var("SKIA_OFFLINE_GN_COMMAND").map(PathBuf::from)
    }
}

fn main() {
    // since 0.25.0
    if cfg!(feature = "svg") {
        cargo::warning("The feature 'svg' has been removed. SVG and XML support is available in all build configurations.");
    }
    // since 0.25.0
    if cfg!(feature = "shaper") {
        cargo::warning("The feature 'shaper' has been removed. To use the SkShaper bindings, enable the feature 'textlayout'.");
    }

    let build_config = skia::BuildConfiguration::default();
    let binaries_config = skia::BinariesConfiguration::from_cargo_env(&build_config);

    let gn_command = which::which("gn").ok();
    let ninja_command = which::which("ninja").ok();

    //
    // offline build?
    //
    if let Some(offline_source_dir) = env::offline_source_dir() {
        println!("STARTING OFFLINE BUILD");

        let final_configuration = skia::FinalBuildConfiguration::from_build_configuration(
            &build_config,
            &offline_source_dir,
        );

        skia::build_offline(
            &final_configuration,
            &binaries_config,
            env::offline_ninja_command()
                .as_deref()
                .or(ninja_command.as_deref()),
            env::offline_gn_command()
                .as_deref()
                .or(gn_command.as_deref()),
        );
    } else {
        //
        // full build?
        //

        println!("STARTING A FULL BUILD");
        let final_configuration = skia::FinalBuildConfiguration::from_build_configuration(
            &build_config,
            &std::env::current_dir().unwrap().join("skia"),
        );
        skia::build(
            &final_configuration,
            &binaries_config,
            ninja_command.as_deref(),
            gn_command.as_deref(),
        );
    };

    binaries_config.commit_to_cargo();
}
