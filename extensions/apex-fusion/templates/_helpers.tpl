{{/*
Expand the name of the chart.
*/}}
{{- define "vector-node.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "vector-node.fullname" -}}
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
{{- define "vector-node.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels for resources.
*/}}
{{- define "vector-node.labels" -}}
helm.sh/chart: {{ include "vector-node.chart" . }}
app.kubernetes.io/name: {{ include "vector-node.name" . }}
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
{{- define "vector-node.selectorLabels" -}}
{{- include "vector-node.selectorLabelsFor" (dict "context" . "component" "vector-node") }}
{{- end }}

{{/*
Selector labels helper for a specific component.
*/}}
{{- define "vector-node.selectorLabelsFor" -}}
{{- $ctx := .context -}}
app.kubernetes.io/name: {{ include "vector-node.name" $ctx }}
app.kubernetes.io/instance: {{ $ctx.Release.Name }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Service account name.
*/}}
{{- define "vector-node.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "vector-node.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Resolve the ConfigMap name that holds the proxy configuration.
*/}}
{{- define "vector-node.proxyConfigName" -}}
{{- if and .Values.proxy.enabled .Values.proxy.config }}
{{- if .Values.proxy.config.name }}
{{- .Values.proxy.config.name }}
{{- else }}
{{- printf "%s-proxy" (include "vector-node.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Resolve the ConfigMap name for custom node configuration files.
*/}}
{{- define "vector-node.configurationConfigMapName" -}}
{{- if and .Values.configuration.create (not .Values.configuration.name) }}
{{- printf "%s-configuration" (include "vector-node.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else if .Values.configuration.name }}
{{- .Values.configuration.name }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}
