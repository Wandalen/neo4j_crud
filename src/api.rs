use actix_web::{ get, post, Responder, web, delete, patch };
use bolt_client::bolt_proto::Value;
use serde::{ Serialize, Deserialize };

use crate::db::Db;

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Todo
{
  pub title : String,
  pub description : String,
}


#[ get( "/get" ) ]
pub async fn get( db : web::Data< Db > ) -> actix_web::Result< impl Responder >
{
  log::info!( "GET" );
  let data = db.get( "" ).await.unwrap();
  // TODO: Rewrite this. IDK how
  // bolt-proto Value has no serialize/deserialize
  let data = data.iter().map( | node |
  {
    let props = node.properties().iter().map( |( key, value )|
    {
      if let Value::String( string ) = value
      {
        ( key, string.to_string() )
      }
      else
      {
        ( key, "".to_string() )
      }
    }).collect::< Vec< _ > >();
    Todo{ title : node.labels()[ 0 ].clone(), description : props[ 0 ].1.to_owned() }
  }).collect::< Vec< _ > >();

  Ok( web::Json( data ) )
}

#[ post( "/post" ) ]
pub async fn post( db : web::Data< Db >, input : web::Json< Todo > ) -> actix_web::Result< impl Responder >
{
  log::info!( "POST" );
  db.create
  (
    format!( ":{title}", title = &input.title ),
    "description".to_owned(),
    input.description.to_owned()
  ).await.unwrap();

  Ok( web::Json( "Posted" ) )
}

#[ patch( "/update/{title}" ) ]
pub async fn update( db : web::Data< Db >, title : web::Path<( String, )>, new_todo : web::Json< String > ) -> actix_web::Result< impl Responder >
{
  log::info!( "UPDATE" );
  db.update
  (
    format!( ":{title}", title = title.0 ),
    format!( "= {{ description: \"{description}\" }}", description = new_todo )
  ).await.unwrap();

  Ok( web::Json( "Updated" ) )
}

#[ delete( "/delete/{title}" ) ]
pub async fn delete( db : web::Data< Db >, title : web::Path<( String , )> ) -> actix_web::Result< impl Responder >
{
  log::info!( "DELETE" );
  db.delete( format!( ":{title}", title = title.0 ) ).await.unwrap();

  Ok( web::Json( "Deleted" ) )
}