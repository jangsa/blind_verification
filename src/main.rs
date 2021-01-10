use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    sub: String,
    name: String,
    age: i32,
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
async fn json(info: web::Json<Info>) -> impl Responder {
    format!("Welcome, {}! Your subject id is {}, and age is {}", info.name, info.sub, info.age)
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
                        )
            .bind(addr)?
            .run()
            .await
    }

}
