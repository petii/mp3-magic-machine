use aws_lambda_events::event::s3::S3Event;
use aws_lambda_events::s3::{S3Entity, S3EventRecord};
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use lambda_runtime::{tracing, Error, LambdaEvent};

fn handle_s3_object(get_object_output: GetObjectOutput) -> Result<(), Error> {
    // get_object_output.body
    let reader = hound::WavReader::new(get_object_output.body.into())?;

    tracing::info!("opened file with wav spec: {}", reader.spec());

    match reader.spec().bits_per_sample {
        16 => tracing::debug!("this is the part where the conversion should happen"),
        _ => tracing::error!("not 16 bits per sample"),
    }

    Ok(())
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
pub(crate) async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    let lambda_runtime_config = lambda_runtime::Config::from_env();
    tracing::info!("lambda runtime config = {:#?}", lambda_runtime_config);

    // Extract some useful information from the request
    let payload = event.payload;

    let s3_client;
    if payload.records.len() < 1 {
        tracing::info!("no records; nothing to do");
        return Ok(());
    }

    let sdk_config = aws_config::load_from_env().await;
    tracing::trace!("sdk config = {:#?}", sdk_config);

    s3_client = aws_sdk_s3::Client::new(&sdk_config);
    tracing::trace!("s3 client = {:#?}", s3_client);

    for record in payload.records {
        tracing::debug!("{:#?}", record);

        let S3EventRecord {
            event_time,
            s3: event_entity,
            ..
        } = record;

        tracing::info!("at {event_time}: s3 entity = {:#?}", event_entity);

        let S3Entity { bucket, object, .. } = event_entity;

        let get_object_result = s3_client
            .get_object()
            .bucket(bucket.name.unwrap())
            .key(object.key.unwrap())
            .send()
            .await;

        match get_object_result {
            Ok(output) => {
                tracing::info!("get_object() = {:#?}", output);
                handle_s3_object(output)?;
            }
            Err(error) => {
                tracing::error!("get_object() = {:#?}", error);
                return Err(error.into());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    // #[tokio::test]
    // async fn test_event_handler() {
    //     let event = LambdaEvent::new(S3Event::default(), Context::default());
    //     let response = function_handler(event).await;
    //     assert!(response.is_err());
    // }
}
