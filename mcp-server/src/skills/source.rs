use crate::config::{SkillCatalogConfig, SkillCatalogSource};

use super::{SkillCatalog, SkillCatalogLoadError, oci};

pub async fn load_skill_catalog(
    config: &SkillCatalogConfig,
) -> Result<SkillCatalog, SkillCatalogLoadError> {
    match config.source {
        SkillCatalogSource::Bundled => Ok(SkillCatalog::bundled()),
        SkillCatalogSource::Oci => {
            let oci_ref = config
                .oci_ref
                .as_deref()
                .ok_or(SkillCatalogLoadError::MissingOciReference)?;
            if !config.allow_untrusted && !oci::is_trusted_catalog_reference(oci_ref)? {
                return Err(SkillCatalogLoadError::UntrustedCatalogReference(
                    oci_ref.to_string(),
                ));
            }
            let payload = oci::fetch_catalog_json(oci_ref, config.max_bytes).await?;
            let payload = std::str::from_utf8(&payload)?;

            SkillCatalog::from_json_str(payload)
        }
    }
}
