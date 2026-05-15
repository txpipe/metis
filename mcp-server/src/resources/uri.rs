#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SupernodeResourceUri<'a> {
    Status,
    ControlPlaneStatus,
    ExtensionCatalog,
    ExtensionCatalogEntry { extension_id: &'a str },
}

pub const STATUS_URI: &str = "supernode://status";
pub const CONTROL_PLANE_STATUS_URI: &str = "supernode://control-plane/status";
pub const EXTENSION_CATALOG_URI: &str = "supernode://extensions/catalog";

const EXTENSION_CATALOG_ENTRY_PREFIX: &str = "supernode://extensions/catalog/";

impl<'a> SupernodeResourceUri<'a> {
    pub fn parse(uri: &'a str) -> Option<Self> {
        match uri {
            STATUS_URI => Some(Self::Status),
            CONTROL_PLANE_STATUS_URI => Some(Self::ControlPlaneStatus),
            EXTENSION_CATALOG_URI => Some(Self::ExtensionCatalog),
            _ => uri
                .strip_prefix(EXTENSION_CATALOG_ENTRY_PREFIX)
                .filter(|extension_id| !extension_id.is_empty() && !extension_id.contains('/'))
                .map(|extension_id| Self::ExtensionCatalogEntry { extension_id }),
        }
    }
}

pub fn extension_catalog_entry_uri(extension_id: &str) -> String {
    format!("{EXTENSION_CATALOG_ENTRY_PREFIX}{extension_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_resource_uris() {
        assert_eq!(
            SupernodeResourceUri::parse(STATUS_URI),
            Some(SupernodeResourceUri::Status)
        );
        assert_eq!(
            SupernodeResourceUri::parse(CONTROL_PLANE_STATUS_URI),
            Some(SupernodeResourceUri::ControlPlaneStatus)
        );
        assert_eq!(
            SupernodeResourceUri::parse(EXTENSION_CATALOG_URI),
            Some(SupernodeResourceUri::ExtensionCatalog)
        );
        assert_eq!(
            SupernodeResourceUri::parse("supernode://extensions/catalog/cardano-node-relay"),
            Some(SupernodeResourceUri::ExtensionCatalogEntry {
                extension_id: "cardano-node-relay"
            })
        );
    }

    #[test]
    fn rejects_unknown_or_nested_resource_uris() {
        assert_eq!(SupernodeResourceUri::parse("supernode://unknown"), None);
        assert_eq!(
            SupernodeResourceUri::parse("supernode://extensions/catalog/"),
            None
        );
        assert_eq!(
            SupernodeResourceUri::parse(
                "supernode://extensions/catalog/cardano-node-relay/profile"
            ),
            None
        );
    }
}
