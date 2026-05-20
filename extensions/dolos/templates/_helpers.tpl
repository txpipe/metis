{{/*
Expand the name of the chart.
*/}}
{{- define "dolos.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "dolos.fullname" -}}
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
{{- define "dolos.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels for resources.
*/}}
{{- define "dolos.labels" -}}
helm.sh/chart: {{ include "dolos.chart" . }}
app.kubernetes.io/name: {{ include "dolos.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels for the StatefulSet and Services.
*/}}
{{- define "dolos.selectorLabels" -}}
{{- include "dolos.selectorLabelsFor" (dict "context" . "component" "dolos") }}
{{- end }}

{{/*
Selector labels helper for specific component.
*/}}
{{- define "dolos.selectorLabelsFor" -}}
{{- $ctx := .context -}}
app.kubernetes.io/name: {{ include "dolos.name" $ctx }}
app.kubernetes.io/instance: {{ $ctx.Release.Name }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Service account name.
*/}}
{{- define "dolos.serviceAccountName" -}}
{{- include "dolos.fullname" . }}
{{- end }}

{{/*
Resolve the ConfigMap name for Dolos configuration.
*/}}
{{- define "dolos.configurationConfigMapName" -}}
{{- printf "%s-config" (include "dolos.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Resolve the ConfigMap name for Metis metrics scripts.
*/}}
{{- define "dolos.metricsConfigMapName" -}}
{{- printf "%s-metrics" (include "dolos.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Render the opinionated built-in Dolos TOML config for the selected network.
*/}}
{{- define "dolos.configPreset" -}}
{{- $network := .Values.dolos.network -}}
{{- $upstreamAddress := required "config.upstreamAddress must be set to a trusted Cardano relay address" .Values.config.upstreamAddress -}}
{{- $genesisNetwork := "preview" -}}
{{- $magic := "2" -}}
{{- $isTestnet := "true" -}}
{{- if eq $network "cardano-mainnet" -}}
{{- $genesisNetwork = "mainnet" -}}
{{- $magic = "764824073" -}}
{{- $isTestnet = "false" -}}
{{- else if eq $network "cardano-preprod" -}}
{{- $genesisNetwork = "preprod" -}}
{{- $magic = "1" -}}
{{- end -}}
[upstream]
peer_address = "{{ $upstreamAddress }}"

[storage]
version = "v3"
path = "/var/data/{{ $genesisNetwork }}/db"

[genesis]
byron_path = "/etc/genesis/{{ $genesisNetwork }}/byron.json"
shelley_path = "/etc/genesis/{{ $genesisNetwork }}/shelley.json"
alonzo_path = "/etc/genesis/{{ $genesisNetwork }}/alonzo.json"
conway_path = "/etc/genesis/{{ $genesisNetwork }}/conway.json"

[sync]
max_rollback = 25920

[serve.grpc]
listen_address = "[::]:50051"

[serve.minibf]
listen_address = "[::]:3001"

[serve.minikupo]
listen_address = "[::]:1442"

[serve.trp]
listen_address = "[::]:8164"

[chain]
type = "cardano"
magic = {{ $magic }}
is_testnet = {{ $isTestnet }}
{{- end }}
