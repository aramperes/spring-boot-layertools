extern crate core;

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};
use clap::{command, value_parser, Arg, Command};
use yaml_rust::{Yaml, YamlLoader};
use zip::ZipArchive;

fn main() -> anyhow::Result<()> {
    let cmd = command!()
        .arg(
            Arg::with_name("jar")
                .required(true)
                .takes_value(true)
                .help("The layered Spring Boot jar to extract")
                .value_parser(value_parser!(PathBuf)),
        )
        .subcommand(Command::new("list").about("List layers from the jar that can be extracted"))
        .subcommand(
            Command::new("extract")
                .about("Extracts layers from the jar for image creation")
                .arg(
                    Arg::new("destination")
                        .help("The destination to extract files to")
                        .long("destination")
                        .default_value(".")
                        .takes_value(true)
                        .value_parser(value_parser!(PathBuf)),
                ),
        )
        .subcommand_required(true)
        .get_matches();

    let jar = cmd
        .get_one::<PathBuf>("jar")
        .with_context(|| "Missing jar")?;

    let map = mmarinus::Map::load(jar, mmarinus::Private, mmarinus::perms::Read)
        .with_context(|| "Failed to open jar with mmap")?;

    let zip = zip::ZipArchive::new(Cursor::new(map.as_ref()))
        .with_context(|| "Failed to open jar archive")?;

    match cmd.subcommand() {
        Some(("list", _)) => list(zip),
        Some(("extract", args)) => extract(
            zip,
            args.get_one::<PathBuf>("destination")
                .with_context(|| "invalid extract destination")?,
        ),
        _ => bail!("unexpected subcommand composition"),
    }
}

fn layers_yaml(zip: &mut ZipArchive<Cursor<&[u8]>>) -> anyhow::Result<Yaml> {
    let index = {
        let mut layers_idx = zip
            .by_name("BOOT-INF/layers.idx")
            .with_context(|| "Failed to open layer index")?;
        let mut layers = String::new();
        layers_idx
            .read_to_string(&mut layers)
            .with_context(|| "Failed to read layer index")?;
        layers
    };

    YamlLoader::load_from_str(&index)
        .with_context(|| "Failed to parse layer index")?
        .into_iter()
        .next()
        .with_context(|| "Invalid layer index yaml: expected 1 root")
}

fn list(mut zip: ZipArchive<Cursor<&[u8]>>) -> anyhow::Result<()> {
    layers_yaml(&mut zip)?
        .as_vec()
        .with_context(|| "Invalid layer index yaml: expected array")?
        .iter()
        .flat_map(|elem| elem.as_hash())
        .flat_map(|elem| elem.keys())
        .flat_map(|elem| elem.as_str())
        .for_each(|x| println!("{}", x));
    Ok(())
}

fn extract(mut zip: ZipArchive<Cursor<&[u8]>>, destination: &PathBuf) -> anyhow::Result<()> {
    std::fs::create_dir_all(destination)
        .with_context(|| "Failed to create destination directory")?;

    layers_yaml(&mut zip)?
        .as_vec()
        .with_context(|| "Invalid layer index yaml: expected array")?
        .iter()
        .flat_map(|elem| elem.as_hash())
        .flat_map(|layers| layers.iter())
        .flat_map(|(name, files)| {
            name.as_str().and_then(|name| {
                files
                    .as_vec()
                    .map(|files| {
                        files
                            .iter()
                            .flat_map(|file| file.as_str())
                            .map(String::from)
                            .collect::<Vec<String>>()
                    })
                    .or_else(|| Some(Vec::with_capacity(0)))
                    .map(|files| (name, files))
            })
        })
        .try_for_each(|(name, files)| extract_layer(&mut zip, destination, name, files))
}

fn extract_layer(
    zip: &mut ZipArchive<Cursor<&[u8]>>,
    destination: &PathBuf,
    layer: &str,
    files: Vec<String>,
) -> anyhow::Result<()> {
    let file_names: Vec<String> = zip.file_names().into_iter().map(String::from).collect();

    let layer_destination = destination.join(layer);
    anyhow::ensure!(
        layer_destination.starts_with(destination),
        "invalid layer name: potential malicious use of relative path"
    );
    std::fs::create_dir_all(&layer_destination).with_context(|| {
        format!(
            "failed to create layer destination directory: {:?}",
            layer_destination
        )
    })?;

    for entry in files.iter() {
        let is_directory = entry.ends_with('/');

        if is_directory {
            let output_path = layer_destination.join(entry);
            anyhow::ensure!(
                output_path.starts_with(&layer_destination),
                "invalid directory name: potential malicious use of relative path"
            );

            std::fs::create_dir_all(&output_path)?;

            for zip_entry in &file_names {
                let child_path = Path::new(zip_entry);
                if !zip_entry.ends_with('/') && child_path.starts_with(entry) {
                    let output_path = layer_destination.join(child_path);
                    if let Some(parent_path) = output_path.parent() {
                        if !parent_path.exists() {
                            std::fs::create_dir_all(parent_path)?;
                        }
                    }

                    let mut zip_file = zip.by_name(zip_entry).with_context(|| {
                        format!("unknown (child) file {} in layer {}", zip_entry, layer)
                    })?;

                    let mut output_file = std::fs::File::create(&output_path)?;
                    std::io::copy(&mut zip_file, &mut output_file)?;
                }
            }
        } else {
            let mut zip_file = zip
                .by_name(entry)
                .with_context(|| format!("unknown file {} in layer {}", entry, layer))?;

            let entry = zip_file
                .enclosed_name()
                .with_context(|| format!("failed to determine enclosed name of file: {}", entry))?;

            let output_path = layer_destination.join(entry);

            if let Some(parent_path) = output_path.parent() {
                if !parent_path.exists() {
                    std::fs::create_dir_all(parent_path)?;
                }
            }

            let mut output_file = std::fs::File::create(&output_path)?;
            std::io::copy(&mut zip_file, &mut output_file)?;
        }
    }

    Ok(())
}