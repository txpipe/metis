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
{{- include "midnight.fullname" . }}
{{- end }}

{{/*
Build the default Midnight node argument string.
*/}}
{{- define "midnight.appendArgs" -}}
{{- $args := list -}}
{{- $args = append $args "--allow-private-ip" }}
{{- $args = append $args "--prometheus-external" }}
{{- $args = append $args "--rpc-external" }}
{{- $args = append $args "--ws-external" }}
{{- $args = append $args "--rpc-methods=Safe" }}
{{- $args = append $args "--rpc-cors=all" }}
{{- $args = append $args "--pool-limit=10" }}
{{- $args = append $args "--trie-cache-size=0" }}
{{- with .Values.node.pruning }}{{- $args = append $args (printf "--pruning=%s" .) }}{{- end }}
{{- range .Values.node.extraArgs }}{{- $args = append $args . }}{{- end }}
{{- join " " $args }}
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
Resolve the Secret name for the DB sync connection string.
*/}}
{{- define "midnight.dbSyncSecretName" -}}
{{- printf "%s-dbsync" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Resolve the Secret key for the DB sync connection string.
*/}}
{{- define "midnight.dbSyncSecretKey" -}}
connection
{{- end }}

{{/*
Resolve the Secret name for the node key.
*/}}
{{- define "midnight.nodeKeySecretName" -}}
{{- printf "%s-node-key" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Resolve the Secret key for the node key.
*/}}
{{- define "midnight.nodeKeySecretKey" -}}
node.key
{{- end }}

{{/*
Resolve the ConfigMap name for Metis metrics scripts.
*/}}
{{- define "midnight.metricsConfigMapName" -}}
{{- printf "%s-metrics" (include "midnight.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}
