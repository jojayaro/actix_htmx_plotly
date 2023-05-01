use actix_web::{get, web, App, HttpServer, Responder, HttpRequest, HttpResponse, Result, http::StatusCode};
use serde::de::IntoDeserializer;
use serde::{Serialize, Deserialize};
use reqwest::Client;
use serde_json::Value;
use chrono::{DateTime, Utc, TimeZone, NaiveDateTime};
use plotly::{self, Layout};
use plotly::common::Mode;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_files::{Files, NamedFile};

#[derive(Debug, Serialize, Deserialize)]
struct Magnetic {
    time_tag: i64,
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
            time_tag: 0,
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
        mag.time_tag = NaiveDateTime::parse_from_str(&json[0].to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap().timestamp();
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

async fn line_plot() {
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
    let mut plot: plotly::Plot = plotly::Plot::new();
    plot.add_trace(trace);
    plot.add_trace(trace2);
    
    plot.write_html("src/out.html");
}

async fn line_plot_div() -> String {

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
        .plot_background_color(plotly::color::NamedColor::Black);

    let mut plot: plotly::Plot = plotly::Plot::new();
    plot.add_trace(trace);
    plot.add_trace(trace2);
    plot.set_layout(layout);

    plot.to_inline_html(Some("div"))
}

// #[get("/")]
// async fn index() -> impl Responder {

//     line_plot().await;
//     let mut builder = HttpResponse::Ok();
//     builder.content_type("text/html; charset=utf-8");
//     builder.body(include_str!("out.html"))


// }

#[get("/")]
async fn index() -> impl Responder {
 
    let start: String = "
        <!doctype html>
        <html lang=\"en\">
        <head>
            <meta charset=\"utf-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
            <title>Aurora Monitor</title>
            <link href=\"../static/bootstrap.css\" rel=\"stylesheet\">
            <script>
            function autoRefresh() {
                window.location = window.location.href;
            }
            setInterval('autoRefresh()', 60000);
            </script>
        </head>
        <body>
        <div class=\"navbar navbar-expand-lg fixed-top navbar-dark bg-dark\">
        <div class=\"container\">
          <a href=\"../\" class=\"navbar-brand\">Jesus Jayaro</a>
            </ul>
            <ul class=\"navbar-nav ms-md-auto\">
              <li class=\"nav-item\">
                <a target=\"_blank\" rel=\"noopener\" class=\"nav-link\" href=\"../multiple\"><i class=\"bi bi-twitter\"></i> Aurora Monitor</a>
              </li>
            </ul>
          </div>
        </div>
      </div>
      <script src=\"https://cdn.jsdelivr.net/npm/bootstrap@5.2.3/dist/js/bootstrap.bundle.min.js\"></script>
        <div>
            <script src=\"https://cdn.jsdelivr.net/npm/mathjax@3.2.2/es5/tex-svg.js\"></script>
            <script src=\"https://cdn.plot.ly/plotly-2.12.1.min.js\"></script>
            ".to_string();
    
    let plot = line_plot_div().await;
    
    let end: String = "</div>
                    </body>
                </html>".to_string();
    let html = start + &plot + &end;

    let mut builder = HttpResponse::Ok();
    builder.content_type("text/html; charset=utf-8");
    builder.body(html)

}

#[get("/multiple")]
async fn multiple_plots_using_divs() -> impl Responder {
 
    let start: String = "<!doctype html>
        <html lang=\"en\">
        <head>
            <meta charset=\"utf-8\">
            <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
            <title>Aurora Monitor</title>
            <link href=\"../static/bootstrap.css\" rel=\"stylesheet\">
            <script>
            function autoRefresh() {
                window.location = window.location.href;
            }
            setInterval('autoRefresh()', 60000);
            </script>
        </head>
        <body>
        <div class=\"navbar navbar-expand-lg fixed-top navbar-dark bg-dark\">
        <div class=\"container\">
          <a href=\"../\" class=\"navbar-brand\">Jesus Jayaro</a>
            </ul>
            <ul class=\"navbar-nav ms-md-auto\">
              <li class=\"nav-item\">
                <a target=\"_blank\" rel=\"noopener\" class=\"nav-link\" href=\"../multiple\"><i class=\"bi bi-twitter\"></i> Aurora Monitor</a>
              </li>
            </ul>
          </div>
        </div>
      </div>
      <script src=\"https://cdn.jsdelivr.net/npm/bootstrap@5.2.3/dist/js/bootstrap.bundle.min.js\"></script>
        <div>
            <script src=\"https://cdn.jsdelivr.net/npm/mathjax@3.2.2/es5/tex-svg.js\"></script>
            <script src=\"https://cdn.plot.ly/plotly-2.12.1.min.js\"></script>
            ".to_string();
    
    let plot = line_plot_div().await;
    
    let end: String = "</div>
                    </body>
                </html>".to_string();
    let html = start + &plot + &end;

    let mut builder = HttpResponse::Ok();
    builder.content_type("text/html; charset=utf-8");
    builder.body(html)

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
        .service(multiple_plots_using_divs)
        .service(Files::new("/static", "./static")
    ))
        .bind_openssl("0.0.0.0:8080", builder)?
        .run()
        .await

}