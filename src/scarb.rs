use std::{env::current_dir, fs, path::PathBuf};

use anyhow::Context;
use cairo_lang_runner::{RunResultValue, SierraCasmRunner, StarknetState};
use cairo_lang_sierra::program::VersionedProgram;
use cairo_lang_test_plugin::TestCompilation;
use cairo_lang_test_runner::{CompiledTestRunner, TestRunConfig};
use camino::Utf8PathBuf;

use itertools::Itertools;
use scarb::{
    core::{Config, TargetKind},
    ops::{self, collect_metadata, CompileOpts, MetadataOptions},
};

// Prepares testing crate
// Copies the exercise file into testing crate
pub fn prepare_crate_for_exercise(file_path: &PathBuf) -> PathBuf {
    let crate_path = current_dir().unwrap().join(PathBuf::from("runner-crate"));
    let src_dir = crate_path.join("src");
    if !src_dir.exists() {
        let _ = fs::create_dir(&src_dir);
    }
    let lib_path = src_dir.join("lib.cairo");
    let file_path = current_dir().unwrap().join(file_path);

    match fs::copy(&file_path, &lib_path) {
        Ok(_) => {}
        Err(err) => panic!("Error occurred while preparing the exercise,\nExercise: {file_path:?}\nLib path: {lib_path:?}\n{err:?}"),
    };
    crate_path
}

// Builds the testing crate with scarb
pub fn scarb_build(file_path: &PathBuf) -> anyhow::Result<String> {
    let crate_path = prepare_crate_for_exercise(file_path);
    let config = scarb_config(crate_path);

    match compile(&config, false) {
        Ok(_) => Ok("".into()),
        Err(_) => anyhow::bail!("Couldn't build the exercise..."),
    }
}

// Runs the crate with scarb
pub fn scarb_run(file_path: &PathBuf) -> anyhow::Result<String> {
    let crate_path = prepare_crate_for_exercise(file_path);
    let config = scarb_config(crate_path);

    let ws = ops::read_workspace(config.manifest_path(), &config)?;

    // Compile before running tests, with test targets true
    compile(&config, false)?;

    let metadata = collect_metadata(
        &MetadataOptions {
            version: 1,
            no_deps: false,
        },
        &ws,
    )?;

    let profile = "dev";
    let default_target_dir = metadata.runtime_manifest.join("target");

    let target_dir = metadata
        .target_dir
        .clone()
        .unwrap_or(default_target_dir)
        .join(profile);

    // Process 'exercise_crate' targets
    // Largely same as this
    // https://github.com/software-mansion/scarb/blob/50e5d942f72a7b756c36fdc57b7899ad8b6ff7c7/extensions/scarb-cairo-run/src/main.rs#L61
    for package in metadata.packages.iter() {
        if package.name != "exercise_crate" {
            continue;
        }
        // Loop through targets and run compiled file tests
        for target in package.targets.iter() {
            // Skip test targets
            if target.kind == "test" {
                continue;
            }

            let file_path = target_dir.join(format!("{}.sierra.json", target.name.clone()));

            assert!(
                file_path.exists(),
                "File {file_path} missing, please compile the project."
            );

            let sierra_program = serde_json::from_str::<VersionedProgram>(
                &fs::read_to_string(file_path.clone())
                    .with_context(|| format!("failed to read Sierra file: {file_path}"))?,
            )
            .with_context(|| format!("failed to deserialize Sierra program: {file_path}"))?
            .into_v1()
            .with_context(|| format!("failed to load Sierra program: {file_path}"))?;

            let runner = SierraCasmRunner::new(sierra_program.program, None, Default::default())?;

            let result = runner
                .run_function_with_starknet_context(
                    runner.find_function("::main")?,
                    &[],
                    None,
                    StarknetState::default(),
                )
                .context("failed to run the function")?;

            return match result.value {
                RunResultValue::Success(return_val) => {
                    Ok(return_val.iter().map(|el| el.to_string()).join(","))
                }
                RunResultValue::Panic(error) => {
                    anyhow::bail!(format!("error running the code, {:?}", error))
                }
            };
        }
    }

    Ok("".into())
}

// Runs tests on the testing crate with scarb
pub fn scarb_test(file_path: &PathBuf) -> anyhow::Result<String> {
    let crate_path = prepare_crate_for_exercise(file_path);
    let config = scarb_config(crate_path);

    let ws = ops::read_workspace(config.manifest_path(), &config)?;

    // Compile before running tests, with test targets true
    compile(&config, true)?;

    let metadata = collect_metadata(
        &MetadataOptions {
            version: 1,
            no_deps: false,
        },
        &ws,
    )
    .unwrap();

    let profile = "dev";
    let default_target_dir = metadata.runtime_manifest.join("target");

    let target_dir = metadata
        .target_dir
        .clone()
        .unwrap_or(default_target_dir)
        .join(profile);

    // Loop through packages, but only process 'exercise_crate'
    // Largely same as this
    // https://github.com/software-mansion/scarb/blob/ff98a787cfc0d94adcc7394fa83348bc01f437d5/extensions/scarb-cairo-test/src/main.rs#L54
    for package in metadata.packages.iter() {
        if package.name != "exercise_crate" {
            continue;
        }
        // Loop through targets and run compiled file tests
        for target in package.targets.iter() {
            if target.kind != "test" {
                continue;
            }
            let file_path = target_dir.join(format!("{}.test.json", target.name.clone()));
            let test_compilation = serde_json::from_str::<TestCompilation>(
                &fs::read_to_string(file_path.clone())
                    .with_context(|| format!("failed to read file: {file_path}"))?,
            )
            .with_context(|| format!("failed to deserialize compiled tests file: {file_path}"))?;

            let config = TestRunConfig {
                filter: "".into(),
                include_ignored: false,
                ignored: false,
            };
            let runner = CompiledTestRunner::new(test_compilation, config);
            runner.run()?;
            println!();
        }
    }

    anyhow::Ok("".into())
}

// Prepares scarb config for exercise runner crate
pub fn scarb_config(crate_path: PathBuf) -> Config {
    let path = Utf8PathBuf::from_path_buf(crate_path.join(PathBuf::from("Scarb.toml"))).unwrap();

    Config::builder(path).build().unwrap()
}

// Compiles runner crate for build/test exercises
pub fn compile(config: &Config, test_targets: bool) -> anyhow::Result<()> {
    let ws = ops::read_workspace(config.manifest_path(), config)?;
    let opts: CompileOpts = match test_targets {
        false => CompileOpts {
            include_targets: vec![],
            exclude_targets: vec![TargetKind::TEST],
        },
        true => CompileOpts {
            include_targets: vec![TargetKind::TEST],
            exclude_targets: vec![],
        },
    };

    let packages = ws.members().map(|p| p.id).collect();

    ops::compile(packages, opts, &ws)
}
