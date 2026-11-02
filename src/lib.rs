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

    Ok(respond(
        200,
        request_id.as_deref(),
        &format!(
            "{{\"function\":{},\"language\":\"rust\",\"method\":{},\"path\":{},\"query\":{},\"requestId\":{},\"items\":[{}]}}",
            json_string(FUNCTION_NAME),
            json_string(&request.method),
            json_string(&request.path),
            json_optional(request.query.as_deref()),
            json_optional(request_id.as_deref()),
            items
                .iter()
                .map(|item| format!(r#"{{"name":{}}}"#, json_string(item)))
                .collect::<Vec<_>>()
                .join(",")
        ),
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
        &format!(
            "{{\"function\":{},\"language\":\"rust\",\"method\":{},\"path\":{},\"query\":{},\"requestId\":{},\"created\":{{\"name\":{}}}}}",
            json_string(FUNCTION_NAME),
            json_string(&request.method),
            json_string(&request.path),
            json_optional(request.query.as_deref()),
            json_optional(request_id.as_deref()),
            json_string(&name)
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
    if !request.body.trim_start().starts_with('{') {
        return Err(failure(
            bindings::exports::fluor::fun::handler::ErrorCode::BadRequest,
            "body must be a JSON object",
        ));
    }
    Ok(respond(
        200,
        request_id.as_deref(),
        &format!(
            "{{\"function\":{},\"language\":\"rust\",\"method\":{},\"path\":{},\"query\":{},\"requestId\":{},\"updated\":{{\"id\":{},\"body\":{}}}}}",
            json_string(FUNCTION_NAME),
            json_string(&request.method),
            json_string(&request.path),
            json_optional(request.query.as_deref()),
            json_optional(request_id.as_deref()),
            json_string(&id),
            json_string(request.body.trim())
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
        &format!(
            "{{\"function\":{},\"language\":\"rust\",\"method\":{},\"path\":{},\"query\":{},\"requestId\":{},\"deleted\":{}}}",
            json_string(FUNCTION_NAME),
            json_string(&request.method),
            json_string(&request.path),
            json_optional(request.query.as_deref()),
            json_optional(request_id.as_deref()),
            json_string(&id)
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

fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    let query = query?;
    query.split('&').find_map(|part| {
        let (name, value) = part.split_once('=')?;
        (name == key).then(|| value.to_string())
    })
}

fn json_string_field(body: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\"");
    let rest = body.get(body.find(&needle)? + needle.len()..)?;
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut value = String::new();
    let mut escaped = false;
    for ch in rest.chars() {
        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(value),
            _ => value.push(ch),
        }
    }
    None
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn json_optional(value: Option<&str>) -> String {
    value.map(json_string).unwrap_or_else(|| "null".to_string())
}
