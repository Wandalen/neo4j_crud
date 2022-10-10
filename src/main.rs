use std::env;

use tokio::io::BufStream;
use tokio_util::compat::*;

use bolt_client::*;
use bolt_proto::{ message::*, value::*, version::*, Message };


#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  dotenv::dotenv().ok();

  let stream = Stream::connect
  (
    std::env::var( "NEO4J_ADDR" ).unwrap(),
    std::env::var( "NEO4J_DOMAIN" ).ok()
  ).await?;
  let stream = BufStream::new( stream ).compat();

  let mut client = Client::new( stream, &[ V4_0, 0, 0, 0 ] ).await?;
    
  // Send a HELLO message with authentication details to the server to initialize
  // the session.
  let response: Message = client.hello
  (
    Metadata::from_iter( vec!
    [
        ( "user_agent", "neo4j" ),
        ( "scheme", "basic" ),
        ( "principal", &env::var("NEO4J_USERNAME")? ),
        ( "credentials", &env::var("NEO4J_PASSWORD")? ),
    ])
  ).await?;
  assert!( Success::try_from( response ).is_ok() );
  
  // Create record with name = "Rust", label = "Language"
  let pull_meta = Metadata::from_iter( vec![( "n", 1 )] );
  let params = Params::from_iter( vec![("name", "Rust")] );
  client.run
  (
    "CREATE ( :Language { name: $name } );",
    Some( params ), None
  ).await?;
  client.pull( Some( pull_meta.clone() ) ).await?;

  // Read all records with label "Lanbuage"
  client.run( "MATCH ( elem:Language ) RETURN elem;", None, None ).await?;
  let ( records, _response ) = client.pull( Some( pull_meta.clone() ) ).await?;
  for record in records
  {
    // Parse to native type
    let node = Node::try_from( record.fields()[ 0 ].clone() )?;
    dbg!( &node );
  }

  // Update node with name: Rust
  let pull_meta = Metadata::from_iter( vec![( "n", -1 )] );
  client.run
  (
    r#"MATCH ( n { name : "Rust" } ) SET n += { speed : "Blazingly Fast" }"#,
    None, None
  ).await?;
  client.pull( Some( pull_meta.clone() ) ).await?;
  
  // Read again to see result of update
  client.run( "MATCH ( elem:Language ) RETURN elem;", None, None ).await?;
  let ( records, _response ) = client.pull( Some( pull_meta.clone() ) ).await?;
  for record in records
  {
    // Parse to native type
    let node = Node::try_from( record.fields()[ 0 ].clone() )?;
    dbg!( &node );
  }

  // Delete all records
  client.run( "MATCH ( n ) DETACH DELETE n", None, None ).await?;
  let pull_meta = Metadata::from_iter( vec![("n", -1)] );
  client.pull( Some( pull_meta ) ).await?;

  // End the connection with the server
  client.goodbye().await?;

  Ok(())
}
