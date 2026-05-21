use crate::config::{ExtensionCatalogConfig, ExtensionCatalogSource};

use super::{CatalogLoadError, ExtensionCatalog, oci};

pub async fn load_catalog(
    config: &ExtensionCatalogConfig,
) -> Result<ExtensionCatalog, CatalogLoadError> {
    match config.source {
        ExtensionCatalogSource::Bundled => Ok(ExtensionCatalog::bundled()),
        ExtensionCatalogSource::Oci => {
            let oci_ref = config
                .oci_ref
                .as_deref()
                .ok_or(CatalogLoadError::MissingOciReference)?;
            if !config.allow_untrusted && !oci::is_trusted_catalog_reference(oci_ref)? {
                return Err(CatalogLoadError::UntrustedCatalogReference(
                    oci_ref.to_string(),
                ));
            }
            let payload = oci::fetch_catalog_json(oci_ref, config.max_bytes).await?;
            let payload = std::str::from_utf8(&payload)?;

            ExtensionCatalog::from_json_str_with_trust(payload, config.allow_untrusted)
        }
    }
}
