use lambda_http::http::{Method, StatusCode};
use lambda_http::{tracing, Body, Error, Request, Response};

// TODO: move config into separate crate/build script or some lambda env
static PROJECT_BUCKET: &str = "ppp-globalbucket-1";

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
pub(crate) async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    tracing::info!("Received request {:?}", event);

    if let Some(uri) = match event.method() {
        &Method::GET => Some(event.uri()),
        &_ => None,
    } {
        let prefix = &uri.path()[1..];
        tracing::info!("prefix = {}", prefix);

        let sdk_config = aws_config::load_from_env().await;
        tracing::trace!("sdk config = {:?}", sdk_config);

        let s3_client = aws_sdk_s3::Client::new(&sdk_config);
        let objects_at_key = s3_client
            .list_objects()
            .bucket(PROJECT_BUCKET)
            .prefix(prefix)
            .send()
            .await?;

        tracing::info!("list object result = {:?}", objects_at_key);
        tracing::info!("s3 objects at path = {:?}", objects_at_key.contents());

        let object_key_list: Vec<_> = objects_at_key
            .contents()
            .iter()
            .map(|o| o.key())
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect();
        let json_result = serde_json::to_string(&object_key_list[..])?;

        let resp = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(json_result.into())
            .map_err(Box::new)?;
        return Ok(resp);
    }

    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_http::{Request, RequestExt};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_generic_http_handler() {
        let request = Request::default();

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();

        assert_eq!(
            body_string,
            "Hello world, this is an AWS Lambda HTTP request"
        );
    }

    #[tokio::test]
    async fn test_http_handler_with_query_string() {
        let mut query_string_parameters: HashMap<String, String> = HashMap::new();
        query_string_parameters.insert("name".into(), "s3_explorer".into());

        let request = Request::default().with_query_string_parameters(query_string_parameters);

        let response = function_handler(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body_bytes = response.body().to_vec();
        let body_string = String::from_utf8(body_bytes).unwrap();

        assert_eq!(
            body_string,
            "Hello s3_explorer, this is an AWS Lambda HTTP request"
        );
    }
}
