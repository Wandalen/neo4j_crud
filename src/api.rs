use actix_web::{ get, post, Responder, web, delete, patch };
use bolt_client::bolt_proto::Value;
use bolt_proto::value::Node;
use serde::{ Serialize, Deserialize };

use crate::db::Db;

#[ derive( Debug, Serialize, Deserialize ) ]
pub struct Todo
{
  pub title : String,
  pub description : String,
}

#[ derive( Debug ) ]
pub enum TodoError
{
  HasNoTitle,
  HasNoDescription,
  IncompatibleTypeInDescription,
}

impl TryFrom< Node > for Todo
{
  type Error = TodoError;

  fn try_from( value: Node ) -> Result< Self, Self::Error >
  {
    let labels = value.labels();
    let props = value.properties();

    if labels.is_empty() { return Err( TodoError::HasNoTitle )  }
    let description = props.get( "description" ).ok_or( TodoError::HasNoDescription )?;

    if let Value::String( description ) = description
    {
      Ok( Todo { title : labels[ 0 ].to_owned(), description : description.to_owned() } )
    }
    else
    {
      Err( TodoError::IncompatibleTypeInDescription )
    }
  }
}


#[ get( "/get" ) ]
pub async fn get( db : web::Data< Db > ) -> actix_web::Result< impl Responder >
{
  log::info!( "GET" );

  let data = db.get( "" ).await.unwrap();
  let data = data.iter()
  .map( | node | Todo::try_from( node.to_owned() ) )
  .filter( | node | node.is_ok() )
  .map( | node | node.unwrap() )
  .collect::< Vec< _ > >();

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
