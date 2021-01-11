use std::collections::HashMap;
use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json::Value;

// Verified Credential VC01's Format
#[derive(Deserialize)]
struct Vc01In04 {
    name: String,
    age: i32,
    job: String,
}

#[derive(Deserialize)]
struct Vc01In03 {
    profile: Vc01In04,
}

#[derive(Deserialize)]
struct Vc01In02 {
    credentialSubject: Vc01In03,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[derive(Deserialize)]
struct Vc01In01 {
    sub: String,
    jti: String,
    iss: String,
    nbf: u32,
    iat: u32,
    exp: u32,
    nonce: String,
    vc: Vc01In02,
}

// Output Json Format of "json" path
#[derive(Serialize)]
struct OutputJson {
    sub: String,
    age: i32,
    adid: String,
}

#[get("/")]
async fn top() -> impl Responder {
    HttpResponse::Ok().body("Hello Top Page!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[post("/json")]
async fn json(vc01_in01: web::Json<Vc01In01>) -> impl Responder {
    web::Json(OutputJson { adid: "testid".to_string(), sub: vc01_in01.sub.clone(), age: vc01_in01.vc.credentialSubject.profile.age})
}

#[get("/{id}/{name}")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let addr_leader: &str = "0.0.0.0:8080";
    let addr: &str = "0.0.0.0:8081";

    if let Some(mode) = std::env::args().nth(1) {
        println!("Running \"leader mode\"@port8080");

        if mode == "leader" {
            HttpServer::new(||
                            App::new()
                            .service(top)
                            .service(index)
                            )
                .bind(addr_leader)?
                .run()
                .await
        } else {
            println!("Running \"default mode\"@port8081. (arg = {})", mode);
            HttpServer::new(||
                            App::new()
                            .service(top)
                            .service(echo)
                            .service(json)
                            )
                .bind(addr)?
                .run()
                .await
        }

    } else {
        println!("No arg. Running \"default mode\"@port8081");
        HttpServer::new(||
                        App::new()
                        .service(top)
                        .service(echo)
                        .service(json)
                        )
            .bind(addr)?
            .run()
            .await
    }

}
