use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Artifact {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub classifier: Option<String>,
    pub extension: Option<String>,
}

impl Artifact {
    pub(crate) fn to_path(&self) -> PathBuf {
        PathBuf::from(format!(
            "{}/{}/{}/{}",
            &self.group_path().to_str().unwrap(),
            &self.artifact,
            &self.version,
            &self.artifact_filename()
        ))
    }
    fn group_path(&self) -> PathBuf {
        PathBuf::from(self.group.replace('.', "/"))
    }
    fn artifact_filename(&self) -> String {
        let classifier_fmt = match self.classifier {
            Some(ref class) => format!("-{classifier}", classifier = class),
            None => "".to_string(),
        };
        let extension_fmt = match self.extension {
            Some(ref extension) => extension.clone(),
            None => "jar".to_string(),
        };
        format!(
            "{artifact}-{version}{classifier}.{extension}",
            artifact = self.artifact,
            version = self.version,
            classifier = classifier_fmt,
            extension = extension_fmt
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArtifactParseError {
    BadNumberOfParts,
}

impl ToString for Artifact {
    fn to_string(&self) -> String {
        let mut strn = String::new();
        strn.push_str(&self.group);
        strn.push(':');
        strn.push_str(&self.artifact);
        strn.push(':');
        strn.push_str(&self.version);
        if let Some(ref classifier) = self.classifier {
            strn.push(':');
            strn.push_str(classifier);
        }
        if let Some(ref ext) = self.extension {
            strn.push('@');
            strn.push_str(ext);
        }
        strn
    }
}

impl FromStr for Artifact {
    type Err = ArtifactParseError;
    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('@').collect();
        let (s, ext): (&str, Option<String>) = match *parts.as_slice() {
            [s, ext] => (s, Some(ext.to_string())),
            _ => (s, None),
        };

        let parts = s.split(':');
        let parts: Vec<&str> = parts.collect();
        match *parts.as_slice() {
            [grp, art, ver] => Ok(Self {
                group: grp.into(),
                artifact: art.into(),
                version: ver.into(),
                classifier: None,
                extension: ext,
            }),
            [grp, art, ver, class] => Ok(Self {
                group: grp.into(),
                artifact: art.into(),
                version: ver.into(),
                classifier: Some(class.into()),
                extension: ext,
            }),
            _ => Err(ArtifactParseError::BadNumberOfParts),
        }
    }
}
