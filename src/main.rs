#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]

//!
//! CRUD examples
//! 

mod db;
mod api;

use std::env;

use actix_web::{ HttpServer, App, web };

#[ actix_web::main ]
async fn main() -> std::io::Result< () >
{
  dotenv::dotenv().ok();
  pretty_env_logger::init();

  log::info!( "Start" );
  let db = db::Db::new
  (
    std::env::var( "NEO4J_ADDR" ).unwrap(),
    std::env::var( "NEO4J_DOMAIN" ).ok()
  ).await.expect( "Could not connect to database" )
  .login
  (
    env::var("NEO4J_USERNAME").unwrap(),
    env::var("NEO4J_PASSWORD").unwrap(),
  ).await.expect( "Looks like login or password is incorrect" );

  let db = web::Data::new( db );

  HttpServer::new( move ||
  {
    App::new()
    .app_data( db.clone() )
    .service( api::get )
    .service( api::post )
    .service( api::update )
    .service( api::delete )
  })
  .bind(( "127.0.0.1", 8080 ))?
  .run().await
}


// db.create( "", "name", "Peter" ).await?;
// db.create( ":languages", "name", "Rust" ).await?;
// db.create( ":languages", "name", "Java" ).await?;

// let nodes = db.get( "" ).await?;
// log::info!( "Read all:\n{:#?}", &nodes );

// let node = nodes[ 0 ].to_owned();
// log::info!( "First node\nid: {}\nlabels: {:#?}\nprops: {:#?}", &node.node_identity(), &node.labels(), &node.properties() );

// db.update( "{ name: \"Rust\" }", "+= { speed : \"Blazingly Fast\" }" ).await?;
// db.update( "{ name: \"Peter\" }", ":human" ).await?;
// log::info!( "Read all after update\n{:#?}", db.get( "" ).await? );

// log::info!( "Read single by label and name-field\n{:#?}", db.get( ":languages{ name : \"Rust\" }" ).await? );
// db.delete( "" ).await?;

// log::info!( "End" );