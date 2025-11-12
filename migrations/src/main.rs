use sea_orm_migration::prelude::*;
use std::env;

use migration::Migrator;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    
    let db_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("up");
    
    let db = sea_orm::Database::connect(&db_url).await
        .expect("Failed to connect to database");
    
    match command {
        "up" => {
            Migrator::up(&db, None).await
                .expect("Failed to run migrations");
            println!("✓ Migrations applied successfully");
        }
        "down" => {
            Migrator::down(&db, None).await
                .expect("Failed to rollback migration");
            println!("✓ Migration rolled back successfully");
        }
        "fresh" => {
            Migrator::fresh(&db).await
                .expect("Failed to run fresh migrations");
            println!("✓ Fresh migrations applied successfully");
        }
        "status" => {
            Migrator::status(&db).await
                .expect("Failed to get migration status");
        }
        _ => {
            eprintln!("Unknown command: {}. Use: up, down, fresh, or status", command);
            std::process::exit(1);
        }
    }
}

