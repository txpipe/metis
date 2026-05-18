use std::fmt;

const MAX_PATH_LEN: usize = 512;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VaultPathKind {
    Runtime,
    Operator,
}

#[derive(Clone, Eq, PartialEq)]
pub struct VaultPath {
    kind: VaultPathKind,
    path: String,
}

impl VaultPath {
    pub fn runtime(path: &str) -> Result<Self, VaultPathError> {
        Self::parse(path, VaultPathKind::Runtime)
    }

    pub fn operator(path: &str) -> Result<Self, VaultPathError> {
        Self::parse(path, VaultPathKind::Operator)
    }

    pub fn kind(&self) -> VaultPathKind {
        self.kind
    }

    pub fn as_str(&self) -> &str {
        &self.path
    }

    fn parse(path: &str, kind: VaultPathKind) -> Result<Self, VaultPathError> {
        let path = path.trim();
        let expected_prefix = match kind {
            VaultPathKind::Runtime => "runtime/",
            VaultPathKind::Operator => "operator/",
        };

        if path.is_empty() || path.len() > MAX_PATH_LEN {
            return Err(VaultPathError::Invalid);
        }

        if !path.starts_with(expected_prefix) {
            return Err(VaultPathError::NotAllowed);
        }

        if path.starts_with('/') || path.ends_with('/') || path.contains("//") {
            return Err(VaultPathError::Invalid);
        }

        for segment in path.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(VaultPathError::Invalid);
            }

            if !segment.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
            }) {
                return Err(VaultPathError::Invalid);
            }
        }

        Ok(Self {
            kind,
            path: path.to_string(),
        })
    }
}

impl fmt::Debug for VaultPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VaultPath")
            .field("kind", &self.kind)
            .field("path", &self.path)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VaultPathError {
    #[error("Vault path is outside the allowed prefix")]
    NotAllowed,
    #[error("Vault path is invalid")]
    Invalid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_path_accepts_runtime_prefix() {
        let path = VaultPath::runtime("runtime/cardano-node/mainnet-bp/block-producer").unwrap();

        assert_eq!(path.kind(), VaultPathKind::Runtime);
        assert_eq!(
            path.as_str(),
            "runtime/cardano-node/mainnet-bp/block-producer"
        );
    }

    #[test]
    fn runtime_path_rejects_operator_or_traversal_paths() {
        assert!(matches!(
            VaultPath::runtime("operator/root"),
            Err(VaultPathError::NotAllowed)
        ));
        assert!(matches!(
            VaultPath::runtime("runtime/../operator/root"),
            Err(VaultPathError::Invalid)
        ));
        assert!(matches!(
            VaultPath::runtime("kv/data/runtime/foo"),
            Err(VaultPathError::NotAllowed)
        ));
    }
}
