use std::collections::HashMap;
use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use rusqlite::{params, Connection, Result};

static mut db_conn: Option<rusqlite::Connection> = None;

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}

#[derive(Deserialize)]
struct SyncCredential {
    gid: String,
    pwd: String,
}

// Input (VC/Verified Credential) Formats
#[derive(Deserialize)]
struct VcCommon {
    sub: String,
    jti: String,
    iss: String,
    nbf: u32,
    iat: u32,
    exp: u32,
    nonce: String,
}

#[derive(Deserialize)]
struct VcProfile {
    #[serde(flatten)]
    common: VcCommon,

    vc: VcCredProfile,
}
#[derive(Deserialize)]
struct VcCredProfile {
    credentialSubject: Vc01PProfile,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
#[derive(Deserialize)]
struct Vc01PProfile {
    profile: Vc01Profile,
}
#[derive(Deserialize)]
struct Vc01Profile {
    name: String,
    age: u32,
    job: String,
    balanceYen: i32,
}

#[derive(Deserialize)]
struct VcCart {
    #[serde(flatten)]
    common: VcCommon,

    vc: VcCredCart,
}
#[derive(Deserialize)]
struct VcCredCart {
    credentialSubject: Vc02PCart,

    #[serde(flatten)]
    extra: HashMap<String, Value>,
}
#[derive(Deserialize)]
struct Vc02PCart {
    cart: Vc02Cart,
}
#[derive(Deserialize)]
struct Vc02Cart {
    productName: String,
    productPriceYen: i32,
    productNumber: i32,
}

#[derive(Deserialize)]
struct RegProfReq {
    gid: String,
    json_base64: String,
}
#[derive(Deserialize)]
struct RegCartReq {
    gid: String,
    json_base64: String,
}


// Output Json Formats (to be VC as well)
#[derive(Serialize)]
struct OutputRegVC {
    gid: String,
    sub: String,
    name: String,
    age: u32,
    job: String,
    balanceYen: i32,
}

#[derive(Serialize)]
struct OutputReceiptVC {
    gid: String,
    sub: String,
    ProductName: String,
    productPriceYen: i32,
    productNumber: i32,
}

#[derive(Serialize)]
struct OutputSync {
    gid: String,
    amount: i32,
    balanceYen: i32,
    adid: String,
}

#[post("/register_profile")]
async fn register_profile(req: web::Form<RegProfReq>) -> impl Responder {
    let vc_prof: VcProfile = serde_json::from_str(&req.json_base64).unwrap();

    unsafe {
        if let Some(conn) = &db_conn {
            conn.execute(
                "INSERT INTO sharedb
                    (pwd, sub, name, age, job, balanceYen,
                     productName, productPriceYen, productNumber)
                VALUES
                    (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    vc_prof.common.sub.clone(),
                    Some("".to_string()),
                    Some(vc_prof.vc.credentialSubject.profile.name.clone()),
                    Some(vc_prof.vc.credentialSubject.profile.age),
                    Some(vc_prof.vc.credentialSubject.profile.job.clone()),
                    Some(vc_prof.vc.credentialSubject.profile.balanceYen),
                    Some("".to_string()),
                    Some("".to_string()),
                    Some(0)
                ]
            ).unwrap();
        }
    }

    web::Json(
        OutputRegVC {
            gid: req.gid.clone(),
            sub: vc_prof.common.sub.clone(),
            name: vc_prof.vc.credentialSubject.profile.name.clone(),
            age: vc_prof.vc.credentialSubject.profile.age,
            job: vc_prof.vc.credentialSubject.profile.job.clone(),
            balanceYen: vc_prof.vc.credentialSubject.profile.balanceYen,
        }
    )
}

#[post("/register_cart")]
async fn register_cart(req: web::Form<RegCartReq>) -> impl Responder {
    let vc_cart: VcCart = serde_json::from_str(&req.json_base64).unwrap();

    unsafe {
    }

    web::Json(
        OutputReceiptVC {
            gid: req.gid.clone(),
            sub: vc_cart.common.sub.clone(),
            ProductName: vc_cart.vc.credentialSubject.cart.productName.to_string(),
            productPriceYen: vc_cart.vc.credentialSubject.cart.productPriceYen,
            productNumber: vc_cart.vc.credentialSubject.cart.productNumber,
        }
    )
}

#[post("/sync")]
async fn sync(cred: web::Form<SyncCredential>) -> impl Responder {
    if cred.pwd == "testpwd" {
        // todo: query

        web::Json(
            OutputSync {
                gid: cred.gid.clone(),
                amount: 9000,
                balanceYen: 1000,
                adid: "wine_ad".to_string(),
            }
        )
    } else {
        web::Json(
            OutputSync {
                gid: "".to_string(),
                amount: 0,
                balanceYen: 0,
                adid: "default_ad".to_string(),
            }
        )
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    unsafe {
        db_conn = Connection::open_in_memory().ok();

        if let Some(conn) = &db_conn {
            conn.execute(
                "CREATE TABLE sharedb (
                    gid             INT IDENTITY(1,1) PRIMARY KEY,
                    pwd             TEXT,
                    sub             TEXT NOT NULL,
                    name            TEXT,
                    age             INTEGER,
                    job             TEXT,
                    balanceYen      INTEGER,
                    productName     TEXT,
                    productPriceYen TEXT,
                    productNumber   INTEGER
                 )",
                params![],
            ).unwrap();
        }

    }

    let addr_leader: &str = "0.0.0.0:8080";
    let addr: &str = "0.0.0.0:8081";

    if let Some(mode) = std::env::args().nth(1) {
        println!("Running \"leader mode\"@port8080");

        if mode == "leader" {
            HttpServer::new(||
                            App::new()
                            .service(register_profile)
                            .service(register_cart)
                            .service(sync)
                            )
                .bind(addr_leader)?
                .run()
                .await
        } else {
            println!("Running \"default mode\"@port8081. (arg = {})", mode);
            HttpServer::new(||
                            App::new()
                            .service(register_profile)
                            .service(register_cart)
                            .service(sync)
                            )
                .bind(addr)?
                .run()
                .await
        }

    } else {
        println!("No arg. Running \"default mode\"@port8081");
        HttpServer::new(||
                        App::new()
                        .service(register_profile)
                        .service(register_cart)
                        .service(sync)
                        )
            .bind(addr)?
            .run()
            .await
    }
}
