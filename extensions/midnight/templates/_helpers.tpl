{{/*
Expand the name of the chart.
*/}}
{{- define "midnight.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "midnight.fullname" -}}
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
{{- define "midnight.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels for resources.
*/}}
{{- define "midnight.labels" -}}
helm.sh/chart: {{ include "midnight.chart" . }}
app.kubernetes.io/name: {{ include "midnight.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{ with .Values.extraLabels }}
{{- toYaml . }}
{{- end }}
{{- end }}

{{/*
Selector labels for the StatefulSet and Service.
*/}}
{{- define "midnight.selectorLabels" -}}
{{- include "midnight.selectorLabelsFor" (dict "context" . "component" "midnight") }}
{{- end }}

{{/*
Selector labels helper for a specific component.
*/}}
{{- define "midnight.selectorLabelsFor" -}}
{{- $ctx := .context -}}
app.kubernetes.io/name: {{ include "midnight.name" $ctx }}
app.kubernetes.io/instance: {{ $ctx.Release.Name }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Service account name.
*/}}
{{- define "midnight.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "midnight.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Join append args into a single string.
*/}}
{{- define "midnight.appendArgs" -}}
{{- if .Values.node.appendArgs }}
{{- join " " .Values.node.appendArgs }}
{{- else -}}
""
{{- end }}
{{- end }}

{{/*
Join bootnodes into a single string.
*/}}
{{- define "midnight.bootnodes" -}}
{{- if .Values.node.bootnodes }}
{{- join " " .Values.node.bootnodes }}
{{- else -}}
""
{{- end }}
{{- end }}

{{/*
Resolve the ConfigMap name for the chain configuration.
*/}}
{{- define "midnight.chainConfigName" -}}
{{- if and .Values.chainConfig.create (not .Values.chainConfig.name) }}
{{- printf "%s-chain-config" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else if .Values.chainConfig.name }}
{{- .Values.chainConfig.name }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Resolve the Secret name for the DB sync connection string.
*/}}
{{- define "midnight.dbSyncSecretName" -}}
{{- if .Values.dbSync.existingSecret.name }}
{{- .Values.dbSync.existingSecret.name }}
{{- else -}}
{{- $connectionString := include "midnight.dbSyncConnectionStringValue" . }}
{{- if $connectionString }}
{{- printf "%s-dbsync" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Resolve the Secret key for the DB sync connection string.
*/}}
{{- define "midnight.dbSyncSecretKey" -}}
{{- if .Values.dbSync.existingSecret.key }}
{{- .Values.dbSync.existingSecret.key }}
{{- else -}}
connection
{{- end }}
{{- end }}

{{/*
Resolve the Secret name for the node key.
*/}}
{{- define "midnight.nodeKeySecretName" -}}
{{- if .Values.nodeKey.existingSecret.name }}
{{- .Values.nodeKey.existingSecret.name }}
{{- else if .Values.nodeKey.value }}
{{- printf "%s-node-key" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else -}}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Resolve the name for the managed DB sync Deployment.
*/}}
{{- define "midnight.dbSyncName" -}}
{{- printf "%s-dbsync" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Resolve the PVC name that stores managed DB sync data.
*/}}
{{- define "midnight.dbSyncDataPVCName" -}}
{{- printf "%s-data" (include "midnight.dbSyncName" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Resolve the Secret key for the node key.
*/}}
{{- define "midnight.nodeKeySecretKey" -}}
{{- if .Values.nodeKey.existingSecret.key }}
{{- .Values.nodeKey.existingSecret.key }}
{{- else -}}
node.key
{{- end }}
{{- end }}

{{/*
Compute the DB sync connection string that should be surfaced to the Midnight node.
*/}}
{{- define "midnight.dbSyncConnectionStringValue" -}}
{{- if .Values.dbSync.connectionString }}
{{- .Values.dbSync.connectionString }}
{{- else if and .Values.dbSync.managed.enabled (not .Values.dbSync.managed.connection.existingSecret.name) .Values.dbSync.managed.connection.username .Values.dbSync.managed.connection.password .Values.dbSync.managed.connection.database }}
{{- $service := include "midnight.dbSyncPostgresServiceName" . -}}
{{- $port := (default 5432 .Values.dbSync.managed.postgres.port | int) -}}
{{- printf "postgresql://%s:%s@%s:%d/%s" .Values.dbSync.managed.connection.username .Values.dbSync.managed.connection.password $service $port .Values.dbSync.managed.connection.database }}
{{- else -}}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Compute the Service name for the managed DB sync Postgres instance.
*/}}
{{- define "midnight.dbSyncPostgresServiceName" -}}
{{- printf "%s-dbsync-postgres" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Resolve the Secret name that holds the managed DB sync Postgres credentials.
*/}}
{{- define "midnight.dbSyncPostgresSecretName" -}}
{{- if and .Values.dbSync.managed.enabled .Values.dbSync.managed.connection.existingSecret.name }}
{{- .Values.dbSync.managed.connection.existingSecret.name }}
{{- else if .Values.dbSync.managed.enabled }}
{{- printf "%s-dbsync-postgres" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else -}}
{{- "" }}
{{- end }}
{{- end }}
