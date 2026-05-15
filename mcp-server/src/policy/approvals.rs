use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalClass {
    Discovery,
    ReadOnlyDebug,
    SensitiveRuntimeRead,
    RuntimeSecretWrite,
    OperatorSecretGuidance,
    OperatorSecretBreakGlass,
    Mutation,
    Destructive,
    LedgerPrepare,
    LedgerSubmit,
}

impl ApprovalClass {
    pub fn all() -> &'static [Self] {
        &[
            Self::Discovery,
            Self::ReadOnlyDebug,
            Self::SensitiveRuntimeRead,
            Self::RuntimeSecretWrite,
            Self::OperatorSecretGuidance,
            Self::OperatorSecretBreakGlass,
            Self::Mutation,
            Self::Destructive,
            Self::LedgerPrepare,
            Self::LedgerSubmit,
        ]
    }

    pub fn requires_approval(self) -> bool {
        match self {
            Self::Discovery | Self::ReadOnlyDebug | Self::OperatorSecretGuidance => false,
            Self::SensitiveRuntimeRead
            | Self::RuntimeSecretWrite
            | Self::OperatorSecretBreakGlass
            | Self::Mutation
            | Self::Destructive
            | Self::LedgerPrepare
            | Self::LedgerSubmit => true,
        }
    }
}
