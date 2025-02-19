use crate::structs::*;
use crate::auth::AuthenticatedUser;
use crate::role::Role;
use rocket::{post, serde::json::Json, State};
use neo4rs::{query, Node};

#[post("/api/set_schedule", format = "json", data = "<schedule_request>")]
pub async fn set_schedule(
    schedule_request: Json<SetScheduleRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Vec<TrailerSchedule>>, Json<&'static str>> {
    if role.0 != "write" || role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (trailer:Trailer)-[:HAS_SCHEDULE]->(s:Schedule)
        WHERE trailer.id = $TrailerID
        SET s.ScheduleDate = $ScheduleDate,
            s.RequestDate = $RequestDate,
            s.CarrierCode = $CarrierCode,
            s.ScheduleTime = $ScheduleTime,
            s.LastFreeDate = $LastFreeDate,
            s.ContactEmail = $ContactEmail,
            s.DoorNumber = $Door
        RETURN trailer.id as TrailerID, s
    ")
    .param("TrailerID", schedule_request.TrailerID.clone())
    .param("ScheduleDate", schedule_request.ScheduleDate.clone())
    .param("RequestDate", schedule_request.RequestDate.clone())
    .param("CarrierCode", schedule_request.CarrierCode.clone())
    .param("ScheduleTime", schedule_request.ScheduleTime.clone())
    .param("LastFreeDate", schedule_request.LastFreeDate.clone())
    .param("ContactEmail", schedule_request.ContactEmail.clone())
    .param("Door", schedule_request.Door.clone());

    match graph.execute(query).await {
        Ok(mut result) => {
            let mut data: Vec<TrailerSchedule> = Vec::new();
            while let Ok(Some(record)) = result.next().await {

                let trailer_id: String = record.get("TrailerID").unwrap();
                let schedule_node: Node = record.get("s").unwrap();
                let schedule_date: String = schedule_node.get("ScheduleDate").unwrap_or("".to_string());
                let schedule_time: String = schedule_node.get("ScheduleTime").unwrap_or("".to_string());
                let arrival_time: String = schedule_node.get("ArrivalTime").unwrap_or("".to_string());
                let carrier_code: String = schedule_node.get("CarrierCode").unwrap_or("".to_string());
                let contact_email: String = schedule_node.get("ContactEmail").unwrap_or("".to_string());
                let door_number: String = schedule_node.get("DoorNumber").unwrap_or("".to_string());
                let is_hot: bool = schedule_node.get("IsHot").unwrap_or(false);
                let last_free_date: String = schedule_node.get("LastFreeDate").unwrap_or("".to_string());
                let load_status: String = schedule_node.get("LoadStatus").unwrap_or("".to_string());
                let request_date: String = schedule_node.get("RequestDate").unwrap_or("".to_string());
                let schedule_data = TrailerSchedule {
                    TrailerID: trailer_id,
                    Schedule: Schedule {
                        ScheduleDate: schedule_date,
                        ScheduleTime: schedule_time,
                        ArrivalTime: arrival_time,
                        CarrierCode: carrier_code,
                        ContactEmail: contact_email,	
                        DoorNumber: door_number,
                        IsHot: is_hot,
                        LastFreeDate: last_free_date,
                        LoadStatus: load_status,
                        RequestDate: request_date,
                    },
                };
                data.push(schedule_data);
            }

            Ok(Json(data))
        },
        Err(e) => {
            println!("Failed to run query: {:?}", e);
            Err(Json("Internal Server Error"))
        }
    }
}

#[post("/api/new_shipment", format = "json", data = "<new_shipment>")]
pub async fn new_shipment(
    new_shipment: Json<Shipment>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" || role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MERGE (s:Shipment {LoadId: $LoadId})
        SET s.ScheduleDate = $ScheduleDate,
            s.ScheduleTime = $ScheduleTime,
            s.ArrivalTime = '',
            s.DepartTime = '',
            s.Dock = $Dock,
            s.Door = $Door,
            s.Status = 'Not Started',
            s.LoadId = $LoadId,
            s.Picker = '',
            s.PickStartTime = '',
            s.VerifiedBy = '',
            s.TrailerNum = $TrailerNum
        RETURN s
    ")
    .param("ScheduleDate", new_shipment.ScheduleDate.clone())
    .param("ScheduleTime", new_shipment.ScheduleTime.clone())
    .param("Dock", new_shipment.Dock.clone())
    .param("LoadId", new_shipment.LoadId.clone())
    .param("LoadNum", new_shipment.LoadNum.clone())
    .param("TrailerNum", new_shipment.TrailerNum.clone())
    .param("Door", new_shipment.Door.clone());

    match graph.execute(query).await {
        Ok(mut result) => {
            if let Ok(Some(record)) = result.next().await {

                let shipment_node: Node = record.get("s").unwrap();
                let schedule_date: String = shipment_node.get("ScheduleDate").unwrap_or("".to_string());
                let schedule_time: String = shipment_node.get("ScheduleTime").unwrap_or("".to_string());
                let arrival_time: String = shipment_node.get("ArrivalTime").unwrap_or("".to_string());
                let depart_time: String = shipment_node.get("DepartTime").unwrap_or("".to_string());
                let dock: String = shipment_node.get("Dock").unwrap_or("".to_string());
                let door: String = shipment_node.get("Door").unwrap_or("".to_string());
                let load_id: String = shipment_node.get("LoadId").unwrap_or("".to_string());
                let load_num: String = shipment_node.get("LoadNum").unwrap_or("".to_string());
                let status: String = shipment_node.get("Status").unwrap_or("".to_string());
                let picker: String = shipment_node.get("Picker").unwrap_or("".to_string());
                let pick_start_time: String = shipment_node.get("PickStartTime").unwrap_or("".to_string());
                let verified_by: String = shipment_node.get("VerifiedBy").unwrap_or("".to_string());
                let trailer_num: String = shipment_node.get("TrailerNum").unwrap_or("".to_string());
                let shipment = Shipment {
                        ScheduleDate: schedule_date,
                        ScheduleTime: schedule_time,
                        ArrivalTime: arrival_time,
                        DepartTime: depart_time,
                        Dock: dock,	
                        Door: door,
                        LoadId: load_id,
                        LoadNum: load_num,
                        Status: status,
                        Picker: picker,
                        PickStartTime: pick_start_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                    };
                    Ok(Json(shipment))
            } else {
                Err(Json("No record found"))
            }
        },
        Err(e) => {
            println!("Failed to run query: {:?}", e);
            Err(Json("Internal Server Error"))
        }
    }
}

#[post("/api/hot_trailer", format = "json", data = "<hot_trailer_request>")]
pub async fn hot_trailer(
    hot_trailer_request: Json<HotTrailerRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Vec<TrailerSchedule>>, Json<&'static str>> {
    if role.0 != "write" || role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;
    println!("{:?}", hot_trailer_request);

    let query = query("
        MATCH (trailer:Trailer)-[:HAS_SCHEDULE]->(s:Schedule)
        WHERE trailer.id = $TrailerID
        SET s.IsHot = NOT s.IsHot  
        RETURN trailer.id as TrailerID, s
    ").param("TrailerID", hot_trailer_request.TrailerID.clone());

    match graph.execute(query).await {
        Ok(mut result) => {
            let mut data: Vec<TrailerSchedule> = Vec::new();
            while let Ok(Some(record)) = result.next().await {

                let trailer_id: String = record.get("TrailerID").unwrap();
                let schedule_node: Node = record.get("s").unwrap();
                let schedule_date: String = schedule_node.get("ScheduleDate").unwrap_or("".to_string());
                let schedule_time: String = schedule_node.get("ScheduleTime").unwrap_or("".to_string());
                let arrival_time: String = schedule_node.get("ArrivalTime").unwrap_or("".to_string());
                let carrier_code: String = schedule_node.get("CarrierCode").unwrap_or("".to_string());
                let contact_email: String = schedule_node.get("ContactEmail").unwrap_or("".to_string());
                let door_number: String = schedule_node.get("DoorNumber").unwrap_or("".to_string());
                let is_hot: bool = schedule_node.get("IsHot").unwrap_or(false);
                let last_free_date: String = schedule_node.get("LastFreeDate").unwrap_or("".to_string());;
                let load_status: String = schedule_node.get("LoadStatus").unwrap_or("".to_string());
                let request_date: String = schedule_node.get("RequestDate").unwrap_or("".to_string());
                let schedule_data = TrailerSchedule {
                    TrailerID: trailer_id,
                    Schedule: Schedule {
                        ScheduleDate: schedule_date,
                        ScheduleTime: schedule_time,
                        ArrivalTime: arrival_time,
                        CarrierCode: carrier_code,
                        ContactEmail: contact_email,	
                        DoorNumber: door_number,
                        IsHot: is_hot,
                        LastFreeDate: last_free_date,
                        LoadStatus: load_status,
                        RequestDate: request_date,
                    },
                };
                data.push(schedule_data);
            }

            Ok(Json(data))
        },
        Err(e) => {
            println!("Failed to run query: {:?}", e);
            Err(Json("Internal Server Error"))
        }
    }
}

#[post("/api/set_door", format = "json", data = "<set_door_request>")]
pub async fn set_door(
    set_door_request: Json<SetDoorRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Vec<TrailerSchedule>>, Json<&'static str>> {
    if role.0 != "write" || role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;
    println!("{:?}", set_door_request);

    let query = query("
        MATCH (trailer:Trailer)-[:HAS_SCHEDULE]->(s:Schedule)
        WHERE trailer.id = $TrailerID
        SET s.DoorNumber = $Door
        RETURN trailer.id as TrailerID, s
    ")
    .param("TrailerID", set_door_request.TrailerID.clone())
    .param("Door", set_door_request.Door.clone());

    match graph.execute(query).await {
        Ok(mut result) => {
            let mut data: Vec<TrailerSchedule> = Vec::new();
            while let Ok(Some(record)) = result.next().await {

                let trailer_id: String = record.get("TrailerID").unwrap();
                let schedule_node: Node = record.get("s").unwrap();
                let schedule_date: String = schedule_node.get("ScheduleDate").unwrap_or("".to_string());
                let schedule_time: String = schedule_node.get("ScheduleTime").unwrap_or("".to_string());
                let arrival_time: String = schedule_node.get("ArrivalTime").unwrap_or("".to_string());
                let carrier_code: String = schedule_node.get("CarrierCode").unwrap_or("".to_string());
                let contact_email: String = schedule_node.get("ContactEmail").unwrap_or("".to_string());
                let door_number: String = schedule_node.get("DoorNumber").unwrap_or("".to_string());
                let is_hot: bool = schedule_node.get("IsHot").unwrap_or(false);
                let last_free_date: String = schedule_node.get("LastFreeDate").unwrap_or("".to_string());
                let load_status: String = schedule_node.get("LoadStatus").unwrap_or("".to_string());
                let request_date: String = schedule_node.get("RequestDate").unwrap_or("".to_string());
                let schedule_data = TrailerSchedule {
                    TrailerID: trailer_id,
                    Schedule: Schedule {
                        ScheduleDate: schedule_date,
                        ScheduleTime: schedule_time,
                        ArrivalTime: arrival_time,
                        CarrierCode: carrier_code,
                        ContactEmail: contact_email,	
                        DoorNumber: door_number,
                        IsHot: is_hot,
                        LastFreeDate: last_free_date,
                        LoadStatus: load_status,
                        RequestDate: request_date,
                    },
                };
                data.push(schedule_data);
            }

            Ok(Json(data))
        },
        Err(e) => {
            println!("Failed to run query: {:?}", e);
            Err(Json("Internal Server Error"))
        }
    }
}

#[post("/api/set_arrivalTime", format = "json", data = "<set_arrival_time_request>")]
pub async fn set_arrival_time(
    set_arrival_time_request: Json<SetArrivalTimeRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Vec<TrailerSchedule>>, Json<&'static str>> {
    if role.0 != "write" || role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;
    println!("{:?}", set_arrival_time_request);

    let load_status = if set_arrival_time_request.ArrivalTime.is_empty() {
        "in-transit".to_string()
    } else {
        "arrived".to_string()
    };

    let query = query("
        MATCH (trailer:Trailer)-[:HAS_SCHEDULE]->(s:Schedule)
        WHERE trailer.id = $TrailerID
        SET s.ArrivalTime = $ArrivalTime
        SET s.LoadStatue = $load_status
        RETURN trailer.id as TrailerID, s
    ")
    .param("TrailerID", set_arrival_time_request.TrailerID.clone())
    .param("ArrivalTime", set_arrival_time_request.ArrivalTime.clone())
    .param("load_status", load_status);

    match graph.execute(query).await {
        Ok(mut result) => {
            let mut data: Vec<TrailerSchedule> = Vec::new();
            while let Ok(Some(record)) = result.next().await {

                let trailer_id: String = record.get("TrailerID").unwrap();
                let schedule_node: Node = record.get("s").unwrap();
                let schedule_date: String = schedule_node.get("ScheduleDate").unwrap_or("".to_string());
                let schedule_time: String = schedule_node.get("ScheduleTime").unwrap_or("".to_string());
                let arrival_time: String = schedule_node.get("ArrivalTime").unwrap_or("".to_string());
                let carrier_code: String = schedule_node.get("CarrierCode").unwrap_or("".to_string());
                let contact_email: String = schedule_node.get("ContactEmail").unwrap_or("".to_string());
                let door_number: String = schedule_node.get("DoorNumber").unwrap_or("".to_string());
                let is_hot: bool = schedule_node.get("IsHot").unwrap_or(false);
                let last_free_date: String = schedule_node.get("LastFreeDate").unwrap_or("".to_string());
                let load_status: String = schedule_node.get("LoadStatus").unwrap_or("".to_string());
                let request_date: String = schedule_node.get("RequestDate").unwrap_or("".to_string());
                let schedule_data = TrailerSchedule {
                    TrailerID: trailer_id,
                    Schedule: Schedule {
                        ScheduleDate: schedule_date,
                        ScheduleTime: schedule_time,
                        ArrivalTime: arrival_time,
                        CarrierCode: carrier_code,
                        ContactEmail: contact_email,	
                        DoorNumber: door_number,
                        IsHot: is_hot,
                        LastFreeDate: last_free_date,
                        LoadStatus: load_status,
                        RequestDate: request_date,
                    },
                };

                data.push(schedule_data);
            }

            Ok(Json(data))
        },
        Err(e) => {
            println!("Failed to run query: {:?}", e);
            Err(Json("Internal Server Error"))
        }
    }
}