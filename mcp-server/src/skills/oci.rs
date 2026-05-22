use crate::oci_client::{self, OciArtifactError};

const SKILL_CATALOG_LAYER_MEDIA_TYPE: &str = "application/vnd.supernode.skill-catalog.v1+json";

pub(crate) type OciSkillCatalogError = OciArtifactError;

pub(super) fn is_trusted_catalog_reference(reference: &str) -> Result<bool, OciSkillCatalogError> {
    oci_client::is_trusted_reference(reference)
}

pub(super) async fn fetch_catalog_json(
    reference: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, OciSkillCatalogError> {
    oci_client::fetch_artifact_json(reference, max_bytes, SKILL_CATALOG_LAYER_MEDIA_TYPE).await
}
