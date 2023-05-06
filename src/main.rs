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

struct Plasma {
    time_tag: DateTime<Utc>,
    density: f32,
    speed: f32,
    temperature: f32
}

impl Plasma{
    fn new() -> Plasma {
        Plasma {
            time_tag: Utc::now(),
            density: 0.0,
            speed: 0.0,
            temperature: 0.0
        }
    }
    fn from_json(json: Value) -> Plasma {
        let mut plasma = Plasma::new();
        plasma.time_tag = Utc.datetime_from_str(&json[0].to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap();
        plasma.density = json[1].to_string().replace("\"","").parse::<f32>().unwrap();
        plasma.speed = json[2].to_string().replace("\"","").parse::<f32>().unwrap();
        plasma.temperature = json[3].to_string().replace("\"","").parse::<f32>().unwrap();
        plasma
    }
}

fn response_to_plasma(response: Value) -> Vec<Plasma> {
    let array: Vec<Value> = response.as_array().unwrap().clone();
    array[1..].iter().map(|x| Plasma::from_json(x.clone())).collect()
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

async fn get_plasma_data() -> std::result::Result<Vec<Plasma>, Box<dyn std::error::Error>> {
    let url = "https://services.swpc.noaa.gov/products/solar-wind/plasma-2-hour.json";
    let response = url_to_json(url).await?;
    let plasma_data = response_to_plasma(response);
    Ok(plasma_data)
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

    let plasma_data = get_plasma_data().await.unwrap();
    let mut x2 = Vec::new();
    let mut y2 = Vec::new();
    let mut y3 = Vec::new();
    let mut y4 = Vec::new();
    for plasma in plasma_data {
        x2.push(plasma.time_tag);
        y2.push(plasma.density);
        y3.push(plasma.speed);
        y4.push(plasma.temperature);
    }


    let trace = plotly::Scatter::new(x.clone(), y)
            .name("Bz")
            .mode(Mode::Lines);
    let trace2 = plotly::Scatter::new(x.clone(), z)
            .name("Bt")
            .mode(Mode::Lines);
    let trace3 = plotly::Scatter::new(x2.clone(), y2)
            .name("Density")
            .mode(Mode::Lines)
            .x_axis("x2")
            .y_axis("y2");
    let trace4 = plotly::Scatter::new(x2.clone(), y3)
            .name("Speed")
            .mode(Mode::Lines)
            .x_axis("x3")
            .y_axis("y3");
    let trace5 = plotly::Scatter::new(x2.clone(), y4)
            .name("Temperature")
            .mode(Mode::Lines)
            .x_axis("x4")
            .y_axis("y4");

    let layout = Layout::new()
        .x_axis(plotly::layout::Axis::new().visible(false))
        .y_axis(plotly::layout::Axis::new().range(vec![-40, 40]))
        .x_axis2(plotly::layout::Axis::new().visible(false))
        //.y_axis2(plotly::layout::Axis::new().range(vec![0, 50]))
        .x_axis3(plotly::layout::Axis::new().visible(false))
        //.y_axis3(plotly::layout::Axis::new().range(vec![0, 1000]))
        //.x_axis4(plotly::layout::Axis::new().visible(false))
        //.y_axis4(plotly::layout::Axis::new().range(vec![0, 10000000]))
        .paper_background_color(plotly::color::NamedColor::Black)
        .plot_background_color(plotly::color::NamedColor::Black)
        //.title("Magnetic Field".into())
        .show_legend(false)
        .height(800)
        .grid(
            plotly::layout::LayoutGrid::new()
                .rows(4)
                .columns(1)
                .pattern(plotly::layout::GridPattern::Independent),
        );
    let config = plotly::configuration::Configuration::new()
        .display_mode_bar(plotly::configuration::DisplayModeBar::False);

    let mut plot: plotly::Plot = plotly::Plot::new();
    plot.add_trace(trace);
    plot.add_trace(trace2);
    plot.add_trace(trace3);
    plot.add_trace(trace4);
    plot.add_trace(trace5);
    plot.set_layout(layout);
    plot.set_configuration(config);
    
    //plot.write_html("src/out.html");
    plot.to_inline_html(Some("div2"))
}

#[get("/")]
async fn index() -> impl Responder {

    let mut builder = HttpResponse::Ok();
    builder.content_type("text/html; charset=utf-8");
    builder.body(include_str!("../static/index.html"))

}

#[get("/aurora")]
async fn aurora_page() -> impl Responder {

    let mut builder = HttpResponse::Ok();
    builder.content_type("text/html; charset=utf-8");
    builder.body(include_str!("../static/plot.html"))

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
        .service(aurora_page)
        .service(Files::new("/static", "./static")
    ))
        .bind_openssl("0.0.0.0:8080", builder)?
        .run()
        .await

}