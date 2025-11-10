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
{{- with .Values.extraLabels }}
{{- toYaml . }}
{{- end }}
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
{{- if .Values.serviceAccount.create }}
{{- default (include "dolos.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Resolve the ConfigMap name for Dolos configuration.
*/}}
{{- define "dolos.configurationConfigMapName" -}}
{{- if and .Values.config.create (not .Values.config.existingConfigMap) }}
{{- printf "%s-config" (include "dolos.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- .Values.config.existingConfigMap | default "" }}
{{- end }}
{{- end }}
