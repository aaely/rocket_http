extern crate rocket;

mod auth;
mod role;
mod structs;
mod getters;
mod loginroutes;
mod setters;
mod wsserver;

use rocket::routes;
use neo4rs::Graph;
use structs::AppState;
use tokio::sync::Mutex;
use std::{collections::HashMap, sync::Arc};
use rocket_cors::{CorsOptions, AllowedHeaders};
use getters::*;
use loginroutes::*;
use setters::*;
use wsserver::*;


/*
    CORS Config
*/

fn custom_cors() -> rocket_cors::Cors {
    CorsOptions::default()
        .allowed_origins(rocket_cors::AllOrSome::All)
        .allowed_headers(AllowedHeaders::some(&["Authorization", "Accept", "Content-Type"]))
        .allow_credentials(true)
        .to_cors()
        .expect("error creating CORS fairing")
}

impl AppState {
    pub async fn new() -> Self {
        let graph = Graph::new("bolt://localhost:7687", "neo4j", "Asdf123$").await.unwrap();

        AppState {
            ws_list: Arc::new(Mutex::new(HashMap::new())),
            graph: Arc::new(graph),
            jwt_secret: "tO7E8uCjD5rXpQl0FhKwV2yMz4bJnAi9sGeR3kTzXvNmPuLsDq8W".to_string(),
        }
    }
}

#[rocket::main]
async fn main() {
    let state = AppState::new().await;


    // Configure CORS
    let cors = custom_cors();

    rocket::custom(
        rocket::Config {
            address: "127.0.0.1".parse().expect("Invalid IP address"),
            port: 8000,
            ..rocket::Config::default()
        }
    )
        .attach(cors)
        .mount("/", routes![get_shipments, shipment_status_update, shipment_door, new_shipment, set_shipment_arrival_time, set_shipment_departure_time, set_shipment_pick_start, get_counts, todays_trucks, get_load_count, date_range_trucks, set_arrival_time, set_door, hot_trailer, set_schedule, get_load_info, trailers, ws_handler, refresh_token, login, schedule_trailer, register])
        .manage(state)
        .launch()
        .await
        .unwrap();
}