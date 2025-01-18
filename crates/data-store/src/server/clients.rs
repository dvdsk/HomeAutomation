use crate::data::Data;
use crate::api::{self, ServerError};

pub(crate) async fn handle(port: u16, data: Data) -> color_eyre::Result<()> {
    rpc::server::run(
        port,
        move |req, _| {
            let data = data.clone();
            perform_request(req, data)
        },
        Option::<rpc::SubscribersUnsupported<api::Response>>::None,
    )
    .await
}
async fn perform_request(request: api::Request, data: Data) -> api::Response {
    match perform_request_inner(request, data).await {
        Ok(resp) => resp,
        Err(e) => api::Response::Error(e),
    }
}

async fn perform_request_inner(
    request: api::Request,
    data: Data,
) -> Result<api::Response, ServerError> {
    Ok(match request {
        api::Request::ListData => api::Response::ListData(data.list().await),
        api::Request::GetData {
            reading,
            start,
            end,
            n,
        } => {
            let res = data.get(reading, start, end, n).await;
            api::Response::GetData(res)
        }
    })
}
