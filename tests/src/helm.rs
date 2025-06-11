use anyhow::Result;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Secret, Service};
use k8s_openapi::api::networking::v1::Ingress;
use serde_yaml::Value;

/// Parse a YAML document into a specific Kubernetes resource type
pub fn parse_k8s_resource<T>(yaml_doc: &Value) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_yaml::from_value(yaml_doc.clone())?)
}

/// Find all resources of a specific kind in the documents
pub fn find_resources_by_kind<'a>(documents: &'a [Value], kind: &str) -> Vec<&'a Value> {
    documents
        .iter()
        .filter(|doc| {
            doc.get("kind")
                .and_then(|k| k.as_str())
                .map(|k| k == kind)
                .unwrap_or(false)
        })
        .collect()
}

/// Validate that a deployment has the expected environment variables from secrets
pub fn validate_deployment_secret_env_vars(
    deployment: &Deployment,
    expected_secret_refs: &[(&str, &str, &str)], // (env_name, secret_name, secret_key)
) -> Result<()> {
    let container = deployment
        .spec
        .as_ref()
        .and_then(|spec| spec.template.spec.as_ref())
        .and_then(|pod_spec| pod_spec.containers.first())
        .ok_or_else(|| anyhow::anyhow!("Deployment has no containers"))?;

    let env_vars = container
        .env
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Container has no environment variables"))?;

    for (env_name, secret_name, secret_key) in expected_secret_refs {
        let env_var = env_vars
            .iter()
            .find(|env| env.name == *env_name)
            .ok_or_else(|| anyhow::anyhow!("Environment variable '{}' not found", env_name))?;

        let secret_ref = env_var
            .value_from
            .as_ref()
            .and_then(|vf| vf.secret_key_ref.as_ref())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Environment variable '{}' does not reference a secret",
                    env_name
                )
            })?;

        if secret_ref.name.as_deref() != Some(*secret_name) {
            anyhow::bail!(
                "Environment variable '{}' references wrong secret: expected '{}', got '{:?}'",
                env_name,
                secret_name,
                secret_ref.name
            );
        }

        if secret_ref.key != *secret_key {
            anyhow::bail!(
                "Environment variable '{}' references wrong key: expected '{}', got '{}'",
                env_name,
                secret_key,
                secret_ref.key
            );
        }
    }

    Ok(())
}

/// Validate that a secret contains expected keys
pub fn validate_secret_keys(secret: &Secret, expected_keys: &[&str]) -> Result<()> {
    let data = secret
        .data
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Secret has no data"))?;

    for key in expected_keys {
        if !data.contains_key(*key) {
            anyhow::bail!("Secret missing expected key: {}", key);
        }
    }

    Ok(())
}

/// Validate that a service has the expected port configuration
pub fn validate_service_port(service: &Service, expected_port: i32, expected_target_port: &str) -> Result<()> {
    let ports = service
        .spec
        .as_ref()
        .and_then(|spec| spec.ports.as_ref())
        .ok_or_else(|| anyhow::anyhow!("Service has no ports"))?;

    let port = ports
        .first()
        .ok_or_else(|| anyhow::anyhow!("Service has no ports defined"))?;

    if port.port != expected_port {
        anyhow::bail!(
            "Service port mismatch: expected {}, got {}",
            expected_port,
            port.port
        );
    }

    let target_port = port
        .target_port
        .as_ref()
        .and_then(|tp| match tp {
            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => Some(s.as_str()),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("Service has no target port"))?;

    if target_port != expected_target_port {
        anyhow::bail!(
            "Service target port mismatch: expected '{}', got '{}'",
            expected_target_port,
            target_port
        );
    }

    Ok(())
}

/// Validate that an ingress has the expected hosts
pub fn validate_ingress_hosts(ingress: &Ingress, expected_hosts: &[&str]) -> Result<()> {
    let rules = ingress
        .spec
        .as_ref()
        .and_then(|spec| spec.rules.as_ref())
        .ok_or_else(|| anyhow::anyhow!("Ingress has no rules"))?;

    let actual_hosts: Vec<String> = rules
        .iter()
        .filter_map(|rule| rule.host.as_ref())
        .cloned()
        .collect();

    for expected_host in expected_hosts {
        if !actual_hosts.contains(&expected_host.to_string()) {
            anyhow::bail!("Ingress missing expected host: {}", expected_host);
        }
    }

    Ok(())
}