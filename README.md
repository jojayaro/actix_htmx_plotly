# Aurora Monitor

Aurora Monitor is a real-time solar wind data visualization tool built with Actix-web, HTMX, and Plotly. It provides an interactive web interface for monitoring space weather conditions that can lead to aurora events.

## Features

- Real-time solar wind data visualization
- Interactive plots for Bz, Bt, Density, Speed, and Temperature
- Customizable time ranges (2 hours, 24 hours, 7 days)
- Responsive design using Bootstrap
- Server-side rendering with Rust
- Client-side updates using HTMX

## Tech Stack

- **Backend**: Rust with Actix-web
- **Frontend**: HTML, CSS (Bootstrap), JavaScript (Plotly.js)
- **Data Handling**: Delta Lake, Apache Arrow
- **Interactivity**: HTMX for seamless updates

## Getting Started

1. Clone the repository
2. Install Rust and Cargo
4. Run the following commands:

```
cargo build
cargo run
```

5. Open your browser and navigate to `https://localhost:8080`

## Code Structure

The main application logic can be found in `src/main.rs`

This file contains the core functionality, including data fetching, processing, and serving the web interface.

## Frontend

The main interface is defined in `static/plot.html`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).

## Acknowledgements

- Data provided by [NOAA Space Weather Prediction Center](https://www.swpc.noaa.gov/)
- Plotly.js for interactive visualizations
- Bootstrap for responsive design

---

Feel free to customize this README further based on any additional information you'd like to include or emphasize!
