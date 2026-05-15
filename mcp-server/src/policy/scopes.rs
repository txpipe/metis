use std::collections::BTreeSet;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scope {
    Discover,
    Debug,
    WorkloadsInstall,
    WorkloadsUpgrade,
    WorkloadsDelete,
    VaultRuntimeMetadata,
    VaultRuntimeRead,
    VaultRuntimeWrite,
    VaultOperatorMetadata,
    VaultOperatorWrite,
    VaultOperatorRead,
    Admin,
}

impl Scope {
    pub fn all() -> BTreeSet<Self> {
        [
            Self::Discover,
            Self::Debug,
            Self::WorkloadsInstall,
            Self::WorkloadsUpgrade,
            Self::WorkloadsDelete,
            Self::VaultRuntimeMetadata,
            Self::VaultRuntimeRead,
            Self::VaultRuntimeWrite,
            Self::VaultOperatorMetadata,
            Self::VaultOperatorWrite,
            Self::VaultOperatorRead,
            Self::Admin,
        ]
        .into_iter()
        .collect()
    }
}
