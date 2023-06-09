use std::sync::Arc;
use actix_web::{get, post, App, HttpServer, Responder, HttpResponse, web};
use serde::{Serialize, Deserialize};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_files::Files;
use rayon::prelude::*;
use deltalake::{*, arrow::array::{Float64Array, StringArray}};
use datafusion::prelude::SessionContext;
use chrono::Utc;

async fn get_timestamp_from_hours(hours: i64) -> i64 {
    let datetime = Utc::now() - chrono::Duration::hours(hours);
    let timestamp = datetime.timestamp();
    timestamp
}

async fn delta_data(hours: i64) -> (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {

    let timestamp = get_timestamp_from_hours(hours).await;

    let ctx = SessionContext::new();
    let table = deltalake::open_table("/home/sidefxs/solar_wind/".to_string())
        .await
        .unwrap();
    ctx.register_table("solar_wind", Arc::new(table)).unwrap();

    let sql = format!("SELECT * FROM solar_wind WHERE timestamp > '{}%'", timestamp);
  
    let batches = ctx
        .sql(&sql).await.unwrap()
        .collect()
        .await.unwrap();

    let time_tag_vec = batches
        .par_iter()
        .map(|x| x.column(1).as_any().downcast_ref::<StringArray>().unwrap().iter().map(|x| x.unwrap().to_string())
        .collect::<Vec<String>>())
        .flatten()
        .collect::<Vec<String>>();

    let speed_vec = batches
        .par_iter()
        .map(|x| x.column(2).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    let density_vec = batches
        .par_iter()
        .map(|x| x.column(3).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    let temperature_vec = batches
        .par_iter()
        .map(|x| x.column(4).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    let bt_vec = batches
        .par_iter()
        .map(|x| x.column(5).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    let bz_vec = batches
        .par_iter()
        .map(|x| x.column(6).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    (time_tag_vec, bt_vec, bz_vec, density_vec, speed_vec, temperature_vec)

}

#[get("/delta-data/{hours}")]
async fn delta_data_handler(hours: web::Path<i64>) -> impl Responder {
    let solarwind = delta_data(*hours).await;
    //println!("{:?}", solarwind);
    HttpResponse::Ok().json(solarwind)
}

#[get("/goes16")] 
async fn goes16() -> String {
    "<img id=\"goes16\" src=\"https://cdn.star.nesdis.noaa.gov/GOES16/GLM/SECTOR/can/EXTENT3/2250x1125.jpg\" width=\"100%\" alt=\"GOES16 Geocolor\">".to_string()
}

#[get("/plot")]
async fn line_plot() -> String {

    let (time_tag, bt, bz, density, speed, temperature) = delta_data(2).await;

    let plot_div: String = format!(
        "<div id=\"div2\" class=\"plotly-graph-div\" style=\"height:100%; width:100%;\"></div>
        <script type=\"text/javascript\">
            Plotly.newPlot(\"div2\", {{
            \"data\": [
            {{
            \"type\": \"scatter\",
            \"name\": \"Bz\",
            \"mode\": \"lines\",
            \"x\": {time_tag:?},
            \"y\": {bz:?}
            }},
            {{
            \"type\": \"scatter\",
            \"name\": \"Bt\",
            \"mode\": \"lines\",
            \"x\": {time_tag:?},
            \"y\": {bt:?}
            }},
            {{
            \"type\": \"scatter\",
            \"name\": \"Density\",
            \"mode\": \"lines\",
            \"x\": {time_tag:?},
            \"y\": {density:?},
            \"xaxis\": \"x2\",
            \"yaxis\": \"y2\"
            }},
            {{
            \"type\": \"scatter\",
            \"name\": \"Speed\",
            \"mode\": \"lines\",
            \"x\": {time_tag:?},
            \"y\": {speed:?},
            \"xaxis\": \"x3\",
            \"yaxis\": \"y3\"
            }},
            {{
            \"type\": \"scatter\",
            \"name\": \"Temperature\",
            \"mode\": \"lines\",
            \"x\": {time_tag:?},
            \"y\": {temperature:?},
            \"xaxis\": \"x4\",
            \"yaxis\": \"y4\"
            }}
        ],
        \"layout\": {{
            \"clickmode\": \"event+select\",
            \"showlegend\": false,
            \"height\": 800,
            \"paper_bgcolor\": \"black\",
            \"plot_bgcolor\": \"black\",
            \"grid\": {{
            \"rows\": 4,
            \"columns\": 1,
            \"pattern\": \"independent\"
            }},
            \"xaxis\": {{
            \"visible\": false
            }},
            \"yaxis\": {{
            \"range\": [
                -40,
                40
            ]
            }},
            \"xaxis2\": {{
            \"visible\": false
            }},
            \"xaxis3\": {{
            \"visible\": false
            }}
        }},
        \"config\": {{
            \"displayModeBar\": false
        }}
        }});
        var plotlyGraph = document.getElementById('div2');
        plotlyGraph.on('plotly_click', function(data) {{
          var pointData = data.points[0].data;
          var xValue = data.points[0].x;
          var yValue = data.points[0].y;

          var xhr = new XMLHttpRequest();
          xhr.open('POST', '/data');
          xhr.setRequestHeader('Content-Type', 'application/json');
          xhr.send(JSON.stringify({{x: xValue, y: yValue}}));
        }});
        </script>");

    plot_div
}

#[get("/plot_update/{hours}")]
async fn line_plot_update(hours: web::Path<i64>) -> impl Responder {

    let (time_tag, bt, bz, density, speed, temperature) = delta_data(*hours).await;

    let body = format!("
    <script type=\"text/javascript\">
    var data = {{
        x: [{time_tag:?}, {time_tag:?}, {time_tag:?}, {time_tag:?}],
        y: [{bz:?}, {bt:?}, {density:?}, {speed:?}, {temperature:?}]
        }};
    Plotly.update(\"div2\", data);
    </script>
    ");

    HttpResponse::Ok().body(body)
}

#[get("/plot_update_hx/{hours}")]
async fn line_plot_update_hx(hours: web::Path<i64>) -> String {

    let time_range = *hours;

    format!("
    <div id=\"plot-updates\" hx-get=\"/plot_update/{time_range}\" hx-trigger=\"load, every 60s\"></div>
    ")
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

#[derive(Debug, Serialize, Deserialize)]
struct Data {
    x: String,
    y: f64,
}

#[post("/data")]
async fn capture_data(data: web::Json<Data>) -> impl Responder {
    println!("Captured data: x={}, y={}", data.x, data.y);
    HttpResponse::Ok()
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
        .service(line_plot_update)
        .service(line_plot_update_hx)
        .service(aurora_page)
        .service(goes16)
        .service(capture_data)
        .service(Files::new("/static", "./static"))
        //.service(Files::new("~/solarwind", "./solarwind"))
        .service(delta_data_handler)
        // .route("/data", web::post().to(capture_data))
    )
        .bind_openssl("0.0.0.0:8080", builder)?
        .run()
        .await

}