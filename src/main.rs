use aws_sdk_dynamodb::{model::AttributeValue, Client};
use lambda_http::{service_fn, Error, IntoResponse, Request, Response};
use serde::{Deserialize, Serialize};
use serde_dynamo::{from_item, to_item};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_from_env().await;
    let table_name = env::var("TABLE_NAME").expect("TABLE_NAME must be set");
    let dynamodb_client = Client::new(&config);

    lambda_http::run(service_fn(|request: Request| {
        return handler(request, &dynamodb_client, &table_name);
    }))
    .await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Entry {
    token: String,
    hash: String,
}

async fn handler(
    request: Request,
    client: &aws_sdk_dynamodb::Client,
    table_name: &str,
) -> Result<impl IntoResponse, Error> {
    println!("Entering the handler");

    let idempotency_token = match request.headers().get("Idempotency-Token") {
        Some(token) => token,
        None => {
            let response = Response::builder()
                .status(200)
                .body(String::from("No idempotency token"))?;

            return Ok(response);
        }
    };

    println!("After getting the token from the header");

    let payload = request.body();

    println!("Before payload hash");

    let payload_hash = md5::compute(payload);

    println!("After payload hash");

    let idempotency_token = idempotency_token.to_str()?;

    println!("Before getting the record");

    let existing_record = client
        .get_item()
        .table_name(table_name)
        .key("token", AttributeValue::S(idempotency_token.to_string()))
        .send()
        .await;

    /* Mapping errors seem a bit cumbersome? */
    let existing_record = match existing_record {
        Ok(value) => value,
        Err(err) => {
            let error_message = err.to_string();
            let response = Response::builder().status(500).body(error_message)?;

            return Ok(response);
        }
    };

    println!("After getting the record");

    let record: Entry = match existing_record.item() {
        Some(item) => from_item(item.to_owned())?,
        None => {
            return save_entry(&payload_hash, idempotency_token, table_name, &client).await;
        }
    };

    if record.hash != format!("{:?}", payload_hash) {
        let response = Response::builder()
            .status(400)
            .body(String::from("Bad hash value"))?;

        return Ok(response);
    }

    let response = Response::builder().status(200).body(String::from(
        "Got the response and the idempotency tokens match",
    ))?;

    return Ok(response);
}

async fn save_entry(
    hash: &md5::Digest,
    token: &str,
    table_name: &str,
    client: &aws_sdk_dynamodb::Client,
) -> Result<Response<String>, Error> {
    let entry = Entry {
        hash: format!("{:?}", hash),
        token: token.to_owned(),
    };
    let item = to_item(entry)?;

    client
        .put_item()
        .table_name(table_name)
        .set_item(Some(item))
        .send()
        .await?;

    return Ok(Response::builder().status(200).body(String::from(
        "Saved the item alongside the idempotency token",
    ))?);
}
