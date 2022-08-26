use serde::*;
use serde_json::*;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_sqldb::*;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct SqldbTesterActor {}

type Db = SqlDbSender<WasmHost>;

/// Implementation of HttpServer trait methods
#[async_trait]
impl HttpServer for SqldbTesterActor {
    async fn handle_request(
        &self,
        ctx: &Context,
        req: &HttpRequest,
    ) -> std::result::Result<HttpResponse, RpcError> {
        let path = &req.path[1..req.path.len()];
        let segments: Vec<&str> = path.trim_end_matches('/').split('/').collect();
        match (&req.method.as_str(), segments.as_slice()) {
            (&"GET", ["execute"]) => {
                //let text = form_urlencoded::parse(req.query_string.as_bytes())
                //.find(|(n, _)| n == "name")
                //.map(|(_, v)| v.to_string())
                //.unwrap_or_else(|| "World".to_string());
                let db = SqlDbSender::new();

                Ok(HttpResponse::default())
            }
            (&"GET", ["query"]) => {
                let db = SqlDbSender::new();
                let selstar = select_star(ctx, &db).await;

                Ok(HttpResponse {
                    body: serde_json::to_string(&selstar)?.into_bytes(),
                    ..Default::default()
                })
            }
            (&"GET", ["ping"]) => Ok(HttpResponse {
                body: format!("Hello").as_bytes().to_vec(),
                ..Default::default()
            }),
            (_, _) => Ok(HttpResponse::not_found()),
        }
    }
}

async fn select_star(ctx: &Context, client: &Db) -> Result<Vec<UserType>, SqlDbError> {
    let resp = client
        .query(
            ctx,
            &Statement {
                sql: format!("select *"),
                database: Some("derp".to_string()),
                ..Default::default()
            },
        )
        .await?;
    let rows: Vec<UserType> = safe_decode(&resp)?;
    Ok(rows)
}

#[derive(Default, Serialize, Deserialize, minicbor::Decode, Clone)]
struct UserType {
    #[n(0)]
    pub id: u64,
    #[n(1)]
    pub name: String,
}

fn safe_decode<'b, T>(resp: &'b QueryResult) -> Result<Vec<T>, minicbor::decode::Error>
where
    T: Default + minicbor::Decode<'b, ()>,
{
    if resp.num_rows == 0 {
        Ok(Vec::new())
    } else {
        wasmbus_rpc::minicbor::decode(&resp.rows)
    }
}
