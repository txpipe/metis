{{- define "control-plane.vaultSealConfig" -}}
{{- if eq .Values.integration.seal.mode "awskms" }}
seal "awskms" {
  region = "{{ required "vault.integration.seal.awskms.region is required when vault.integration.seal.mode=awskms" .Values.integration.seal.awskms.region }}"
  kms_key_id = "{{ required "vault.integration.seal.awskms.kmsKeyId is required when vault.integration.seal.mode=awskms" .Values.integration.seal.awskms.kmsKeyId }}"
  {{- with .Values.integration.seal.awskms.endpoint }}
  endpoint = "{{ . }}"
  {{- end }}
}
{{- else if eq .Values.integration.seal.mode "gcpckms" }}
seal "gcpckms" {
  project = "{{ required "vault.integration.seal.gcpckms.project is required when vault.integration.seal.mode=gcpckms" .Values.integration.seal.gcpckms.project }}"
  region = "{{ required "vault.integration.seal.gcpckms.region is required when vault.integration.seal.mode=gcpckms" .Values.integration.seal.gcpckms.region }}"
  key_ring = "{{ required "vault.integration.seal.gcpckms.keyRing is required when vault.integration.seal.mode=gcpckms" .Values.integration.seal.gcpckms.keyRing }}"
  crypto_key = "{{ required "vault.integration.seal.gcpckms.cryptoKey is required when vault.integration.seal.mode=gcpckms" .Values.integration.seal.gcpckms.cryptoKey }}"
}
{{- else if eq .Values.integration.seal.mode "azurekeyvault" }}
seal "azurekeyvault" {
  tenant_id = "{{ required "vault.integration.seal.azurekeyvault.tenantId is required when vault.integration.seal.mode=azurekeyvault" .Values.integration.seal.azurekeyvault.tenantId }}"
  client_id = "{{ required "vault.integration.seal.azurekeyvault.clientId is required when vault.integration.seal.mode=azurekeyvault" .Values.integration.seal.azurekeyvault.clientId }}"
  vault_name = "{{ required "vault.integration.seal.azurekeyvault.vaultName is required when vault.integration.seal.mode=azurekeyvault" .Values.integration.seal.azurekeyvault.vaultName }}"
  key_name = "{{ required "vault.integration.seal.azurekeyvault.keyName is required when vault.integration.seal.mode=azurekeyvault" .Values.integration.seal.azurekeyvault.keyName }}"
  {{- with .Values.integration.seal.azurekeyvault.environment }}
  environment = "{{ . }}"
  {{- end }}
  {{- with .Values.integration.seal.azurekeyvault.resource }}
  resource = "{{ . }}"
  {{- end }}
}
{{- else if eq .Values.integration.seal.mode "ocikms" }}
seal "ocikms" {
  key_id = "{{ required "vault.integration.seal.ocikms.keyId is required when vault.integration.seal.mode=ocikms" .Values.integration.seal.ocikms.keyId }}"
  crypto_endpoint = "{{ required "vault.integration.seal.ocikms.cryptoEndpoint is required when vault.integration.seal.mode=ocikms" .Values.integration.seal.ocikms.cryptoEndpoint }}"
  management_endpoint = "{{ required "vault.integration.seal.ocikms.managementEndpoint is required when vault.integration.seal.mode=ocikms" .Values.integration.seal.ocikms.managementEndpoint }}"
  {{- if .Values.integration.seal.ocikms.authTypeApiKey }}
  auth_type_api_key = "true"
  {{- end }}
}
{{- else if eq .Values.integration.seal.mode "transit" }}
seal "transit" {
  address = "{{ required "vault.integration.seal.transit.address is required when vault.integration.seal.mode=transit" .Values.integration.seal.transit.address }}"
  key_name = "{{ required "vault.integration.seal.transit.keyName is required when vault.integration.seal.mode=transit" .Values.integration.seal.transit.keyName }}"
  mount_path = "{{ .Values.integration.seal.transit.mountPath }}"
  {{- with .Values.integration.seal.transit.namespace }}
  namespace = "{{ . }}"
  {{- end }}
  {{- if .Values.integration.seal.transit.disableRenewal }}
  disable_renewal = "true"
  {{- end }}
  {{- with .Values.integration.seal.transit.tlsCaCert }}
  tls_ca_cert = "{{ . }}"
  {{- end }}
  {{- with .Values.integration.seal.transit.tlsClientCert }}
  tls_client_cert = "{{ . }}"
  {{- end }}
  {{- with .Values.integration.seal.transit.tlsClientKey }}
  tls_client_key = "{{ . }}"
  {{- end }}
  {{- with .Values.integration.seal.transit.tlsServerName }}
  tls_server_name = "{{ . }}"
  {{- end }}
  {{- if .Values.integration.seal.transit.tlsSkipVerify }}
  tls_skip_verify = "true"
  {{- end }}
}
{{- end }}
{{- end }}

{{- define "control-plane.vaultConfigCommon" -}}
ui = true

listener "tcp" {
  tls_disable = 1
  address = "[::]:8200"
  cluster_address = "[::]:8201"

  telemetry {
    unauthenticated_metrics_access = "true"
  }
}

{{ include "control-plane.vaultSealConfig" . }}

telemetry {
  prometheus_retention_time = "30s"
  disable_hostname = true
}
{{- end }}
