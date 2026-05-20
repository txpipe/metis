{{- define "apex-fusion.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

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

{{- define "apex-fusion.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "apex-fusion.labels" -}}
helm.sh/chart: {{ include "apex-fusion.chart" . }}
app.kubernetes.io/name: {{ include "apex-fusion.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "apex-fusion.selectorLabels" -}}
app.kubernetes.io/name: {{ include "apex-fusion.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/component: apex-fusion
{{- end }}

{{- define "apex-fusion.proxyConfigName" -}}
{{- printf "%s-proxy" (include "apex-fusion.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "apex-fusion.metricsConfigMapName" -}}
{{- printf "%s-metrics" (include "apex-fusion.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "apex-fusion.topologyConfigMapName" -}}
{{- if ne (default "image-default" .Values.node.topology.mode) "image-default" }}
{{- printf "%s-topology" (include "apex-fusion.fullname" .) | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}

{{- define "apex-fusion.networkMagic" -}}
{{- if ne .Values.node.networkMagic nil -}}
{{- printf "%v" .Values.node.networkMagic -}}
{{- else -}}
{{- $networkMagicByNetwork := dict "vector-testnet" 1 "prime-testnet" 3311 -}}
{{- with get $networkMagicByNetwork .Values.node.network }}{{ printf "%v" . }}{{ end -}}
{{- end -}}
{{- end }}

{{- define "apex-fusion.shelleyTransitionEpoch" -}}
{{- $shelleyTransitionEpochByNetwork := dict "vector-testnet" 0 "prime-testnet" 0 "prime-mainnet" 2 -}}
{{- with get $shelleyTransitionEpochByNetwork .Values.node.network }}{{ printf "%v" . }}{{ end -}}
{{- end }}

{{- define "apex-fusion.releaseFullnameFor" -}}
{{- $releaseName := required "releaseName is required" .releaseName -}}
{{- $chartName := required "chartName is required" .chartName -}}
{{- if contains $chartName $releaseName -}}
{{- $releaseName | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" $releaseName $chartName | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end }}
