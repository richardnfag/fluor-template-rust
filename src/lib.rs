mod bindings {
    wit_bindgen::generate!({
        path: "wit/world.wit",
        world: "function",
    });

    use super::Component;
    export!(Component);
}

struct Component;

impl bindings::exports::fluor::fun::handler::Guest for Component {
    fn handle(
        _request: bindings::exports::fluor::fun::handler::Request,
    ) -> Result<
        bindings::exports::fluor::fun::handler::Response,
        bindings::exports::fluor::fun::handler::Error,
    > {
        Ok(bindings::exports::fluor::fun::handler::Response {
            status: 200,
            headers: vec![bindings::exports::fluor::fun::handler::Header {
                name: "content-type".to_string(),
                value: "application/json".to_string(),
            }],
            body: r#"{"message":"Hello from {{project-name}}"}"#.to_string(),
        })
    }
}
