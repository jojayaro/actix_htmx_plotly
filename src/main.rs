use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use reqwest::Client;
use serde_json::Value;
use chrono::{DateTime, Utc, TimeZone};
use plotly::{self, Layout};
use plotly::common::Mode;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_files::{Files};

#[derive(Debug, Serialize, Deserialize)]
struct Magnetic {
    time_tag: DateTime<Utc>,
    bx: f64,
    by: f64,
    bz: f64,
    lon: f64,
    lat: f64,
    bt: f64
}

impl Magnetic {
    fn new() -> Magnetic {
        Magnetic {
            time_tag: Utc::now(),
            bx: 0.0,
            by: 0.0,
            bz: 0.0,
            lon: 0.0,
            lat: 0.0,
            bt: 0.0
        }
    }
    fn from_json(json: Value) -> Magnetic {
        let mut mag = Magnetic::new();
        mag.time_tag = Utc.datetime_from_str(&json[0].to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap();
        //mag.time_tag = NaiveDateTime::parse_from_str(&json[0].to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap().timestamp();
        mag.bx = json[1].to_string().replace("\"","").parse::<f64>().unwrap();
        mag.by = json[2].to_string().replace("\"","").parse::<f64>().unwrap();
        mag.bz = json[3].to_string().replace("\"","").parse::<f64>().unwrap();
        mag.lon = json[4].to_string().replace("\"","").parse::<f64>().unwrap();
        mag.lat = json[5].to_string().replace("\"","").parse::<f64>().unwrap();
        mag.bt = json[6].to_string().replace("\"","").parse::<f64>().unwrap();
        mag
    }
}

fn response_to_magnetic(response: Value) -> Vec<Magnetic> {
    let array: Vec<Value> = response.as_array().unwrap().clone();
    array[1..].iter().map(|x| Magnetic::from_json(x.clone())).collect()
}

async fn url_to_json(url: &str) -> std::result::Result<Value, Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    let res = client
        .get(url)
        .send()
        .await?
        .json::<Value>()
        .await?;
    Ok(res)
}

async fn get_magnetic_data() -> std::result::Result<Vec<Magnetic>, Box<dyn std::error::Error>> {
    let url = "https://services.swpc.noaa.gov/products/solar-wind/mag-2-hour.json";
    let response = url_to_json(url).await?;
    let magnetic_data = response_to_magnetic(response);
    Ok(magnetic_data)
}

#[get("/plot")]
async fn line_plot() -> String {
    let magnetic_data = get_magnetic_data().await.unwrap();
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut z = Vec::new();
    for mag in magnetic_data {
        x.push(mag.time_tag);
        y.push(mag.bz);
        z.push(mag.bt);
    }
    let trace = plotly::Scatter::new(x.clone(), y)
            .name("Bz")
            .mode(Mode::Lines);
    let trace2 = plotly::Scatter::new(x.clone(), z)
            .name("Bt")
            .mode(Mode::Lines);

    let layout = Layout::new()
        .paper_background_color(plotly::color::NamedColor::Black)
        .plot_background_color(plotly::color::NamedColor::Black)
        .y_axis(plotly::layout::Axis::new().range(vec![-40, 40]))
        .title("Magnetic Field".into())
        .show_legend(false);

    let mut plot: plotly::Plot = plotly::Plot::new();
    plot.add_trace(trace);
    plot.add_trace(trace2);
    plot.set_layout(layout);
    
    //plot.write_html("src/out.html");
    plot.to_inline_html(Some("div2"))
}

#[get("/")]
async fn index() -> impl Responder {

    let mut builder = HttpResponse::Ok();
    builder.content_type("text/html; charset=utf-8");
    builder.body(include_str!("plot.html"))

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("key.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(|| App::new()
        .service(index)
        .service(line_plot)
        .service(Files::new("/static", "./static")
    ))
        .bind_openssl("0.0.0.0:8080", builder)?
        .run()
        .await

}