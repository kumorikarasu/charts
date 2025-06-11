use anyhow::Result;
use helm_tests::*;
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Secret, Service, ConfigMap};
use k8s_openapi::api::networking::v1::Ingress;
use k8s_openapi::api::batch::v1::Job;
use std::collections::HashMap;

const CHART_PATH: &str = "../charts/life/";

#[test]
fn test_helm_lint() -> Result<()> {
    let output = run_helm_lint_with_values(CHART_PATH, &[
        "--set", "firebase_api_key=test-api-key",
        "--set", "firebase_auth_domain=test.firebaseapp.com",
        "--set", "firebase_project_id=test-project",
        "--set", "firebase_storage_bucket=test.appspot.com",
        "--set", "firebase_messaging_sender_id=123456789",
        "--set", "firebase_app_id=test-app-id",
        "--set", "firebase_vapid_key=test-vapid-key",
        "--set", "api_endpoint=https://api.test.com"
    ])?;
    assert!(output.contains("1 chart(s) linted, 0 chart(s) failed"));
    Ok(())
}

#[test]
fn test_basic_template_rendering() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    assert!(!output.trim().is_empty());
    
    // Parse the YAML documents
    let documents = parse_yaml_documents(&output)?;
    assert!(!documents.is_empty());
    
    Ok(())
}

#[test]
fn test_api_deployment_exists() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    let documents = parse_yaml_documents(&output)?;
    
    let deployments = find_resources_by_kind(&documents, "Deployment");
    let api_deployment = deployments
        .iter()
        .find(|d| {
            d.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("api"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("API deployment not found"))?;
    
    let deployment: Deployment = parse_k8s_resource(api_deployment)?;
    
    // Validate the deployment has the correct image
    let container = deployment
        .spec
        .as_ref()
        .unwrap()
        .template
        .spec
        .as_ref()
        .unwrap()
        .containers
        .first()
        .unwrap();
    
    assert!(container.image.as_ref().unwrap().contains("simbru-api"));
    
    Ok(())
}

#[test]
fn test_frontend_deployment_exists() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    let documents = parse_yaml_documents(&output)?;
    
    let deployments = find_resources_by_kind(&documents, "Deployment");
    let frontend_deployment = deployments
        .iter()
        .find(|d| {
            d.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("frontend"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("Frontend deployment not found"))?;
    
    let deployment: Deployment = parse_k8s_resource(frontend_deployment)?;
    
    // Validate the deployment has the correct image
    let container = deployment
        .spec
        .as_ref()
        .unwrap()
        .template
        .spec
        .as_ref()
        .unwrap()
        .containers
        .first()
        .unwrap();
    
    assert!(container.image.as_ref().unwrap().contains("simbru-pwa"));
    
    Ok(())
}

#[test]
fn test_api_deployment_environment_variables() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    let documents = parse_yaml_documents(&output)?;
    
    let deployments = find_resources_by_kind(&documents, "Deployment");
    let api_deployment = deployments
        .iter()
        .find(|d| {
            d.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("api"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("API deployment not found"))?;
    
    let deployment: Deployment = parse_k8s_resource(api_deployment)?;
    
    // Validate environment variables reference secrets correctly
    let expected_secret_refs = vec![
        ("POSTGRES_CONNECTION_STRING", "test-release-life-pg-credentials", "connection-string"),
        ("OAUTH_CLIENT_ID", "test-release-life-oauth-secrets", "client-id"),
        ("OAUTH_CLIENT_SECRET", "test-release-life-oauth-secrets", "client-secret"),
        ("OAUTH_REDIRECT_URL", "test-release-life-oauth-secrets", "redirect-url"),
        ("FIREBASE_API_KEY", "test-release-life-firebase-secrets", "api-key"),
        ("FIREBASE_AUTH_DOMAIN", "test-release-life-firebase-secrets", "auth-domain"),
        ("FIREBASE_PROJECT_ID", "test-release-life-firebase-secrets", "project-id"),
        ("FIREBASE_STORAGE_BUCKET", "test-release-life-firebase-secrets", "storage-bucket"),
        ("FIREBASE_MESSAGING_SENDER_ID", "test-release-life-firebase-secrets", "messaging-sender-id"),
        ("FIREBASE_APP_ID", "test-release-life-firebase-secrets", "app-id"),
        ("FIREBASE_VAPID_KEY", "test-release-life-firebase-secrets", "vapid-key"),
        ("API_ENDPOINT", "test-release-life-firebase-secrets", "api-endpoint"),
    ];
    
    validate_deployment_secret_env_vars(&deployment, &expected_secret_refs)?;
    
    Ok(())
}

#[test]
fn test_postgres_secret_creation() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("postgres_connection_string".to_string(), "postgres://test:pass@host/db".to_string());
    values.insert("postgres_app_password".to_string(), "testpass123".to_string());
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    let documents = parse_yaml_documents(&output)?;
    
    let secrets = find_resources_by_kind(&documents, "Secret");
    let postgres_secret = secrets
        .iter()
        .find(|s| {
            s.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("pg-credentials"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("Postgres credentials secret not found"))?;
    
    let secret: Secret = parse_k8s_resource(postgres_secret)?;
    
    // Validate the secret has the expected keys
    validate_secret_keys(&secret, &["connection-string", "app-password"])?;
    
    Ok(())
}

#[test]
fn test_services_exist() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    let documents = parse_yaml_documents(&output)?;
    
    let services = find_resources_by_kind(&documents, "Service");
    
    // Find API service
    let api_service = services
        .iter()
        .find(|s| {
            s.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("api"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("API service not found"))?;
    
    let api_svc: Service = parse_k8s_resource(api_service)?;
    validate_service_port(&api_svc, 8000, "http")?;
    
    // Find Frontend service
    let frontend_service = services
        .iter()
        .find(|s| {
            s.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("frontend"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("Frontend service not found"))?;
    
    let frontend_svc: Service = parse_k8s_resource(frontend_service)?;
    validate_service_port(&frontend_svc, 8080, "http")?;
    
    Ok(())
}

#[test]
fn test_ingresses_exist() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    let documents = parse_yaml_documents(&output)?;
    
    let ingresses = find_resources_by_kind(&documents, "Ingress");
    
    // Find API ingress
    let api_ingress = ingresses
        .iter()
        .find(|i| {
            i.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("api"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("API ingress not found"))?;
    
    let api_ing: Ingress = parse_k8s_resource(api_ingress)?;
    validate_ingress_hosts(&api_ing, &["simbru-api.home.ryougi.ca", "simbru-api.ryougi.ca"])?;
    
    // Find Frontend ingress
    let frontend_ingress = ingresses
        .iter()
        .find(|i| {
            i.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("frontend"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("Frontend ingress not found"))?;
    
    let frontend_ing: Ingress = parse_k8s_resource(frontend_ingress)?;
    validate_ingress_hosts(&frontend_ing, &["simbru.home.ryougi.ca", "simbru.ryougi.ca"])?;
    
    Ok(())
}

#[test]
fn test_db_init_job_exists() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    let documents = parse_yaml_documents(&output)?;
    
    let jobs = find_resources_by_kind(&documents, "Job");
    let db_init_job = jobs
        .iter()
        .find(|j| {
            j.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("db-init"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("DB init job not found"))?;
    
    let job: Job = parse_k8s_resource(db_init_job)?;
    
    // Validate job has pre-install hook annotation
    let annotations = job
        .metadata
        .annotations
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Job has no annotations"))?;
    
    assert!(annotations.contains_key("helm.sh/hook"));
    assert!(annotations["helm.sh/hook"].contains("pre-install"));
    
    Ok(())
}

#[test]
fn test_db_init_configmap_exists() -> Result<()> {
    let mut values = HashMap::new();
    values.insert("firebase_api_key".to_string(), "test-api-key".to_string());
    values.insert("firebase_auth_domain".to_string(), "test.firebaseapp.com".to_string());
    values.insert("firebase_project_id".to_string(), "test-project".to_string());
    values.insert("firebase_storage_bucket".to_string(), "test.appspot.com".to_string());
    values.insert("firebase_messaging_sender_id".to_string(), "123456789".to_string());
    values.insert("firebase_app_id".to_string(), "test-app-id".to_string());
    values.insert("firebase_vapid_key".to_string(), "test-vapid-key".to_string());
    values.insert("api_endpoint".to_string(), "https://api.test.com".to_string());
    
    let output = run_helm_template(CHART_PATH, Some(&values))?;
    let documents = parse_yaml_documents(&output)?;
    
    let configmaps = find_resources_by_kind(&documents, "ConfigMap");
    let db_init_cm = configmaps
        .iter()
        .find(|cm| {
            cm.get("metadata")
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                .map(|name| name.contains("db-init"))
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow::anyhow!("DB init ConfigMap not found"))?;
    
    let configmap: ConfigMap = parse_k8s_resource(db_init_cm)?;
    
    // Validate ConfigMap has init.sql
    let data = configmap
        .data
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("ConfigMap has no data"))?;
    
    assert!(data.contains_key("init.sql"));
    assert!(data["init.sql"].contains("CREATE DATABASE life"));
    assert!(data["init.sql"].contains("CREATE USER lifeapp"));
    
    Ok(())
}
