{{/* Expand the name of the chart. */}}
{{- define "cardano-db-sync.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/* Create a default fully qualified app name. */}}
{{- define "cardano-db-sync.fullname" -}}
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

{{/* Chart label. */}}
{{- define "cardano-db-sync.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/* Common labels. */}}
{{- define "cardano-db-sync.labels" -}}
helm.sh/chart: {{ include "cardano-db-sync.chart" . }}
app.kubernetes.io/name: {{ include "cardano-db-sync.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/* Selector labels for a component. */}}
{{- define "cardano-db-sync.selectorLabelsFor" -}}
{{- $ctx := .context -}}
app.kubernetes.io/name: {{ include "cardano-db-sync.name" $ctx }}
app.kubernetes.io/instance: {{ $ctx.Release.Name }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/* Service account name. */}}
{{- define "cardano-db-sync.serviceAccountName" -}}
{{- include "cardano-db-sync.fullname" . }}
{{- end }}

{{/* PostgreSQL Service name. */}}
{{- define "cardano-db-sync.postgresServiceName" -}}
{{- printf "%s-postgres" (include "cardano-db-sync.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/* DB Sync headless Service name. */}}
{{- define "cardano-db-sync.dbSyncServiceName" -}}
{{- include "cardano-db-sync.fullname" . }}
{{- end }}

{{/* Credentials Secret name. */}}
{{- define "cardano-db-sync.secretName" -}}
{{- printf "%s-postgres" (include "cardano-db-sync.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/* DB Sync data PVC name. */}}
{{- define "cardano-db-sync.dataPVCName" -}}
{{- printf "%s-data" (include "cardano-db-sync.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/* Metrics ConfigMap name. */}}
{{- define "cardano-db-sync.metricsConfigMapName" -}}
{{- printf "%s-metrics" (include "cardano-db-sync.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}
