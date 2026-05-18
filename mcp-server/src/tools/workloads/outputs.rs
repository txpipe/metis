use k8s_openapi::api::core::v1::Service;
use serde::Serialize;

use crate::catalog::ExtensionCatalog;
use crate::catalog::ExtensionOutputDefinition;
use crate::k8s::HelmReleaseSummary;

use super::registry;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkloadOutput {
    pub name: String,
    pub description: String,
    pub scope: String,
    pub url: String,
    pub namespace: String,
    pub service_name: String,
    pub service_type: Option<String>,
    pub port_name: String,
    pub port: i32,
    pub protocol: String,
}

pub(crate) fn outputs_for_release(
    namespace: &str,
    release_name: &str,
    release: Option<&HelmReleaseSummary>,
    services: &[Service],
    catalog: &ExtensionCatalog,
) -> Vec<WorkloadOutput> {
    let Some(release) = release else {
        return vec![];
    };
    let Some(extension) = registry::extension_for_release(release, catalog) else {
        return vec![];
    };

    services
        .iter()
        .filter(|service| is_release_service(namespace, release_name, service))
        .flat_map(|service| {
            extension.outputs.iter().flat_map(move |output| {
                let internal = output_entries(
                    namespace,
                    service,
                    output,
                    "internal",
                    internal_hosts(namespace, service),
                );
                let external = output_entries(
                    namespace,
                    service,
                    output,
                    "external",
                    external_hosts(service),
                );
                internal.into_iter().chain(external).collect::<Vec<_>>()
            })
        })
        .collect()
}

fn is_release_service(namespace: &str, release_name: &str, service: &Service) -> bool {
    service.metadata.namespace.as_deref() == Some(namespace)
        && service
            .metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get("app.kubernetes.io/instance"))
            .is_some_and(|instance| instance == release_name)
        && service
            .spec
            .as_ref()
            .and_then(|spec| spec.cluster_ip.as_deref())
            != Some("None")
}

fn output_entries(
    namespace: &str,
    service: &Service,
    output: &ExtensionOutputDefinition,
    scope: &str,
    hosts: Vec<String>,
) -> Vec<WorkloadOutput> {
    let service_name = service.metadata.name.as_deref().unwrap_or_default();
    let service_type = service.spec.as_ref().and_then(|spec| spec.type_.clone());
    let Some(port) = service
        .spec
        .as_ref()
        .and_then(|spec| spec.ports.as_ref())
        .and_then(|ports| {
            ports
                .iter()
                .find(|port| port.name.as_deref() == Some(output.port_name.as_str()))
        })
    else {
        return vec![];
    };

    hosts
        .into_iter()
        .map(|host| WorkloadOutput {
            name: output.name.clone(),
            description: output.description.clone(),
            scope: scope.to_string(),
            url: format!("{}{}:{}", output_scheme(&output.protocol), host, port.port),
            namespace: namespace.to_string(),
            service_name: service_name.to_string(),
            service_type: service_type.clone(),
            port_name: output.port_name.clone(),
            port: port.port,
            protocol: output.protocol.clone(),
        })
        .collect()
}

fn internal_hosts(namespace: &str, service: &Service) -> Vec<String> {
    service
        .metadata
        .name
        .as_ref()
        .map(|name| vec![format!("{name}.{namespace}.svc.cluster.local")])
        .unwrap_or_default()
}

fn external_hosts(service: &Service) -> Vec<String> {
    if service.spec.as_ref().and_then(|spec| spec.type_.as_deref()) != Some("LoadBalancer") {
        return vec![];
    }

    service
        .status
        .as_ref()
        .and_then(|status| status.load_balancer.as_ref())
        .and_then(|load_balancer| load_balancer.ingress.as_ref())
        .map(|ingress| {
            ingress
                .iter()
                .filter_map(|entry| entry.ip.clone().or_else(|| entry.hostname.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn output_scheme(protocol: &str) -> &'static str {
    match protocol {
        "HTTP" => "http://",
        "WebSocket" => "ws://",
        "gRPC" => "grpc://",
        _ => "tcp://",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use k8s_openapi::api::core::v1::LoadBalancerIngress;
    use k8s_openapi::api::core::v1::LoadBalancerStatus;
    use k8s_openapi::api::core::v1::ServicePort;
    use k8s_openapi::api::core::v1::ServiceSpec;
    use k8s_openapi::api::core::v1::ServiceStatus;
    use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

    use crate::catalog::ExtensionCatalog;
    use crate::k8s::HelmChartSummary;
    use crate::k8s::HelmReleaseSummary;

    use super::*;

    #[test]
    fn relay_outputs_include_internal_n2n_and_n2c() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("cardano-node"));
        let service = service_with_ports(
            "relay-preview-cardano-node",
            "cardano",
            "relay-preview",
            "ClusterIP",
            vec![("n2n", 3000), ("n2c", 3307), ("metrics", 12798)],
            vec![],
        );

        let outputs = outputs_for_release(
            "cardano",
            "relay-preview",
            Some(&release),
            &[service],
            &catalog,
        );

        assert_eq!(outputs.len(), 2);
        assert!(outputs.iter().any(|output| {
            output.name == "n2n"
                && output.scope == "internal"
                && output.url == "tcp://relay-preview-cardano-node.cardano.svc.cluster.local:3000"
        }));
        assert!(outputs.iter().any(|output| {
            output.name == "n2c"
                && output.url == "tcp://relay-preview-cardano-node.cardano.svc.cluster.local:3307"
        }));
    }

    #[test]
    fn dolos_outputs_include_internal_endpoints() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("dolos"));
        let service = service_with_ports(
            "dolos-preview",
            "cardano-preview",
            "relay-preview",
            "ClusterIP",
            vec![
                ("grpc", 50051),
                ("minibf", 3001),
                ("minikupo", 1442),
                ("trp", 8164),
            ],
            vec![],
        );

        let outputs = outputs_for_release(
            "cardano-preview",
            "relay-preview",
            Some(&release),
            &[service],
            &catalog,
        );

        assert_eq!(outputs.len(), 4);
        assert!(outputs.iter().any(|output| output.name == "trp"));
        assert!(outputs.iter().any(|output| output.name == "blockfrost"));
        assert!(outputs.iter().any(|output| output.name == "kupo"));
        assert!(outputs.iter().any(|output| {
            output.name == "utxorpc"
                && output.url == "grpc://dolos-preview.cardano-preview.svc.cluster.local:50051"
        }));
    }

    #[test]
    fn hydra_outputs_include_api_websocket_p2p_and_monitoring() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("hydra-node"));
        let service = service_with_ports(
            "hydra-preview-hydra-node",
            "hydra",
            "hydra-preview",
            "ClusterIP",
            vec![("api", 4001), ("p2p", 5001), ("monitoring", 6001)],
            vec![],
        );

        let outputs = outputs_for_release(
            "hydra",
            "hydra-preview",
            Some(&release),
            &[service],
            &catalog,
        );

        assert_eq!(outputs.len(), 4);
        assert!(outputs.iter().any(|output| {
            output.name == "api"
                && output.url == "http://hydra-preview-hydra-node.hydra.svc.cluster.local:4001"
        }));
        assert!(outputs.iter().any(|output| {
            output.name == "ws"
                && output.url == "ws://hydra-preview-hydra-node.hydra.svc.cluster.local:4001"
        }));
        assert!(outputs.iter().any(|output| output.name == "p2p"));
        assert!(outputs.iter().any(|output| output.name == "monitoring"));
    }

    #[test]
    fn outputs_filter_by_namespace() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("dolos"));
        let wrong_namespace = service_with_ports(
            "dolos-preview",
            "other",
            "relay-preview",
            "ClusterIP",
            vec![
                ("grpc", 50051),
                ("minibf", 3001),
                ("minikupo", 1442),
                ("trp", 8164),
            ],
            vec![],
        );

        let outputs = outputs_for_release(
            "cardano-preview",
            "relay-preview",
            Some(&release),
            &[wrong_namespace],
            &catalog,
        );

        assert!(outputs.is_empty());
    }

    #[test]
    fn load_balancer_service_adds_external_outputs() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("dolos"));
        let service = service_with_ports(
            "dolos-preview",
            "cardano-preview",
            "relay-preview",
            "LoadBalancer",
            vec![
                ("grpc", 50051),
                ("minibf", 3001),
                ("minikupo", 1442),
                ("trp", 8164),
            ],
            vec![load_balancer_ingress(Some("203.0.113.10"), None)],
        );

        let outputs = outputs_for_release(
            "cardano-preview",
            "relay-preview",
            Some(&release),
            &[service],
            &catalog,
        );

        assert!(outputs.iter().any(|output| {
            output.name == "blockfrost"
                && output.scope == "external"
                && output.url == "http://203.0.113.10:3001"
        }));
    }

    #[test]
    fn load_balancer_hostname_is_used_for_external_outputs() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("cardano-node"));
        let service = service_with_ports(
            "relay-preview-cardano-node",
            "cardano",
            "relay-preview",
            "LoadBalancer",
            vec![("n2n", 3000), ("n2c", 3307)],
            vec![load_balancer_ingress(None, Some("relay.example.com"))],
        );

        let outputs = outputs_for_release(
            "cardano",
            "relay-preview",
            Some(&release),
            &[service],
            &catalog,
        );

        assert!(outputs.iter().any(|output| {
            output.name == "n2n"
                && output.scope == "external"
                && output.url == "tcp://relay.example.com:3000"
        }));
    }

    #[test]
    fn headless_services_are_not_reported_as_outputs() {
        let catalog = ExtensionCatalog::embedded();
        let release = helm_release(Some("dolos"));
        let mut service = service_with_ports(
            "dolos-preview-headless",
            "cardano-preview",
            "relay-preview",
            "ClusterIP",
            vec![
                ("grpc", 50051),
                ("minibf", 3001),
                ("minikupo", 1442),
                ("trp", 8164),
            ],
            vec![],
        );
        service.spec.as_mut().unwrap().cluster_ip = Some("None".to_string());

        let outputs = outputs_for_release(
            "cardano-preview",
            "relay-preview",
            Some(&release),
            &[service],
            &catalog,
        );

        assert!(outputs.is_empty());
    }

    fn helm_release(chart_name: Option<&str>) -> HelmReleaseSummary {
        HelmReleaseSummary {
            name: "relay-preview".to_string(),
            namespace: "cardano".to_string(),
            revision: 1,
            status: Some("deployed".to_string()),
            chart: HelmChartSummary {
                name: chart_name.map(str::to_string),
                version: Some("0.1.0".to_string()),
            },
            app_version: Some("11.0.1".to_string()),
            description: None,
            updated: None,
            secret_name: Some("sh.helm.release.v1.relay-preview.v1".to_string()),
            config: None,
        }
    }

    fn service_with_ports(
        name: &str,
        namespace: &str,
        release_name: &str,
        service_type: &str,
        ports: Vec<(&str, i32)>,
        ingress: Vec<LoadBalancerIngress>,
    ) -> Service {
        Service {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                labels: Some(BTreeMap::from([(
                    "app.kubernetes.io/instance".to_string(),
                    release_name.to_string(),
                )])),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some(service_type.to_string()),
                cluster_ip: Some("10.0.0.1".to_string()),
                ports: Some(
                    ports
                        .into_iter()
                        .map(|(name, port)| ServicePort {
                            name: Some(name.to_string()),
                            port,
                            protocol: Some("TCP".to_string()),
                            ..Default::default()
                        })
                        .collect(),
                ),
                ..Default::default()
            }),
            status: Some(ServiceStatus {
                load_balancer: (!ingress.is_empty()).then_some(LoadBalancerStatus {
                    ingress: Some(ingress),
                }),
                ..Default::default()
            }),
        }
    }

    fn load_balancer_ingress(ip: Option<&str>, hostname: Option<&str>) -> LoadBalancerIngress {
        LoadBalancerIngress {
            ip: ip.map(str::to_string),
            hostname: hostname.map(str::to_string),
            ..Default::default()
        }
    }
}
