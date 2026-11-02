mod bindings {
    wit_bindgen::generate!({
        path: "wit/world.wit",
        world: "function",
    });

    use super::Component;
    export!(Component);
}

struct Component;

const FUNCTION_NAME: &str = "{{project-name}}";

type HandlerError = bindings::exports::fluor::fun::handler::Error;
type HandlerResponse = bindings::exports::fluor::fun::handler::Response;
type Request = bindings::exports::fluor::fun::handler::Request;

impl bindings::exports::fluor::fun::handler::Guest for Component {
    fn handle(request: Request) -> Result<HandlerResponse, HandlerError> {
        let request_id = header_value(&request.headers, "x-request-id");
        match request.method.as_str() {
            "GET" => get_items(&request, request_id),
            "POST" => create_item(&request, request_id),
            "PUT" => update_item(&request, request_id),
            "DELETE" => delete_item(&request, request_id),
            method => Err(failure(
                bindings::exports::fluor::fun::handler::ErrorCode::BadRequest,
                &format!("method {method} is not supported"),
            )),
        }
    }
}

fn get_items(
    request: &Request,
    request_id: Option<String>,
) -> Result<HandlerResponse, HandlerError> {
    let name = query_param(request.query.as_deref(), "name");
    let mut items = vec!["notebook", "pencil"];
    if let Some(name) = name.as_deref() {
        if !items.contains(&name) {
            return Err(failure(
                bindings::exports::fluor::fun::handler::ErrorCode::NotFound,
                &format!("item {name} was not found"),
            ));
        }
        items.retain(|item| *item == name);
    }

    let items = items
        .iter()
        .map(|item| serde_json::json!({ "name": item }))
        .collect::<Vec<_>>();
    Ok(respond(
        200,
        request_id.as_deref(),
        &document(request, request_id.as_deref(), "items", serde_json::Value::Array(items)),
    ))
}

fn create_item(
    request: &Request,
    request_id: Option<String>,
) -> Result<HandlerResponse, HandlerError> {
    let name = json_string_field(&request.body, "name").ok_or_else(|| {
        failure(
            bindings::exports::fluor::fun::handler::ErrorCode::BadRequest,
            "body must be a JSON object with a string name",
        )
    })?;
    Ok(respond(
        201,
        request_id.as_deref(),
        &document(
            request,
            request_id.as_deref(),
            "created",
            serde_json::json!({ "name": name }),
        ),
    ))
}

fn update_item(
    request: &Request,
    request_id: Option<String>,
) -> Result<HandlerResponse, HandlerError> {
    let id = query_param(request.query.as_deref(), "id").ok_or_else(|| {
        failure(
            bindings::exports::fluor::fun::handler::ErrorCode::BadRequest,
            "query parameter id is required",
        )
    })?;
    let body = json_object(&request.body).ok_or_else(|| {
        failure(
            bindings::exports::fluor::fun::handler::ErrorCode::BadRequest,
            "body must be a JSON object",
        )
    })?;
    Ok(respond(
        200,
        request_id.as_deref(),
        &document(
            request,
            request_id.as_deref(),
            "updated",
            serde_json::json!({ "id": id, "body": body }),
        ),
    ))
}

fn delete_item(
    request: &Request,
    request_id: Option<String>,
) -> Result<HandlerResponse, HandlerError> {
    let id = query_param(request.query.as_deref(), "id").ok_or_else(|| {
        failure(
            bindings::exports::fluor::fun::handler::ErrorCode::BadRequest,
            "query parameter id is required",
        )
    })?;
    Ok(respond(
        200,
        request_id.as_deref(),
        &document(
            request,
            request_id.as_deref(),
            "deleted",
            serde_json::Value::String(id),
        ),
    ))
}

fn respond(status: u16, request_id: Option<&str>, body: &str) -> HandlerResponse {
    let mut headers = vec![bindings::exports::fluor::fun::handler::Header {
        name: "content-type".to_string(),
        value: "application/json".to_string(),
    }];
    if let Some(request_id) = request_id {
        headers.push(bindings::exports::fluor::fun::handler::Header {
            name: "x-request-id".to_string(),
            value: request_id.to_string(),
        });
    }
    HandlerResponse {
        status,
        headers,
        body: body.to_string(),
    }
}

fn failure(
    code: bindings::exports::fluor::fun::handler::ErrorCode,
    message: &str,
) -> HandlerError {
    HandlerError {
        code,
        message: message.to_string(),
    }
}

fn header_value(
    headers: &[bindings::exports::fluor::fun::handler::Header],
    name: &str,
) -> Option<String> {
    headers.iter().find_map(|header| {
        header
            .name
            .eq_ignore_ascii_case(name)
            .then(|| header.value.clone())
    })
}

fn document(
    request: &Request,
    request_id: Option<&str>,
    field: &str,
    value: serde_json::Value,
) -> String {
    let mut payload = serde_json::json!({
        "function": FUNCTION_NAME,
        "language": "rust",
        "method": request.method,
        "path": request.path,
        "query": request.query,
        "requestId": request_id,
    });
    payload[field] = value;
    payload.to_string()
}

fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    let query = query?;
    query.split('&').find_map(|part| {
        let (name, value) = part.split_once('=')?;
        (form_decode(name) == key).then(|| form_decode(value))
    })
}

fn form_decode(value: &str) -> String {
    let mut bytes = Vec::new();
    let input = value.as_bytes();
    let mut index = 0;
    while index < input.len() {
        match input[index] {
            b'+' => {
                bytes.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < input.len() => {
                let Some(byte) = input
                    .get(index + 1..index + 3)
                    .and_then(|hex| std::str::from_utf8(hex).ok())
                    .and_then(|hex| u8::from_str_radix(hex, 16).ok())
                else {
                    return value.to_string();
                };
                bytes.push(byte);
                index += 3;
            }
            byte => {
                bytes.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| value.to_string())
}

fn json_object(body: &str) -> Option<serde_json::Map<String, serde_json::Value>> {
    match serde_json::from_str(body) {
        Ok(serde_json::Value::Object(object)) => Some(object),
        _ => None,
    }
}

fn json_string_field(body: &str, field: &str) -> Option<String> {
    match json_object(body)?.get(field)? {
        serde_json::Value::String(value) if !value.is_empty() => Some(value.clone()),
        _ => None,
    }
}
