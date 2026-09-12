//! Reference content projected from the backend's registered operations.
use super::DeveloperOperation;
use dioxus::prelude::*;

#[component]
pub fn Introduction(operations: Vec<DeveloperOperation>, spec: String) -> Element {
    let api_url = serde_json::from_str::<serde_json::Value>(&spec)
        .ok()
        .and_then(|value| {
            value
                .pointer("/servers/0/url")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        });
    let example = operations.iter().find(|operation| {
        operation.api_key_callable && !operation.mutation && operation.method == "GET"
    });
    rsx! {
        div { class: "fe-api-introduction",
            section { class: "fe-panel", id: "api-quick-start",
                h2 { "Quick start" }
                p { "Create an API key in your developer account, select the available scopes your integration needs, and save the secret when it is shown." }
                div { class: "fe-actions",
                    crate::fullstack::shell::ShellLink { class: "fe-button", href: "/developer", "Manage API keys" }
                    crate::fullstack::shell::ShellLink { class: "fe-button", href: "/developer/usage", "View API usage" }
                    a { class: "fe-button", href: "/api/v1/developer/openapi", "OpenAPI specification" }
                }
                if let Some(api_url) = api_url {
                    p { class: "fe-help", "Set EPSX_API_URL to " code { "{api_url}" } " and EPSX_API_KEY to your secret key in your server environment." }
                }
                if let Some(operation) = example {
                    for (language, code) in examples(&operation.path) {
                        details { class: "fe-api-example",
                            summary { "{language}" }
                            pre { code { "{code}" } }
                        }
                    }
                }
            }
            section { class: "fe-panel", id: "api-authentication",
                h2 { "Authentication and rate limits" }
                p { "Send your API key in the Authorization: Bearer header. Use it from your server; keep the secret out of browser code and source control." }
                p { "Each operation lists its required scopes. Operations marked Browser session use your signed-in account. Your developer account shows the rate limits and API access returned for your plan." }
                crate::fullstack::shell::ShellLink { class: "fe-text-link", href: "/developer", "View your access and rate limits" }
            }
            nav { class: "fe-panel fe-api-index", aria_label: "API operations",
                h2 { "API reference" }
                for operation in operations.iter() {
                    a { href: format!("#operation-{}", operation.operation_id),
                        span { class: "fe-api-method", "{operation.method}" }
                        code { "{operation.path}" }
                    }
                }
            }
        }
    }
}

fn examples(path: &str) -> [(&'static str, String); 3] {
    // Paths are display-only and validated by decode_openapi. JSON quoting keeps
    // reference strings literal in JavaScript/Python snippets.
    let quoted_path = serde_json::to_string(path).unwrap_or_default();
    [
        ("cURL", format!("curl --get \"$EPSX_API_URL{path}\" \\\n  --header \"Authorization: Bearer $EPSX_API_KEY\"")),
        ("JavaScript (server)", format!("const response = await fetch(new URL({quoted_path}, process.env.EPSX_API_URL), {{\n  headers: {{ Authorization: `Bearer ${{process.env.EPSX_API_KEY}}` }}\n}});\nif (!response.ok) throw new Error(`HTTP ${{response.status}}`);\nconst data = await response.json();")),
        ("Python", format!("import json, os, urllib.request\n\nrequest = urllib.request.Request(\n    os.environ[\"EPSX_API_URL\"].rstrip(\"/\") + {quoted_path},\n    headers={{\"Authorization\": \"Bearer \" + os.environ[\"EPSX_API_KEY\"]}},\n)\nwith urllib.request.urlopen(request) as response:\n    data = json.load(response)")),
    ]
}

#[component]
pub fn OperationDetails(operation: DeveloperOperation) -> Element {
    rsx! {
        div { class: "fe-api-details",
            p { class: "fe-help",
                if operation.api_key_callable { "Authentication: API key" } else { "Authentication: Browser session" }
            }
            if !operation.parameters.is_empty() {
                h3 { "Parameters" }
                div { class: "overflow-x-auto", table {
                    thead { tr { th { "Name" } th { "Location" } th { "Required" } th { "Description / schema" } } }
                    tbody { for parameter in operation.parameters.iter() {
                        tr { td { code { "{parameter.name}" } } td { "{parameter.location}" }
                            td { if parameter.required { "Yes" } else { "No" } }
                            td { "{parameter.description}" code { class: "block", "{parameter.schema}" } }
                        }
                    } }
                } }
            }
            if let Some(body) = &operation.request_body {
                details { class: "fe-api-example", summary { "Request body" } pre { code { "{body}" } } }
            }
            if !operation.responses.is_empty() {
                details { class: "fe-api-example", summary { "Responses" }
                    dl { class: "fe-api-responses", for (code, description) in operation.responses.iter() {
                        div { dt { code { "{code}" } } dd { "{description}" } }
                    } }
                }
            }
        }
    }
}
