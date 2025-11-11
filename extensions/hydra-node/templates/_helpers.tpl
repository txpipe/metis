{{/*
Expand the name of the chart.
*/}}
{{- define "hydra-node.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "hydra-node.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version label.
*/}}
{{- define "hydra-node.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels for resources.
*/}}
{{- define "hydra-node.labels" -}}
helm.sh/chart: {{ include "hydra-node.chart" . }}
app.kubernetes.io/name: {{ include "hydra-node.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- with .Values.extraLabels }}
{{- toYaml . }}
{{- end }}
{{- end }}

{{/*
Selector labels for the StatefulSet and Services.
*/}}
{{- define "hydra-node.selectorLabels" -}}
app.kubernetes.io/name: {{ include "hydra-node.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: hydra-node
{{- end }}

{{/*
Service account name.
*/}}
{{- define "hydra-node.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "hydra-node.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Resolve ConfigMap name for protocol parameters.
*/}}
{{- define "hydra-node.protocolParametersConfigMapName" -}}
{{- if and .Values.ledger.protocolParameters.create (not .Values.ledger.protocolParameters.name) }}
{{- printf "%s-protocol-parameters" (include "hydra-node.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- .Values.ledger.protocolParameters.name | default "" }}
{{- end }}
{{- end }}

{{/*
Resolve ConfigMap name for initial UTxO.
*/}}
{{- define "hydra-node.initialUtxoConfigMapName" -}}
{{- if and .Values.ledger.initialUtxo.create (not .Values.ledger.initialUtxo.name) }}
{{- printf "%s-initial-utxo" (include "hydra-node.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- .Values.ledger.initialUtxo.name | default "" }}
{{- end }}
{{- end }}

{{/*
Resolve Secret name for Hydra signing key.
*/}}
{{- define "hydra-node.hydraSigningSecretName" -}}
{{- if .Values.keys.hydraSigning.existingSecret.name }}
{{- .Values.keys.hydraSigning.existingSecret.name }}
{{- else if .Values.keys.hydraSigning.value }}
{{- printf "%s-hydra-signing" (include "hydra-node.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Resolve Secret key for Hydra signing key.
*/}}
{{- define "hydra-node.hydraSigningSecretKey" -}}
{{- if .Values.keys.hydraSigning.existingSecret.key }}
{{- .Values.keys.hydraSigning.existingSecret.key }}
{{- else }}
{{- default "hydra.sk" .Values.keys.hydraSigning.filename }}
{{- end }}
{{- end }}

{{/*
Resolve ConfigMap name for Hydra verification keys.
*/}}
{{- define "hydra-node.hydraVerificationConfigMapName" -}}
{{- if .Values.keys.hydraVerification.existingConfigMap.name }}
{{- .Values.keys.hydraVerification.existingConfigMap.name }}
{{- else if gt (len (default (list) .Values.keys.hydraVerification.items)) 0 }}
{{- printf "%s-hydra-verification" (include "hydra-node.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Resolve Secret name for Cardano signing key.
*/}}
{{- define "hydra-node.cardanoSigningSecretName" -}}
{{- if .Values.keys.cardano.signing.existingSecret.name }}
{{- .Values.keys.cardano.signing.existingSecret.name }}
{{- else if and .Values.keys.cardano.enabled .Values.keys.cardano.signing.value }}
{{- printf "%s-cardano-signing" (include "hydra-node.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Resolve Secret key for Cardano signing key.
*/}}
{{- define "hydra-node.cardanoSigningSecretKey" -}}
{{- if .Values.keys.cardano.signing.existingSecret.key }}
{{- .Values.keys.cardano.signing.existingSecret.key }}
{{- else }}
{{- default "cardano.sk" .Values.keys.cardano.signing.filename }}
{{- end }}
{{- end }}

{{/*
Resolve ConfigMap name for Cardano verification key.
*/}}
{{- define "hydra-node.cardanoVerificationConfigMapName" -}}
{{- if .Values.keys.cardano.verification.existingConfigMap.name }}
{{- .Values.keys.cardano.verification.existingConfigMap.name }}
{{- else if and .Values.keys.cardano.enabled .Values.keys.cardano.verification.value }}
{{- printf "%s-cardano-verification" (include "hydra-node.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}
