use std::collections::HashMap;
use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use rusqlite::{params, Connection, Result};

static mut db_conn: Option<rusqlite::Connection> = None;

#[derive(Deserialize)]
struct SyncCredential {
    gid: i64,
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
struct RegisterRequest {
    gid: i64,
    vc_prof: String,
    vc_cart: String,
}


// Output Json Formats (to be VC as well)
#[derive(Serialize)]
struct OutputRegVC {
    gid: i64,
    sub: String,
    name: String,
    age: u32,
    job: String,
    balanceYen: i32,
}

#[derive(Debug, Default)]
struct PersonalParams {
    age: u32,
    job: String,
    balanceYen: i32,
    productName: String,
    productPriceYen: i32,
    productNumber: i32,
    gid: i64,
}

#[derive(Serialize)]
struct OutputReceiptVC {
    gid: i64,
    sub: String,
    ProductName: String,
    productPriceYen: i32,
    productNumber: i32,
}

#[derive(Default, Serialize)]
struct OutputSync {
    gid: i64,
    amount: i32,
    adid: String,
}

fn insert(gid: i64, vc_prof: &VcProfile, vc_cart: &VcCart) -> Result<i64, rusqlite::Error> {
    unsafe {
        if let Some(conn) = &mut db_conn {
            let tx = conn.transaction()?;
        
            let last_gid = if gid == 0 {
                tx.execute(
                    "INSERT INTO sharedb
                        (pwd, sub, name, age, job, balanceYen,
                         productName, productPriceYen, productNumber)
                    VALUES
                        (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        Some("testpwd".to_string()),
                        vc_prof.common.sub.clone(),
                        Some(vc_prof.vc.credentialSubject.profile.name.clone()),
                        Some(vc_prof.vc.credentialSubject.profile.age),
                        Some(vc_prof.vc.credentialSubject.profile.job.clone()),
                        Some(vc_prof.vc.credentialSubject.profile.balanceYen),
                        Some(vc_cart.vc.credentialSubject.cart.productName.to_string()),
                        Some(vc_cart.vc.credentialSubject.cart.productPriceYen),
                        Some(vc_cart.vc.credentialSubject.cart.productNumber)
                    ]
                )?;
            
                tx.last_insert_rowid()
            } else {
                // todo: update where gid = gid
                gid
            };

            tx.commit()?;

            Ok(last_gid)
        } else {
            Ok(0)
        }
    }
}

#[post("/register")]
async fn register(req: web::Form<RegisterRequest>) -> impl Responder {
    let vc_prof: VcProfile = serde_json::from_str(&req.vc_prof).unwrap();
    let vc_cart: VcCart = serde_json::from_str(&req.vc_cart).unwrap();

    let last_gid = insert(req.gid, &vc_prof, &vc_cart).unwrap();

    web::Json(
        OutputRegVC {
            gid: last_gid,
            sub: vc_prof.common.sub.clone(),
            name: vc_prof.vc.credentialSubject.profile.name.clone(),
            age: vc_prof.vc.credentialSubject.profile.age,
            job: vc_prof.vc.credentialSubject.profile.job.clone(),
            balanceYen: vc_prof.vc.credentialSubject.profile.balanceYen,
        }
    )
}

fn getGroup(gid: i64) -> PersonalParams {

    unsafe {

        if let Some(conn) = &db_conn {
        
            let mut q = conn.prepare(
                "SELECT
                    age, job, balanceYen,
                    productName, productPriceYen, productNumber
                    ,gid
                FROM sharedb
                WHERE gid = :gid"
                ).unwrap();
        
            let mut pp_iter = q.query_map_named(
                &[(":gid", &gid)],
                |row| {
                    Ok(
                        PersonalParams {
                            age: row.get(0)?,
                            job: row.get(1)?,
                            balanceYen: row.get(2)?,
                            productName: row.get(3)?,
                            productPriceYen: row.get(4)?,
                            productNumber: row.get(5)?,
                            gid: row.get(6)?,
                        }
                    )
                }).unwrap();

            pp_iter.next().unwrap().unwrap()
        
        } else {

            Default::default()

        }
    }

}

#[post("/sync")]
async fn sync(cred: web::Form<SyncCredential>) -> impl Responder {

    let grp = getGroup(cred.gid);

    // todo: implement below by MPC
    let amount = grp.productPriceYen * grp.productNumber;
    let adid =
        if grp.age >= 20 {
            "beer_ad".to_string()
        } else {
            "juice_ad".to_string()
        };

    web::Json(
        OutputSync {
            gid: cred.gid,
            amount: amount,
            adid: adid,
        }
    )

}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    unsafe {
        db_conn = Connection::open_in_memory().ok();

        if let Some(conn) = &db_conn {
            conn.execute(
                "CREATE TABLE sharedb (
                    gid             INTEGER PRIMARY KEY,
                    pwd             TEXT,
                    sub             TEXT NOT NULL,
                    name            TEXT,
                    age             INTEGER,
                    job             TEXT,
                    balanceYen      INTEGER,
                    productName     TEXT,
                    productPriceYen INTEGER,
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
                            .service(register)
                            .service(sync)
                            )
                .bind(addr_leader)?
                .run()
                .await
        } else {
            println!("Running \"default mode\"@port8081. (arg = {})", mode);
            HttpServer::new(||
                            App::new()
                            .service(register)
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
                        .service(register)
                        .service(sync)
                        )
            .bind(addr)?
            .run()
            .await
    }
}
