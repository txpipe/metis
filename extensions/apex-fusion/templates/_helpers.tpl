{{/*
Expand the name of the chart.
*/}}
{{- define "apex-fusion.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "apex-fusion.fullname" -}}
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
{{- define "apex-fusion.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels for resources.
*/}}
{{- define "apex-fusion.labels" -}}
helm.sh/chart: {{ include "apex-fusion.chart" . }}
app.kubernetes.io/name: {{ include "apex-fusion.name" . }}
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
{{- define "apex-fusion.selectorLabels" -}}
{{- include "apex-fusion.selectorLabelsFor" (dict "context" . "component" "apex-fusion") }}
{{- end }}

{{/*
Selector labels helper for a specific component.
*/}}
{{- define "apex-fusion.selectorLabelsFor" -}}
{{- $ctx := .context -}}
app.kubernetes.io/name: {{ include "apex-fusion.name" $ctx }}
app.kubernetes.io/instance: {{ $ctx.Release.Name }}
app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Service account name.
*/}}
{{- define "apex-fusion.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "apex-fusion.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Resolve the ConfigMap name that holds the proxy configuration.
*/}}
{{- define "apex-fusion.proxyConfigName" -}}
{{- if and .Values.proxy.enabled .Values.proxy.config }}
{{- if .Values.proxy.config.name }}
{{- .Values.proxy.config.name }}
{{- else }}
{{- printf "%s-proxy" (include "apex-fusion.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Resolve the ConfigMap name for custom node configuration files.
*/}}
{{- define "apex-fusion.configurationConfigMapName" -}}
{{- if and .Values.configuration.create (not .Values.configuration.name) }}
{{- printf "%s-configuration" (include "apex-fusion.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else if .Values.configuration.name }}
{{- .Values.configuration.name }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}
