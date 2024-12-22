mod api;
mod model;
mod repository;

use api::todo::{
    get_todo,
    submit_todo,
    start_todo,
    complete_todo,
    pause_todo,
    fail_todo,
};
use repository::ddb::DDBRepository;
use actix_web::{HttpServer, App, web::Data, middleware::Logger};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let config = aws_config::load_from_env().await;
    HttpServer::new(move || {
        let ddb_repo: DDBRepository = DDBRepository::init(
            String::from("todo"),
            config.clone()
        );
        let ddb_data = Data::new(
            ddb_repo
        );
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(ddb_data)
            .service(get_todo)
            .service(submit_todo)
            .service(start_todo)
            .service(complete_todo)
            .service(pause_todo)
            .service(fail_todo)
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await
}