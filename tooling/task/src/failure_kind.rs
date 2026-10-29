use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::budget::Millis;
use crate::check_name::CheckName;
use crate::crate_name::CrateName;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum FailureKind {
    Compile {
        paths: Vec<PathBuf>,
    },
    Lint {
        paths: Vec<PathBuf>,
    },
    Test {
        #[serde(rename = "crate")]
        crate_name: CrateName,
        name: CheckName,
    },
    Budget {
        test: CheckName,
        cpu: Millis,
    },
    Golden {
        paths: Vec<PathBuf>,
    },
    Infra {
        job: CheckName,
    },
}
