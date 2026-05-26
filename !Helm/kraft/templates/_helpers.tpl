
{{- define "kraft.validate" -}}
{{- if not .Values.database.secret -}}
  {{- fail "database.secret must be set to the name of a Kubernetes secret containing database credentials" -}}
{{- end -}}
{{- if and .Values.ntfy.enabled (not .Values.ntfy.secret) -}}
    {{- fail "ntfy enabled but no secret provided" -}}
{{- end -}}

{{- end -}}
