use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::budget::Millis;
use crate::crate_name::CrateName;
use crate::job_name::JobName;
use crate::test_name::TestName;

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
        name: TestName,
    },
    Budget {
        test: TestName,
        cpu: Millis,
    },
    Golden {
        paths: Vec<PathBuf>,
    },
    Infra {
        job: JobName,
    },
}
