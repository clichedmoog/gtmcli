//! Tests using a mock HTTP server to test CLI commands without real API calls.
//! Uses wiremock to simulate GTM API responses based on real API response shapes.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn setup_server() -> MockServer {
    MockServer::start().await
}

fn gtm_with_server(server: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("gtm").expect("binary exists");
    cmd.env("GTM_API_BASE", server.uri());
    cmd.env("GTM_QUIET", "1"); // suppress update check
    cmd.args(["--format", "json"]);
    cmd
}

fn gtm_table_with_server(server: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("gtm").expect("binary exists");
    cmd.env("GTM_API_BASE", server.uri());
    cmd.env("GTM_QUIET", "1");
    cmd.args(["--format", "table"]);
    cmd
}

fn parse_json(output: &assert_cmd::assert::Assert) -> Value {
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    serde_json::from_str(&stdout).expect("should be valid JSON")
}

// ─── Accounts ───

#[tokio::test]
async fn test_mock_accounts_list() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "account": [
                {
                    "accountId": "123456",
                    "name": "Test Account",
                    "path": "accounts/123456",
                    "features": {
                        "supportMultipleContainers": true,
                        "supportUserPermissions": true
                    }
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args(["accounts", "list"])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert!(json["account"].is_array());
    assert_eq!(json["account"][0]["accountId"], "123456");
    assert_eq!(json["account"][0]["name"], "Test Account");
}

#[tokio::test]
async fn test_mock_accounts_list_table() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "account": [
                {
                    "accountId": "123456",
                    "name": "Test Account",
                    "path": "accounts/123456"
                }
            ]
        })))
        .mount(&server)
        .await;

    gtm_table_with_server(&server)
        .args(["accounts", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("123456"))
        .stdout(predicate::str::contains("Test Account"));
}

#[tokio::test]
async fn test_mock_accounts_get() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "accountId": "123456",
            "name": "Test Account",
            "path": "accounts/123456",
            "fingerprint": "1234567890"
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args(["accounts", "get", "--account-id", "123456"])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert_eq!(json["accountId"], "123456");
}

// ─── Containers ───

#[tokio::test]
async fn test_mock_containers_list() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "container": [
                {
                    "accountId": "123456",
                    "containerId": "789",
                    "name": "My Container",
                    "publicId": "GTM-XXXX",
                    "usageContext": ["web"],
                    "path": "accounts/123456/containers/789"
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args(["containers", "list", "--account-id", "123456"])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert!(json["container"].is_array());
    assert_eq!(json["container"][0]["publicId"], "GTM-XXXX");
}

#[tokio::test]
async fn test_mock_containers_create() {
    let server = setup_server().await;
    Mock::given(method("POST"))
        .and(path("/accounts/123456/containers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "accountId": "123456",
            "containerId": "999",
            "name": "New Container",
            "usageContext": ["web"],
            "path": "accounts/123456/containers/999"
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "containers",
            "create",
            "--account-id",
            "123456",
            "--name",
            "New Container",
            "--usage-context",
            "web",
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert_eq!(json["name"], "New Container");
    assert_eq!(json["containerId"], "999");
}

// ─── Workspaces ───

#[tokio::test]
async fn test_mock_workspaces_list() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "workspace": [
                {
                    "accountId": "123456",
                    "containerId": "789",
                    "workspaceId": "1",
                    "name": "Default Workspace",
                    "path": "accounts/123456/containers/789/workspaces/1"
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "workspaces",
            "list",
            "--account-id",
            "123456",
            "--container-id",
            "789",
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert!(json["workspace"].is_array());
    assert_eq!(json["workspace"][0]["name"], "Default Workspace");
}

// ─── Tags ───

#[tokio::test]
async fn test_mock_tags_list() {
    let server = setup_server().await;
    // Workspace resolution
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "workspace": [{"workspaceId": "1"}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/workspaces/1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tag": [
                {
                    "tagId": "42",
                    "name": "GA4 Config",
                    "type": "gaawc",
                    "path": "accounts/123456/containers/789/workspaces/1/tags/42"
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "tags",
            "list",
            "--account-id",
            "123456",
            "--container-id",
            "789",
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert!(json["tag"].is_array());
    assert_eq!(json["tag"][0]["name"], "GA4 Config");
}

#[tokio::test]
async fn test_mock_tags_create() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "workspace": [{"workspaceId": "1"}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/accounts/123456/containers/789/workspaces/1/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "tagId": "99",
            "name": "Test Tag",
            "type": "html",
            "firingTriggerId": ["1"]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "tags",
            "create",
            "--account-id",
            "123456",
            "--container-id",
            "789",
            "--name",
            "Test Tag",
            "--type",
            "html",
            "--firing-trigger-id",
            "1",
            "--params",
            r#"{"html":"<script>test</script>"}"#,
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert_eq!(json["tagId"], "99");
    assert_eq!(json["name"], "Test Tag");
}

#[tokio::test]
async fn test_mock_tags_delete_requires_force() {
    let server = setup_server().await;
    // Without --force, should print warning and exit successfully (no API call)
    gtm_with_server(&server)
        .args([
            "tags",
            "delete",
            "--account-id",
            "123456",
            "--container-id",
            "789",
            "--tag-id",
            "42",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("--force"));
}

#[tokio::test]
async fn test_mock_tags_delete_with_force() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "workspace": [{"workspaceId": "1"}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/accounts/123456/containers/789/workspaces/1/tags/42"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    gtm_with_server(&server)
        .args([
            "tags",
            "delete",
            "--account-id",
            "123456",
            "--container-id",
            "789",
            "--tag-id",
            "42",
            "--force",
        ])
        .assert()
        .success();
}

// ─── Triggers ───

#[tokio::test]
async fn test_mock_triggers_list() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "workspace": [{"workspaceId": "1"}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/accounts/123456/containers/789/workspaces/1/triggers",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "trigger": [
                {
                    "triggerId": "5",
                    "name": "All Pages",
                    "type": "pageview"
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "triggers",
            "list",
            "--account-id",
            "123456",
            "--container-id",
            "789",
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert_eq!(json["trigger"][0]["name"], "All Pages");
}

// ─── Variables ───

#[tokio::test]
async fn test_mock_variables_list() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/workspaces"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "workspace": [{"workspaceId": "1"}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(
            "/accounts/123456/containers/789/workspaces/1/variables",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "variable": [
                {
                    "variableId": "10",
                    "name": "Page URL",
                    "type": "u"
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "variables",
            "list",
            "--account-id",
            "123456",
            "--container-id",
            "789",
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert_eq!(json["variable"][0]["name"], "Page URL");
}

// ─── Versions ───

#[tokio::test]
async fn test_mock_versions_list() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/versions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "containerVersion": [
                {
                    "containerVersionId": "1",
                    "name": "v1.0",
                    "path": "accounts/123456/containers/789/versions/1"
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "versions",
            "list",
            "--account-id",
            "123456",
            "--container-id",
            "789",
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert!(json["containerVersion"].is_array());
}

#[tokio::test]
async fn test_mock_versions_live() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/versions:live"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "containerVersionId": "3",
            "name": "Production",
            "path": "accounts/123456/containers/789/versions/3"
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "versions",
            "live",
            "--account-id",
            "123456",
            "--container-id",
            "789",
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert_eq!(json["containerVersionId"], "3");
}

// ─── Environments ───

#[tokio::test]
async fn test_mock_environments_list() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/containers/789/environments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "environment": [
                {
                    "environmentId": "1",
                    "name": "Live",
                    "type": "live",
                    "path": "accounts/123456/containers/789/environments/1"
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args([
            "environments",
            "list",
            "--account-id",
            "123456",
            "--container-id",
            "789",
        ])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert!(json["environment"].is_array());
}

// ─── Permissions ───

#[tokio::test]
async fn test_mock_permissions_list() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/123456/user_permissions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "userPermission": [
                {
                    "accountId": "123456",
                    "emailAddress": "user@example.com",
                    "accountAccess": {"permission": "admin"}
                }
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args(["permissions", "list", "--account-id", "123456"])
        .assert()
        .success();
    let json = parse_json(&assert);
    assert!(json["userPermission"].is_array());
}

// ─── Pagination ───

#[tokio::test]
async fn test_mock_pagination() {
    let server = setup_server().await;

    // Page 1
    Mock::given(method("GET"))
        .and(path("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "account": [
                {"accountId": "1", "name": "Account 1"}
            ],
            "nextPageToken": "page2"
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Page 2
    Mock::given(method("GET"))
        .and(path("/accounts"))
        .and(query_param("pageToken", "page2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "account": [
                {"accountId": "2", "name": "Account 2"}
            ]
        })))
        .mount(&server)
        .await;

    let assert = gtm_with_server(&server)
        .args(["accounts", "list"])
        .assert()
        .success();
    let json = parse_json(&assert);
    let accounts = json["account"].as_array().unwrap();
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0]["accountId"], "1");
    assert_eq!(accounts[1]["accountId"], "2");
    // nextPageToken should be removed from final result
    assert!(json.get("nextPageToken").is_none());
}

// ─── Dry Run ───

#[tokio::test]
async fn test_mock_dry_run() {
    let server = setup_server().await;
    // No mock needed — dry-run should NOT call the API

    gtm_with_server(&server)
        .args([
            "--dry-run",
            "tags",
            "create",
            "--account-id",
            "123456",
            "--container-id",
            "789",
            "--workspace-id",
            "1",
            "--name",
            "Dry Run Tag",
            "--type",
            "html",
            "--firing-trigger-id",
            "1",
            "--params",
            r#"{"html":"test"}"#,
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("[dry-run]"));
}

// ─── Error Handling ───

#[tokio::test]
async fn test_mock_api_error_404() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts/999999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": {
                "code": 404,
                "message": "Account not found"
            }
        })))
        .mount(&server)
        .await;

    gtm_with_server(&server)
        .args(["accounts", "get", "--account-id", "999999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Account not found"));
}

#[tokio::test]
async fn test_mock_api_error_403() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "error": {
                "code": 403,
                "message": "The caller does not have permission"
            }
        })))
        .mount(&server)
        .await;

    gtm_with_server(&server)
        .args(["accounts", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("permission"));
}

// ─── Compact Format ───

#[tokio::test]
async fn test_mock_compact_format() {
    let server = setup_server().await;
    Mock::given(method("GET"))
        .and(path("/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "account": [
                {"accountId": "123", "name": "Test"}
            ]
        })))
        .mount(&server)
        .await;

    let mut cmd = Command::cargo_bin("gtm").expect("binary exists");
    cmd.env("GTM_API_BASE", server.uri());
    cmd.env("GTM_QUIET", "1");
    cmd.args(["--format", "compact", "accounts", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("123"))
        .stdout(predicate::str::contains("Test"));
}
