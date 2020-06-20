extern crate satellites_service;

use actix_web::{App, guard, HttpServer, web};

use satellites_service::{create_schema, index, index_playground, prepare_env};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let pool = prepare_env();
    let schema = create_schema(pool);

    HttpServer::new(move || App::new()
        .data(schema.clone())
        .service(web::resource("/").guard(guard::Post()).to(index))
        .service(web::resource("/").guard(guard::Get()).to(index_playground))
    )
        .bind("0.0.0.0:8002")?
        .run()
        .await
}
