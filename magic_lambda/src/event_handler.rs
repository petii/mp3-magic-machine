use aws_lambda_events::event::s3::S3Event;
use aws_lambda_events::s3::{S3Bucket, S3Entity, S3EventRecord, S3Object};
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::primitives::ByteStream;
use channel_io::ChannelReader;
use hound::WavReader;
use lambda_runtime::{tracing, Error, LambdaEvent};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tokio::task::{yield_now, JoinHandle};
use zip::write::SimpleFileOptions;

static PROJECT_BUCKET: &str = "ppp-globalbucket-1";
static ARCHIVE_KEY_BASE: &str = "mp3-magic-machine/archive";

async fn handle_s3_object(
    object_key: String,
    get_object_output: GetObjectOutput,
) -> Result<Vec<PathBuf>, Error> {
    let mut object_body = get_object_output.body;
    let key_last = object_key
        .split('/')
        .last()
        .unwrap()
        .rsplit('.')
        .skip(1)
        .collect::<Vec<&str>>()
        .join("");

    let (tx, rx) = flume::unbounded();

    let body_bytestream_reader_handle: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
        while let Some(body) = object_body.try_next().await.map_err(Box::new)? {
            tx.send(body).map_err(Box::new)?;
            yield_now().await;
        }
        Ok(())
    });

    let object_channel_reader = ChannelReader::new(rx);

    let reader = WavReader::new(object_channel_reader)?;

    tracing::info!("opened file with wav spec: {:?}", reader.spec());

    let mut all_files = Vec::<PathBuf>::new();

    match reader.spec().bits_per_sample {
        16 => {
            let files =
                wav_to_mp3::mp3_encode_i16::from_reader(reader, Path::new("/tmp"), &key_last[..]);
            all_files.extend(files)
        }
        _ => tracing::error!("not 16 bits per sample"),
    }

    body_bytestream_reader_handle.await??;

    Ok(all_files)
}

fn compress_files(files: Vec<PathBuf>, output: PathBuf) -> Result<PathBuf, Error> {
    let out_file = File::create(output.clone())?;

    let mut zip = zip::ZipWriter::new(out_file);

    let mut buffer = Vec::new();

    for file in files {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        let mut file_handle = std::fs::File::open(&file)?;
        file_handle.read_to_end(&mut buffer)?;

        zip.start_file(file_name, SimpleFileOptions::default())?;
        zip.write_all(&buffer)?;

        buffer.clear();
    }

    zip.finish()?;

    Ok(output)
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
pub(crate) async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    // Extract some useful information from the request
    let payload = event.payload;

    if payload.records.len() < 1 {
        tracing::info!("no records; nothing to do");
        return Ok(());
    }

    let sdk_config = aws_config::load_from_env().await;
    tracing::trace!("sdk config = {:?}", sdk_config);

    let s3_client = aws_sdk_s3::Client::new(&sdk_config);

    let mut all_files = Vec::<PathBuf>::new();
    let mut last_event_time = chrono::Utc::now();

    for record in payload.records {
        tracing::debug!("{:?}", record);

        let S3EventRecord {
            event_time,
            s3: event_entity,
            ..
        } = record;

        tracing::info!("at {event_time}: s3 entity = {:?}", event_entity);

        let S3Entity {
            bucket: S3Bucket {
                name: bucket_name, ..
            },
            object: S3Object { key, .. },
            ..
        } = event_entity;

        let get_object_result = s3_client
            .get_object()
            .bucket(bucket_name.clone().unwrap())
            .key(key.clone().unwrap())
            .send()
            .await?;

        tracing::info!("get_object() = {:?}", get_object_result);

        let files = handle_s3_object(key.unwrap(), get_object_result).await?;
        all_files.extend(files);

        last_event_time = event_time;
    }

    let date_as_key = last_event_time.format("%Y/%m-%d");
    let output_key = format!("{ARCHIVE_KEY_BASE}/{date_as_key}.zip");

    tracing::info!("output_key = {output_key}");

    let archive = compress_files(
        all_files,
        Path::new("/tmp")
            .join(format!("{}", last_event_time.format("%Y-%m-%d")))
            .with_extension("zip"),
    )?;

    tracing::info!("archive = {}", archive.display());

    s3_client
        .put_object()
        .bucket(PROJECT_BUCKET)
        .key(output_key)
        .body(ByteStream::from_path(archive).await?)
        .send()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_event_handler() {
        let event = LambdaEvent::new(S3Event::default(), Context::default());
        let response = function_handler(event).await.unwrap();
        assert_eq!(dbg!(response), ());
    }
}
