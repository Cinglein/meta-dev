use derive_more::{Display, From};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Display, From, Serialize, Deserialize)]
#[serde(transparent)]
#[from(&str, String)]
pub struct JobName(String);
