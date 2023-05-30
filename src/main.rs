use std::sync::Arc;

use actix_web::{get, post, App, HttpServer, Responder, HttpResponse, web};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use actix_files::{Files};
use rayon::prelude::*;
//use actix_web::web::Form;
use deltalake::{*, arrow::array::{Int64Array, Float64Array, StringArray}};
use datafusion::prelude::SessionContext;
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Debug, Serialize, Deserialize)]
struct SolarWind {
    timestamp: i64,
    time_tag: String,
    speed: f64,
    density: f64,
    temperature: f64,
    bt: f64,
    bz: f64
}

async fn plot_data() -> (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let solarwind_url = "https://services.swpc.noaa.gov/products/geospace/propagated-solar-wind-1-hour.json";

    let response = reqwest::get(solarwind_url)
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap()
        .as_array()
        .unwrap()
        .par_iter()
        .skip(1)
        .map(|x| x.clone())
        .collect::<Vec<Value>>();

    let time_tag = response
        .par_iter()
        .map(|x| x[0].to_string().replace("\"",""))
        .collect::<Vec<String>>();

    let speed = response
        .par_iter()
        .map(|x| x[1].to_string().replace("\"","").parse::<f64>().unwrap_or(0.0))
        .collect::<Vec<f64>>();

    let density = response
        .par_iter()
        .map(|x| x[2].to_string().replace("\"","").parse::<f64>().unwrap_or(0.0))
        .collect::<Vec<f64>>();

    let temperature = response
        .par_iter()
        .map(|x| x[3].to_string().replace("\"","").parse::<f64>().unwrap_or(0.0))
        .collect::<Vec<f64>>();

    let bt = response
        .par_iter()
        .map(|x| x[7].to_string().replace("\"","").parse::<f64>().unwrap_or(0.0))
        .collect::<Vec<f64>>();

    let bz = response
        .par_iter()
        .map(|x| x[6].to_string().replace("\"","").parse::<f64>().unwrap_or(0.0))
        .collect::<Vec<f64>>();

    (time_tag, bt, bz, density, speed, temperature)
}

async fn get_timestamp(datetime: String) -> i64 {
    let datetime = NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d-%H:%M:%S").unwrap();
    let datetime = DateTime::<Utc>::from_utc(datetime, Utc);
    let timestamp = datetime.timestamp();
    timestamp
}

async fn get_timestamp_from_hours(hours: i64) -> i64 {
    let datetime = Utc::now() - chrono::Duration::hours(hours);
    let timestamp = datetime.timestamp();
    timestamp
}

//async fn delta_data(datetime: String) -> (Vec<String>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    //let timestamp = get_timestamp(datetime).await;

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

    // let timestamp_vec = batches
    //     .iter()
    //     .map(|x| x.column(0).as_any().downcast_ref::<Int64Array>().unwrap().values().to_vec())
    //     .flatten()
    //     .collect::<Vec<i64>>();

    let time_tag_vec = batches
        .iter()
        .map(|x| x.column(1).as_any().downcast_ref::<StringArray>().unwrap().iter().map(|x| x.unwrap().to_string())
        .collect::<Vec<String>>())
        .flatten()
        .collect::<Vec<String>>();

    let speed_vec = batches
        .iter()
        .map(|x| x.column(2).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    let density_vec = batches
        .iter()
        .map(|x| x.column(3).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    let temperature_vec = batches
        .iter()
        .map(|x| x.column(4).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    let bt_vec = batches
        .iter()
        .map(|x| x.column(5).as_any().downcast_ref::<Float64Array>().unwrap().values().to_vec())
        .flatten()
        .collect::<Vec<f64>>();

    let bz_vec = batches
        .iter()
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

    //let (time_tag, bt, bz, density, speed, temperature) = plot_data().await;

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

    //let (time_tag, bt, bz, density, speed, temperature) = plot_data().await;

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

    //let (time_tag, bt, bz, density, speed, temperature) = plot_data().await;

    // let (time_tag, bt, bz, density, speed, temperature) = delta_data(*hours).await;

    let time_range = *hours;

    format!("
    <div id=\"plot-updates\" hx-get=\"/plot_update/{time_range}\" hx-trigger=\"load, every 60s\"></div>
    ")
}

// #[get("/plot_update/{hours}")]
// async fn line_plot_update(hours: web::Path<i64>) -> String {

//     //let (time_tag, bt, bz, density, speed, temperature) = plot_data().await;

//     let (time_tag, bt, bz, density, speed, temperature) = delta_data(*hours).await;

//     format!("
//     <script type=\"text/javascript\">
//     var data = {{
//         x: [{time_tag:?}, {time_tag:?}, {time_tag:?}, {time_tag:?}],
//         y: [{bz:?}, {bt:?}, {density:?}, {speed:?}, {temperature:?}]
//         }};
//     Plotly.update(\"div2\", data);
//     </script>
//     ")
// }

// #[get("/")]
// async fn index() -> impl Responder {
//     let mut builder = HttpResponse::Ok();
//     builder.content_type("text/html; charset=utf-8");
//     builder.body(include_str!("../static/index.html"))

// }

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

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     HttpServer::new(|| App::new()
//         .service(index)
//         .service(line_plot)
//         .service(line_plot_update)
//         .service(aurora_page)
//         .service(goes16)
//         .service(capture_data)
// )
//         .bind("127.0.0.1:8081")?
//         .run()
//         .await
// }

        // div2.on('plotly_click', function (eventData) {{
        //     const pointData = eventData.points[0].data;
        // htmx.ajax({{
        //     url: '/capture-data',
        //     method: 'post',
        //     body: {{ data: pointData }},
        //     headers: {{ 'Content-Type': 'application/json' }}
        //     }});
        // }});

// #[derive(Debug, Serialize, Deserialize)]
// struct Magnetic {
//     time_tag: DateTime<Utc>,
//     bx: f64,
//     by: f64,
//     bz: f64,
//     lon: f64,
//     lat: f64,
//     bt: f64
// }

// impl Magnetic {
//     fn new() -> Magnetic {
//         Magnetic {
//             time_tag: Utc::now(),
//             bx: 0.0,
//             by: 0.0,
//             bz: 0.0,
//             lon: 0.0,
//             lat: 0.0,
//             bt: 0.0
//         }
//     }
//     fn from_json(json: Value) -> Magnetic {
//         let mut mag = Magnetic::new();
//         mag.time_tag = Utc.datetime_from_str(&json[0].to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap();
//         //mag.time_tag = NaiveDateTime::parse_from_str(&json[0].to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap().timestamp();
//         mag.bx = json[1].to_string().replace("\"","").parse::<f64>().unwrap();
//         mag.by = json[2].to_string().replace("\"","").parse::<f64>().unwrap();
//         mag.bz = json[3].to_string().replace("\"","").parse::<f64>().unwrap();
//         mag.lon = json[4].to_string().replace("\"","").parse::<f64>().unwrap();
//         mag.lat = json[5].to_string().replace("\"","").parse::<f64>().unwrap();
//         mag.bt = json[6].to_string().replace("\"","").parse::<f64>().unwrap();
//         mag
//     }
// }

// struct Plasma {
//     time_tag: DateTime<Utc>,
//     density: f32,
//     speed: f32,
//     temperature: f32
// }

// impl Plasma{
//     fn new() -> Plasma {
//         Plasma {
//             time_tag: Utc::now(),
//             density: 0.0,
//             speed: 0.0,
//             temperature: 0.0
//         }
//     }
//     fn from_json(json: Value) -> Plasma {
//         let mut plasma = Plasma::new();
//         plasma.time_tag = Utc.datetime_from_str(&json[0].to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap();
//         plasma.density = json[1].to_string().replace("\"","").parse::<f32>().unwrap();
//         plasma.speed = json[2].to_string().replace("\"","").parse::<f32>().unwrap();
//         plasma.temperature = json[3].to_string().replace("\"","").parse::<f32>().unwrap();
//         plasma
//     }
// }

// fn response_to_plasma(response: Value) -> Vec<Plasma> {
//     let array: Vec<Value> = response.as_array().unwrap().clone();
//     array[1..].par_iter().map(|x| Plasma::from_json(x.clone())).collect()
// }

// fn response_to_magnetic(response: Value) -> Vec<Magnetic> {
//     let array: Vec<Value> = response.as_array().unwrap().clone();
//     array[1..].par_iter().map(|x| Magnetic::from_json(x.clone())).collect()
// }

// async fn url_to_json(url: &str) -> std::result::Result<Value, Box<dyn std::error::Error>> {
//     let client = Client::builder().build()?;
//     let res = client
//         .get(url)
//         .send()
//         .await?
//         .json::<Value>()
//         .await?;
//     Ok(res)
// }

// async fn get_magnetic_data() -> std::result::Result<Vec<Magnetic>, Box<dyn std::error::Error>> {
//     let url = "https://services.swpc.noaa.gov/products/solar-wind/mag-2-hour.json";
//     let response = url_to_json(url).await?;
//     let magnetic_data = response_to_magnetic(response);
//     Ok(magnetic_data)
// }

// async fn get_plasma_data() -> std::result::Result<Vec<Plasma>, Box<dyn std::error::Error>> {
//     let url = "https://services.swpc.noaa.gov/products/solar-wind/plasma-2-hour.json";
//     let response = url_to_json(url).await?;
//     let plasma_data = response_to_plasma(response);
//     Ok(plasma_data)
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Data {
//     time_tag: String,
//     bx_gsm: String,
//     by_gsm: String,
//     bz_gsm: String,
//     lon_gsm: String,
//     lat_gsm: String,
//     bt: String,
// }

// #[derive(Debug, Serialize, Deserialize)]
// struct Data2 {
//     time_tag: String,
//     density: String,
//     speed: String,
//     temperature: String,
// }

    // let solarwind_url = "https://services.swpc.noaa.gov/products/geospace/propagated-solar-wind-1-hour.json";

    // let response = reqwest::get(solarwind_url)
    //     .await
    //     .unwrap()
    //     .json::<Value>()
    //     .await
    //     .unwrap()
    //     .as_array()
    //     .unwrap()
    //     .par_iter()
    //     .skip(1)
    //     .map(|x| x.clone())
    //     .collect::<Vec<Value>>();

    // let time_tag = response
    //     .par_iter()
    //     .map(|x| x[0].to_string().replace("\"",""))
    //     .collect::<Vec<String>>();

    // let speed = response
    //     .par_iter()
    //     .map(|x| x[1].to_string().replace("\"","").parse::<f64>().unwrap())
    //     .collect::<Vec<f64>>();

    // let density = response
    //     .par_iter()
    //     .map(|x| x[2].to_string().replace("\"","").parse::<f64>().unwrap())
    //     .collect::<Vec<f64>>();

    // let temperature = response
    //     .par_iter()
    //     .map(|x| x[3].to_string().replace("\"","").parse::<f64>().unwrap())
    //     .collect::<Vec<f64>>();

    // let bt = response
    //     .par_iter()
    //     .map(|x| x[7].to_string().replace("\"","").parse::<f64>().unwrap())
    //     .collect::<Vec<f64>>();

    // let bz = response
    //     .par_iter()
    //     .map(|x| x[6].to_string().replace("\"","").parse::<f64>().unwrap())
    //     .collect::<Vec<f64>>();

    // #[get("/plot")]
// async fn line_plot() -> String {
//     // include_str!("../static/plot_div.html").to_string()
//     let mag_url = "https://services.swpc.noaa.gov/products/solar-wind/mag-2-hour.json";
//     let response = reqwest::get(mag_url).await.unwrap().text().await.unwrap();

//     let plasma_url = "https://services.swpc.noaa.gov/products/solar-wind/plasma-2-hour.json";
//     let response2 = reqwest::get(plasma_url).await.unwrap().text().await.unwrap();

//     let data: Vec<Data> = serde_json::from_str(&response).unwrap();
//     let data2: Vec<Data2> = serde_json::from_str(&response2).unwrap();

//     let time_tags: Vec<DateTime<Utc>> = data.par_iter().skip(1)
//         .map(|d| Utc.datetime_from_str(&d.time_tag.to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap())
//         .collect();

//     let bz_gsms: Vec<f64> = data.par_iter().skip(1)
//         .map(|d| d.bz_gsm.to_string().replace("\"","").parse::<f64>().unwrap())
//         .collect();

//     let bt_gsms: Vec<f64> = data.par_iter().skip(1)
//         .map(|d| d.bt.to_string().replace("\"","").parse::<f64>().unwrap())
//         .collect();

//     let time_tags2: Vec<DateTime<Utc>> = data2.par_iter()
//         .skip(1) // Skip the first item (labels of the fields)
//         .map(|d| Utc.datetime_from_str(&d.time_tag.to_string().replace("\"",""), "%Y-%m-%d %H:%M:%S%.3f").unwrap())
//         .collect();

//     let densities: Vec<f64> = data2.par_iter()
//         .skip(1) // Skip the first item (labels of the fields)
//         .map(|d| d.density.to_string().replace("\"","").parse::<f64>().unwrap())
//         .collect();

//     let speeds: Vec<f64> = data2.par_iter()
//         .skip(1) // Skip the first item (labels of the fields)
//         .map(|d| d.speed.to_string().replace("\"","").parse::<f64>().unwrap())
//         .collect();

//     let temperatures: Vec<f64> = data2.par_iter()
//         .skip(1) // Skip the first item (labels of the fields)
//         .map(|d| d.temperature.to_string().replace("\"","").parse::<f64>().unwrap())
//         .collect();



//     // let magnetic_data = get_magnetic_data().await.unwrap();
//     // let mut x = Vec::new();
//     // let mut y = Vec::new();
//     // let mut z = Vec::new();
//     // for mag in magnetic_data {
//     //     x.push(mag.time_tag);
//     //     y.push(mag.bz);
//     //     z.push(mag.bt);
//     // }

//     // let plasma_data = get_plasma_data().await.unwrap();
//     // let mut x2 = Vec::new();
//     // let mut y2 = Vec::new();
//     // let mut y3 = Vec::new();
//     // let mut y4 = Vec::new();
//     // for plasma in plasma_data {
//     //     x2.push(plasma.time_tag);
//     //     y2.push(plasma.density);
//     //     y3.push(plasma.speed);
//     //     y4.push(plasma.temperature);
//     // }


//     let trace = plotly::Scatter::new(time_tags.clone(), bz_gsms)
//             .name("Bz")
//             .mode(Mode::Lines);
//     let trace2 = plotly::Scatter::new(time_tags.clone(), bt_gsms)
//             .name("Bt")
//             .mode(Mode::Lines);
//     let trace3 = plotly::Scatter::new(time_tags2.clone(), densities)
//             .name("Density")
//             .mode(Mode::Lines)
//             .x_axis("x2")
//             .y_axis("y2");
//     let trace4 = plotly::Scatter::new(time_tags2.clone(), speeds)
//             .name("Speed")
//             .mode(Mode::Lines)
//             .x_axis("x3")
//             .y_axis("y3");
//     let trace5 = plotly::Scatter::new(time_tags2.clone(), temperatures)
//             .name("Temperature")
//             .mode(Mode::Lines)
//             .x_axis("x4")
//             .y_axis("y4");

//     let layout = Layout::new()
//         .x_axis(plotly::layout::Axis::new().visible(false))
//         .y_axis(plotly::layout::Axis::new().range(vec![-40, 40]))
//         .x_axis2(plotly::layout::Axis::new().visible(false))
//         //.y_axis2(plotly::layout::Axis::new().range(vec![0, 50]))
//         .x_axis3(plotly::layout::Axis::new().visible(false))
//         //.y_axis3(plotly::layout::Axis::new().range(vec![0, 1000]))
//         //.x_axis4(plotly::layout::Axis::new().visible(false))
//         //.y_axis4(plotly::layout::Axis::new().range(vec![0, 10000000]))
//         .paper_background_color(plotly::color::NamedColor::Black)
//         .plot_background_color(plotly::color::NamedColor::Black)
//         //.title("Magnetic Field".into())
//         .show_legend(false)
//         .height(800)
//         .grid(
//             plotly::layout::LayoutGrid::new()
//                 .rows(4)
//                 .columns(1)
//                 .pattern(plotly::layout::GridPattern::Independent),
//         );
//     let config = plotly::configuration::Configuration::new()
//         .display_mode_bar(plotly::configuration::DisplayModeBar::False);

//     let mut plot: plotly::Plot = plotly::Plot::new();
//     plot.add_trace(trace);
//     plot.add_trace(trace2);
//     plot.add_trace(trace3);
//     plot.add_trace(trace4);
//     plot.add_trace(trace5);
//     plot.set_layout(layout);
//     plot.set_configuration(config);
    
//     //plot.write_html("src/out.html");
//     plot.to_inline_html(Some("div2"))
// }