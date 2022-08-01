use std::io;
use std::io::BufReader;
use std::str::FromStr;

use anyhow::Context;
use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;
use zip::ZipArchive;

const PROP_LAYERS_INDEX: &str = "Spring-Boot-Layers-Index";
const PROP_CLASSPATH_INDEX: &str = "Spring-Boot-Classpath-Index";

#[derive(Debug, Clone, PartialEq)]
pub struct JarManifest {
    pub layers_index: String,
    pub classpath_index: String,
}

impl FromStr for JarManifest {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        JarManifest::from_lines(s.lines().map(String::from))
    }
}

impl JarManifest {
    pub fn from_reader<R: io::BufRead>(read: R) -> anyhow::Result<Self> {
        Self::from_lines(read.lines().take_while(|r| r.is_ok()).map(Result::unwrap))
    }

    pub fn from_zip<R: io::Read + io::Seek>(zip: &mut ZipArchive<R>) -> anyhow::Result<Self> {
        Self::from_reader(BufReader::new(
            zip.by_name("META-INF/MANIFEST.MF")
                .with_context(|| "Jar does not contain a Manifest")?,
        ))
        .with_context(|| "Failed to read Jar Manifest")
    }

    fn from_lines<R: Iterator<Item = String>>(mut iter: R) -> anyhow::Result<Self> {
        let (layers_index, classpath_index) = iter
            .fold_while(
                (None as Option<String>, None as Option<String>),
                |(layers_index, classpath_index), line| {
                    let result = if line.starts_with(PROP_LAYERS_INDEX) {
                        (
                            line.split_once(':')
                                .map(|x| x.1)
                                .map(str::trim_start)
                                .map(String::from),
                            classpath_index,
                        )
                    } else if line.starts_with(PROP_CLASSPATH_INDEX) {
                        (
                            layers_index,
                            line.split_once(':')
                                .map(|x| x.1)
                                .map(str::trim_start)
                                .map(String::from),
                        )
                    } else {
                        (layers_index, classpath_index)
                    };

                    if result.0.is_none() || result.1.is_none() {
                        Continue(result)
                    } else {
                        Done(result)
                    }
                },
            )
            .into_inner();

        Ok(Self {
            layers_index: layers_index.with_context(|| {
                format!("MANIFEST.MF missing '{}'; layered Jar?", PROP_LAYERS_INDEX)
            })?,
            classpath_index: classpath_index.with_context(|| {
                format!(
                    "MANIFEST.MF missing '{}'; layered Jar?",
                    PROP_CLASSPATH_INDEX
                )
            })?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MANIFEST: &str = "
Spring-Boot-Version: 2.7.1
Spring-Boot-Classes: BOOT-INF/classes/
Spring-Boot-Lib: BOOT-INF/lib/
Spring-Boot-Classpath-Index: BOOT-INF/classpath.idx
Spring-Boot-Layers-Index: BOOT-INF/layers.idx
";

    const INVALID_MANIFEST_NO_LAYERS: &str = "
Spring-Boot-Version: 2.7.1
Spring-Boot-Classes: BOOT-INF/classes/
Spring-Boot-Lib: BOOT-INF/lib/
Spring-Boot-Classpath-Index: BOOT-INF/classpath.idx
";

    const INVALID_MANIFEST_NO_CLASSPATH: &str = "
Spring-Boot-Version: 2.7.1
Spring-Boot-Classes: BOOT-INF/classes/
Spring-Boot-Lib: BOOT-INF/lib/
Spring-Boot-Layers-Index: BOOT-INF/layers.idx
";

    const INVALID_MANIFEST_NOT_LAYERED: &str = "
Manifest-Version: 1.0
Created-By: Maven JAR Plugin 3.2.2
";

    #[test]
    fn parse() {
        assert_eq!(
            JarManifest::from_str(VALID_MANIFEST).unwrap(),
            JarManifest {
                layers_index: "BOOT-INF/layers.idx".into(),
                classpath_index: "BOOT-INF/classpath.idx".into(),
            }
        );

        JarManifest::from_str(INVALID_MANIFEST_NOT_LAYERED)
            .expect_err("Should not be able to parse un-layered Jar");

        JarManifest::from_str(INVALID_MANIFEST_NO_LAYERS)
            .expect_err("Should not be able to parse Jar missing layers index");

        JarManifest::from_str(INVALID_MANIFEST_NO_CLASSPATH)
            .expect_err("Should not be able to parse Jar missing classpath index");
    }
}
