use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use tokio_tungstenite::tungstenite::protocol::Message;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use neo4rs::Graph;


pub type WebSocketList = Arc<Mutex<HashMap<SocketAddr, UnboundedSender<Message>>>>;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoadCountRequest {
    pub prefix: String,
}

#[derive(Deserialize)]
pub struct TodaysTrucksRequest {
    pub date: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
    pub refresh_token: Option<String>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub username: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub password: String,
    pub role: String
}

#[derive(Deserialize)]
pub struct LoadInfoRequest {
    pub param: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Part {
    pub partNumber: String,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Trailer {
    pub TrailerID: String,
    pub Schedule: Schedule,
    pub CiscoIDs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Sids {
    pub TrailerID: String,
    pub Sids: Vec<SidAndParts>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidAndParts {
    pub Sid: String,
    pub Cisco: String,
    pub Part: String,
    pub Quantity: i32,
}

#[derive(Deserialize)]
pub struct DateRangeTruckRequest {
    pub date1: String,
    pub date2: String,
}

#[derive(Serialize)]
pub struct TrailerResponse {
    pub TrailerID: String,
    pub Sids: Vec<SidAndParts>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SidParts {
    pub Sid: Sid,
    pub Parts: Vec<Part>,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct Count {
    pub item: String,
    pub location: String,
    pub actual: u32,
    pub expected: u32,
    pub actual_lp_count: u32,
    pub expected_lp_count: u32,
    pub comment: String,
    pub date: String,
}

#[derive(Serialize)]
pub struct CountSummary {
    pub part_number: String,
    pub num_locations: u32,
    pub actual: u32,
    pub expected: u32,
    pub actual_lp_count: u32,
    pub expected_lp_count: u32,
    pub date: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IncomingMessage {
    pub r#type: String,
    pub data: MessageData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageData {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct SetScheduleRequest {
    pub TrailerID: String,
    pub ScheduleDate: String,
    pub RequestDate: String,
    pub CarrierCode: String,
    pub ScheduleTime: String,
    pub LastFreeDate: String,
    pub ContactEmail: String,
    pub Door: String,
    pub ClaimComments: String,
    pub Seal: String,
}

#[derive(Deserialize, Debug)]
pub struct SetArrivalTimeRequest {
    pub TrailerID: String,
    pub ArrivalTime: String,
}

#[derive(Serialize, Debug)]
pub struct TrailerSchedule {
    pub TrailerID: String,
    pub Schedule: Schedule,
}

#[derive(Deserialize, Debug)]
pub struct SetDoorRequest {
    pub TrailerID: String,
    pub Door: String,
}

#[derive(Deserialize, Debug)]
pub struct HotTrailerRequest {
    pub TrailerID: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Schedule {
    pub ScheduleDate: String,
    pub ScheduleTime: String,
    pub ArrivalTime: String,
    pub CarrierCode: String,
    pub ContactEmail: String,	
    pub DoorNumber: String,
    pub IsHot: bool,
    pub LastFreeDate: String,
    pub LoadStatus: String,
    pub RequestDate: String,
    pub Seal: String,
    pub IsMulti: bool,
    pub IsStat6: bool,
    pub ClaimComments: String,
    pub HasClaim: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Shipment {
    pub ScheduleDate: String,
    pub ScheduleTime: String,
    pub ArrivalTime: String,
    pub DepartTime: String,
    pub Dock: String,
    pub Door: String,
    pub LoadId: String,
    pub LoadNum: String,
    pub Status: String,
    pub Picker: String,
    pub PickStartTime: String,
    pub PickFinishTime: String,
    pub VerifiedBy: String,
    pub TrailerNum: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ShipmentArrivalTimeRequest {
    pub ArrivalTime: String,
    pub LoadId: String,
    pub TrailerNum: String,
}

#[derive(Deserialize)]
pub struct ShipmentDepartTimeRequest {
    pub DepartTime: String,
    pub LoadId: String,
}

#[derive(Deserialize)]
pub struct VerifiedByRequest {
    pub VerifiedBy: String,
    pub LoadId: String,
}

#[derive(Deserialize)]
pub struct ShipmentPickFinishRequest {
    pub LoadId: String,
    pub FinishTime: String,
}

#[derive(Deserialize)]
pub struct ShipmentDoor {
    pub LoadId: String,
    pub Door: String,
}

#[derive(Deserialize)]
pub struct ShipmentBeginLoading {
    pub LoadId: String,
}

#[derive(Deserialize)]
pub struct PickStartRequest {
    pub StartTime: String,
    pub LoadId: String,
    pub Picker: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sid {
    pub CiscoID: String,
    pub id: String,
}

pub struct AppState {
    pub graph: Arc<Graph>,
    pub jwt_secret: String,
    pub ws_list: WebSocketList,
}

#[derive(Deserialize)]
pub struct SidsRequest {
    pub date: String,
}