use anyhow::Result;
use base64::Engine;
use std::collections::HashMap;
use std::process::Command;

pub mod helm;

pub use helm::*;

/// Helper function to run helm template command
pub fn run_helm_template(chart_path: &str, values: Option<&HashMap<String, String>>) -> Result<String> {
    let mut cmd = Command::new("helm");
    cmd.args(["template", "test-release", chart_path]);
    
    if let Some(vals) = values {
        for (key, value) in vals {
            cmd.args(["--set", &format!("{}={}", key, value)]);
        }
    }
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Helm template failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    Ok(String::from_utf8(output.stdout)?)
}

/// Helper function to run helm lint
pub fn run_helm_lint(chart_path: &str) -> Result<String> {
    let output = Command::new("helm")
        .args(["lint", chart_path])
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Helm lint failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    Ok(String::from_utf8(output.stdout)?)
}

/// Helper function to run helm lint with values
pub fn run_helm_lint_with_values(chart_path: &str, extra_args: &[&str]) -> Result<String> {
    let mut cmd = Command::new("helm");
    cmd.args(["lint", chart_path]);
    cmd.args(extra_args);
    
    let output = cmd.output()?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Helm lint failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    
    Ok(String::from_utf8(output.stdout)?)
}

/// Parse YAML documents from helm template output
pub fn parse_yaml_documents(yaml_content: &str) -> Result<Vec<serde_yaml::Value>> {
    let documents: Vec<serde_yaml::Value> = yaml_content
        .split("---")
        .filter(|doc| !doc.trim().is_empty())
        .map(|doc| serde_yaml::from_str(doc.trim()))
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(documents)
}

/// Find a resource by kind and name in parsed documents
pub fn find_resource_by_kind_and_name<'a>(
    documents: &'a [serde_yaml::Value],
    kind: &str,
    name: &str,
) -> Option<&'a serde_yaml::Value> {
    documents.iter().find(|doc| {
        if let Some(resource_kind) = doc.get("kind").and_then(|k| k.as_str()) {
            if let Some(metadata) = doc.get("metadata") {
                if let Some(resource_name) = metadata.get("name").and_then(|n| n.as_str()) {
                    return resource_kind == kind && resource_name.contains(name);
                }
            }
        }
        false
    })
}

/// Extract base64 decoded value from secret data
pub fn extract_secret_value(secret: &serde_yaml::Value, key: &str) -> Result<String> {
    let data = secret
        .get("data")
        .ok_or_else(|| anyhow::anyhow!("Secret has no data field"))?;
    
    let encoded_value = data
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Secret key '{}' not found", key))?;
    
    let decoded = base64::engine::general_purpose::STANDARD.decode(encoded_value)?;
    Ok(String::from_utf8(decoded)?)
}