use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};

#[get("/")]
async fn top() -> impl Responder {
    HttpResponse::Ok().body("Hello Top Page!")
}

#[get("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/{id}/{name}")]
async fn index(web::Path((id, name)): web::Path<(u32, String)>) -> impl Responder {
    format!("Hello {}! id:{}", name, id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let addr_leader: &str = "127.0.0.1:8080";
    let addr: &str = "127.0.0.1:8081";

    if let Some(mode) = std::env::args().nth(1) {
        println!("Running \"leader mode\"");

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
            println!("Running \"default mode\". (arg = {})", mode);
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
        println!("No arg. Running \"default mode\"");
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
