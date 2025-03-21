use crate::structs::*;
use crate::auth::AuthenticatedUser;
use crate::role::Role;
use futures_util::TryFutureExt;
use rocket::{post, serde::json::Json, State};
use neo4rs::{query, Node};

#[post("/api/set_schedule", format = "json", data = "<schedule_request>")]
pub async fn set_schedule(
    schedule_request: Json<SetScheduleRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Vec<TrailerSchedule>>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
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
            s.DoorNumber = $Door,
            s.Seal = $Seal
        RETURN trailer.id as TrailerID, s
    ")
    .param("TrailerID", schedule_request.TrailerID.clone())
    .param("ClaimComments", schedule_request.ClaimComments.clone())
    .param("ScheduleDate", schedule_request.ScheduleDate.clone())
    .param("RequestDate", schedule_request.RequestDate.clone())
    .param("CarrierCode", schedule_request.CarrierCode.clone())
    .param("ScheduleTime", schedule_request.ScheduleTime.clone())
    .param("LastFreeDate", schedule_request.LastFreeDate.clone())
    .param("ContactEmail", schedule_request.ContactEmail.clone())
    .param("Seal", schedule_request.Seal.clone())
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
                let carrier_claim: String = schedule_node.get("ClaimComments").unwrap();
                let has_claim: bool = schedule_node.get("HasClaim").unwrap();
                let is_stat6: bool = schedule_node.get("IsStat6").unwrap();
                let seal: String = schedule_node.get("Seal").unwrap();
                let is_multi: bool = schedule_node.get("IsMulti").unwrap_or(false);
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
                        ClaimComments: carrier_claim,
                        IsStat6: is_stat6,
                        IsMulti: is_multi,
                        Seal: seal,
                        HasClaim: has_claim,
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

#[post("/api/delete_shipment", format = "json", data = "<delete_shipment>")]
pub async fn delete_shipment(
    delete_shipment: Json<DeleteShipmentRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<(), Json<&'static str>> {
    if role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (s:Shipment {LoadId: $LoadId})
        DETACH DELETE s
    ").param("LoadId", delete_shipment.LoadId.clone());

    match graph.execute(query).await {
        Ok(_) => Ok(()),
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
    if role.0 != "write" && role.0 != "admin" {
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
            s.Status = 'NOT STARTED',
            s.LoadId = $LoadId,
            s.Picker = '',
            s.PickStartTime = '',
            s.VerifiedBy = '',
            s.LoadNum = $LoadNum,
            s.TrailerNum = ''
        RETURN s
    ")
    .param("ScheduleDate", new_shipment.ScheduleDate.clone())
    .param("ScheduleTime", new_shipment.ScheduleTime.clone())
    .param("Dock", new_shipment.Dock.clone())
    .param("LoadId", new_shipment.LoadId.clone())
    .param("LoadNum", new_shipment.LoadNum.clone())
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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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

#[post("/api/shipment_door", format = "json", data = "<shipment_door>")]
pub async fn shipment_door(
    shipment_door: Json<ShipmentDoor>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MERGE (s:Shipment {LoadId: $LoadId})
        SET s.Door = $Door
        RETURN s
    ")
    .param("LoadId", shipment_door.LoadId.clone())
    .param("Door", shipment_door.Door.clone());

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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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
    if role.0 != "write" && role.0 != "admin" {
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
                let carrier_claim: String = schedule_node.get("ClaimComments").unwrap();
                let has_claim: bool = schedule_node.get("HasClaim").unwrap();
                let is_stat6: bool = schedule_node.get("IsStat6").unwrap();
                let seal: String = schedule_node.get("Seal").unwrap();
                let is_multi: bool = schedule_node.get("IsMulti").unwrap_or(false);
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
                        ClaimComments: carrier_claim,
                        HasClaim: has_claim,
                        IsMulti: is_multi,
                        IsStat6: is_stat6,
                        Seal: seal,
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
    if role.0 != "write" && role.0 != "admin" {
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
                let carrier_claim: String = schedule_node.get("ClaimComments").unwrap();
                let has_claim: bool = schedule_node.get("HasClaim").unwrap();
                let is_stat6: bool = schedule_node.get("IsStat6").unwrap();
                let seal: String = schedule_node.get("Seal").unwrap();
                let is_multi: bool = schedule_node.get("IsMulti").unwrap_or(false);
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
                        ClaimComments: carrier_claim,
                        HasClaim: has_claim,
                        IsMulti: is_multi,
                        IsStat6: is_stat6,
                        Seal: seal,
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
    if role.0 != "write" && role.0 != "admin" {
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
                let carrier_claim: String = schedule_node.get("ClaimComments").unwrap();
                let has_claim: bool = schedule_node.get("HasClaim").unwrap();
                let is_stat6: bool = schedule_node.get("IsStat6").unwrap();
                let seal: String = schedule_node.get("Seal").unwrap();
                let is_multi: bool = schedule_node.get("IsMulti").unwrap_or(false);
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
                        ClaimComments: carrier_claim,
                        HasClaim: has_claim,
                        IsMulti: is_multi,
                        IsStat6: is_stat6,
                        Seal: seal,
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

#[post("/api/set_shipment_trailer", format = "json", data = "<set_shipment_arrival_time>")]
pub async fn set_shipment_trailer(
    set_shipment_arrival_time: Json<ShipmentArrivalTimeRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (s:Shipment {LoadId: $LoadId})
        SET s.ArrivalTime = $ArrivalTime,
            s.TrailerNum = $TrailerNum
        RETURN s
    ")
    .param("LoadId", set_shipment_arrival_time.LoadId.clone())
    .param("TrailerNum", set_shipment_arrival_time.TrailerNum.clone())
    .param("ArrivalTime", set_shipment_arrival_time.ArrivalTime.clone());

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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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

#[post("/api/set_shipment_departureTime", format = "json", data = "<set_shipment_departure_time>")]
pub async fn set_shipment_departure_time(
    set_shipment_departure_time: Json<ShipmentDepartTimeRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (s:Shipment {LoadId: $LoadId})
        SET s.DepartTime = $DepartTime,
            s.Status = 'COMPLETE',
            s.Seal = $Seal
        RETURN s
    ")
    .param("LoadId", set_shipment_departure_time.LoadId.clone())
    .param("Seal", set_shipment_departure_time.Seal.clone())
    .param("DepartTime", set_shipment_departure_time.DepartTime.clone());

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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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

#[post("/api/set_shipment_pick_start", format = "json", data = "<set_shipment_pick_start>")]
pub async fn set_shipment_pick_start(
    set_shipment_pick_start: Json<PickStartRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (s:Shipment {LoadId: $LoadId})
        SET s.Status = 'PICKING',
            s.Picker = $Picker,
            s.PickStartTime = $PickStartTime
        RETURN s
    ")
    .param("LoadId", set_shipment_pick_start.LoadId.clone())
    .param("PickStartTime", set_shipment_pick_start.StartTime.clone())
    .param("Picker", set_shipment_pick_start.Picker.clone());

    match graph.execute(query).await {
        Ok(mut result) => {
            let mut data: Vec<Shipment> = Vec::new();
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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let verified_by: String = shipment_node.get("VerifiedBy").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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

#[post("/api/shipment_pick_finish", format = "json", data = "<shipment_pick_finish>")]
pub async fn shipment_pick_finish(
    shipment_pick_finish: Json<ShipmentPickFinishRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (s:Shipment {LoadId: $LoadId})
        SET s.Status = 'VERIFICATION',
            s.PickFinishTime = $FinishTime
        RETURN s
    ")
    .param("LoadId", shipment_pick_finish.LoadId.clone())
    .param("FinishTime", shipment_pick_finish.FinishTime.clone());

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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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

#[post("/api/shipment_verification", format = "json", data = "<shipment_verification>")]
pub async fn shipment_verification(
    shipment_verification: Json<VerifiedByRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (s:Shipment {LoadId: $LoadId})
        SET s.Status = 'READY TO LOAD',
            s.VerifiedBy = $VerifiedBy
        RETURN s
    ")
    .param("LoadId", shipment_verification.LoadId.clone())
    .param("VerifiedBy", shipment_verification.VerifiedBy.clone());

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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let trailer_num: String = shipment_node.get("TrailerNum").unwrap_or("".to_string());
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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

#[post("/api/shipment_begin_loading", format = "json", data = "<shipment_begin_loading>")]
pub async fn shipment_begin_loading(
    shipment_begin_loading: Json<ShipmentBeginLoading>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (s:Shipment {LoadId: $LoadId})
        SET s.Status = 'LOADING'
        RETURN s
    ")
    .param("LoadId", shipment_begin_loading.LoadId.clone());

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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let verified_by: String = shipment_node.get("VerifiedBy").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let trailer_num: String = shipment_node.get("TrailerNum").unwrap_or("".to_string());
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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

#[post("/api/shipment_hold", format = "json", data = "<shipment_hold>")]
pub async fn shipment_hold(
    shipment_hold: Json<ShipmentBeginLoading>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Shipment>, Json<&'static str>> {
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;

    let query = query("
        MATCH (s:Shipment {LoadId: $LoadId})
        SET s.IsHold = NOT s.IsHold
        RETURN s
    ")
    .param("LoadId", shipment_hold.LoadId.clone());

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
                let pick_finish_time: String = shipment_node.get("PickFinishTime").unwrap_or("".to_string());
                let verified_by: String = shipment_node.get("VerifiedBy").unwrap_or("".to_string());
                let trailer_num: String = shipment_node.get("TrailerNum").unwrap_or("".to_string());
                let is_hold: bool = shipment_node.get("IsHold").unwrap_or(false);
                let seal: String = shipment_node.get("Seal").unwrap_or("".to_string());
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
                        PickFinishTime: pick_finish_time,
                        VerifiedBy: verified_by,
                        TrailerNum: trailer_num,
                        IsHold: is_hold,
                        Seal: seal,
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

#[post("/api/shipment_lines", format = "json", data = "<shipment_lines>")]
pub async fn shipment_lines(
    shipment_lines: Json<ShipmentLinesRequest>,
    state: &State<AppState>,
    _user: AuthenticatedUser,
    role: Role,
) -> Result<Json<Vec<ShipmentLine>>, Json<&'static str>> {
    
    if role.0 != "write" && role.0 != "admin" {
        return Err(Json("Forbidden"));
    }

    let graph = &state.graph;
    let mut lines = shipment_lines.Lines.clone();
    // Retain only lines with non-zero quantity.
    lines.retain(|line| line.quantity != 0);
    println!("{:?}", shipment_lines.clone());

    // Delete existing shipment lines.
    let delete_query = query("
        MATCH (s:Shipment {LoadId: $LoadId})-[:HAS_LINE]->(sl:ShipmentLine)
        WITH sl
        DETACH DELETE sl
        RETURN COUNT(sl) as count
    ")
    .param("LoadId", shipment_lines.LoadId.clone());

    match graph.execute(delete_query).await {
        Ok(mut result) => {
            if let Ok(Some(record)) = result.next().await {
                let count: u32 = record.get("count").unwrap_or(0);
                println!("{}",count);
            }
        },
        Err(e) => {
            println!("{:?}",e);
        }
    }
    // Collect created shipment lines.
    let mut created_lines = Vec::new();

    // For each new line, create it and return the created node.
    for line in lines {
        let create_query = query("
            MATCH (s:Shipment {LoadId: $LoadId})
            CREATE (s)-[:HAS_LINE]->(sl:ShipmentLine {
                PartNumber: $item,
                Quantity: $quantity,
                Ip: $ip
            })
            RETURN sl.PartNumber as item, sl.Quantity as quantity, sl.Ip as ip
        ")
        .param("item", line.item.clone())
        .param("quantity", line.quantity.clone())
        .param("LoadId", shipment_lines.LoadId.clone())
        .param("ip", line.ip.clone());

        match graph.execute(create_query).await {
            Ok(mut result) => {
                if let Ok(Some(record)) = result.next().await {
                    let item: String = record.get("item").unwrap_or("".to_string());
                    let quantity: u32 = record.get("quantity").unwrap();
                    let ip: String = record.get("ip").unwrap_or("".to_string());
                    let line = ShipmentLine {
                        item,
                        quantity,
                        ip,
                    };
                    created_lines.push(line);
                }
            },
            Err(e) => {
                println!("{:?}",e);
            }
        }
    }

    // Return the created shipment lines as JSON.
    Ok(Json(created_lines))
}