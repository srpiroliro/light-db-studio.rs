use std::fmt::format;
use std::ops::IndexMut;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use bytes::Bytes;
use http_body_util::Full;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::sql::Reader;

type Resp = Response<Full<Bytes>>;
type Req = Request<Incoming>;

pub async fn init(reader: Reader) -> anyhow::Result<()> {
    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let listener = TcpListener::bind(addr).await?;

    let reader = Arc::new(reader);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let reader = Arc::clone(&reader);

        tokio::spawn(async move {
            // service_fn must return Result<Resp, Infallible>
            let svc = service_fn(move |req: Req| handle(req, Arc::clone(&reader)));
            if let Err(err) = http1::Builder::new().serve_connection(io, svc).await {
                eprintln!("server error: {err}");
            }
        });
    }
}

async fn handle(req: Req, reader: Arc<Reader>) -> Result<Resp, Infallible> {
    let path = req.uri().path();
    let parts: Vec<&str> = path
        .trim_matches('/')
        .split('/')
        .filter(|v| !v.trim().is_empty())
        .collect();

    println!("{:?}", parts);

    let content = match parts.as_slice() {
        [] => {
            let schemas = &reader.schemas().await.unwrap();
            let mut table =
                "<caption>schemas</caption><tr><th>#</th><th>name</th></tr>".to_string();

            for (i, schema) in schemas.iter().enumerate() {
                table.push_str(&format!(
                    "<tr><td>{0}</td><td><a href='/{1}'>{1}</a></td></tr>",
                    i, schema
                ));
            }

            table
        }
        [schema] => {
            let tables = &reader.tables(schema.to_string()).await.unwrap();

            let mut content =
                "<caption>tables</caption><tr><th>#</th><th>name</th><th>count</th></tr>"
                    .to_string();

            for (i, table) in tables.iter().enumerate() {
                content.push_str(&format!(
                    "<tr><td>{0}</td><td><a href='/{1}/{2}'>{2}</a></td></tr>",
                    i, schema, table
                ));
            }

            content
        }
        [schema, table] => {
            let mut content = format!("<caption>{}</caption>", table);

            let data = &reader
                .view(schema.to_string(), table.to_string())
                .await
                .unwrap();
            let mut data_iter = data.into_iter();
            match data_iter.next() {
                None => {
                    content.push_str("<tr>no data</tr>");
                }

                Some(first) => {
                    let keys: String = first
                        .keys()
                        .map(|k| format!("<th>{}</th>", esc_html(k)))
                        .collect();
                    let row = "<tr>".to_string() + &keys + "</tr>";
                    content.push_str(&row);

                    let vals: String = first
                        .values()
                        .map(|k| format!("<td>{}</td>", esc_html(k)))
                        .collect();
                    let row = "<tr>".to_string() + &vals + "</tr>";
                    content.push_str(&row);

                    for row in data_iter {
                        let vals: String = row
                            .values()
                            .map(|k| format!("<td>{}</td>", esc_html(k)))
                            .collect();
                        let row = "<tr>".to_string() + &vals + "</tr>";
                        content.push_str(&row);
                    }
                }
            }

            content
        }
        _ => {
            // 404
            "ERROR!".to_string()
        }
    };

    let html = format!(
        "<!doctype html><meta charset=utf-8><table>{}</table>",
        content
    );

    let body = Full::new(Bytes::from(html));
    let resp = Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(body)
        .unwrap();

    Ok(resp)
}

fn esc_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}
