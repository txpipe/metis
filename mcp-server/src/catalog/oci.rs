use crate::oci_client::{self, OciArtifactError};

const CATALOG_LAYER_MEDIA_TYPE: &str = "application/vnd.supernode.extension-catalog.v1+json";

pub(crate) type OciCatalogError = OciArtifactError;

pub(super) fn is_trusted_catalog_reference(reference: &str) -> Result<bool, OciCatalogError> {
    oci_client::is_trusted_reference(reference)
}

pub(super) async fn fetch_catalog_json(
    reference: &str,
    max_bytes: usize,
) -> Result<Vec<u8>, OciCatalogError> {
    oci_client::fetch_artifact_json(reference, max_bytes, CATALOG_LAYER_MEDIA_TYPE).await
}
