use tokio::{ io::BufStream, net::ToSocketAddrs };
use tokio_util::compat::{ Compat, TokioAsyncReadCompatExt };

use bolt_client::*;
use bolt_proto::{ message::*, value::*, version::*, Message };


///
/// Database wrapper
/// 

pub struct Db
{
  client : Client< Compat< BufStream< Stream > > >,
}

impl Db
{
  ///
  /// Create new Db instance and connect to database
  /// 
  pub async fn new< Addr, Domain >( addr : Addr, domain : Option< Domain > ) -> Result< Self, Box< dyn std::error::Error > >
  where
    Addr : ToSocketAddrs,
    Domain : AsRef< str >
  {
    let stream = Stream::connect( addr, domain ).await?;
    let stream = BufStream::new( stream ).compat();
    let client = Client::new( stream, &[ V4_0, 0, 0, 0 ] ).await?;

    Ok( Self{ client } )
  }

  ///
  /// Login user to database
  /// 
  //? Cant be called before new, because need self
  //* But mast be called before other methods
  pub async fn login< Un, Pw >( mut self, username : Un, password : Pw ) -> Result< Self, Box< dyn std::error::Error > >
  where
    Un : Into< String >,
    Pw : Into< String >
  {
    let response: Message = self.client.hello
    (
      Metadata::from_iter( vec!
      [
          ( "user_agent", "neo4j" ),
          ( "scheme", "basic" ),
          ( "principal", &username.into() ),
          ( "credentials", &password.into() ),
      ])
    ).await?;

    // ! Make better errors
    Success::try_from( response ).or( Err( "Login error" ) )?;

    Ok( self )
  }

  ///
  /// Create new instance inside database
  /// 
  pub async fn create< S >( &mut self, label : S, field_name : S, value : S ) -> Result< (), Box< dyn std::error::Error > >
  where
    S : Into< String >,
  {
    log::info!( "Create" );
    let pull_meta = Metadata::from_iter( vec![( "n", 1 )] );

    let params = Params::from_iter( vec![( "value", value.into() )] );
    let query = format!( "CREATE ( {lb} {{ {f_n} : $value }} );", lb=label.into(), f_n = field_name.into() );
    self.client.run( query, Some( params ), None ).await?;
    self.client.pull( Some( pull_meta.clone() ) ).await?;

    Ok( () )
  }

  ///
  /// Gets Nodes that match to filter
  /// 
  // filter examples
  // ":Language{ name : "Rust" } => All Nodes where labels = ["Language"] and name = "Rust"
  // empty => All Nodes
  pub async fn get< S >( &mut self, filter : S ) -> Result< Vec< Node >, Box< dyn std::error::Error > >
  where
    S : Into< String >
  {
    log::info!( "Read" );
    let pull_meta = Metadata::from_iter( vec![( "n", -1 )] );

    let query = format!( "MATCH ( n{filter} ) RETURN n;", filter = filter.into() );
    self.client.run( query, None, None ).await?;
    let ( records, _response ) = self.client.pull( Some( pull_meta.clone() ) ).await?;

    Ok( records.iter().map( | rec | Node::try_from( rec.fields()[ 0 ].clone() ).unwrap() ).collect() )
  }

  /// 
  /// Updates all Node that match to filter
  /// Changes - how to change the nodes
  /// 
  // filter examples
  // ":Language{ name : "Rust" } => All Nodes where labels = ["Language"] and name = "Rust"
  // empty => All Nodes
  // changes examples
  // "+= { speed : "Blazingly Fast" }" => Add field speed with value "Blazingly Fast"
  // "= { title : "The fastest language in the world" } => replace the Nodes to this
  // ":human" => Add to the Nodes label "human"
  pub async fn update< S >( &mut self, filter : S, changes : S ) -> Result< (), Box< dyn std::error::Error > >
  where
    S : Into< String >
  {
    log::info!( "Update" );
    let pull_meta = Metadata::from_iter( vec![( "n", -1 )] );

    let query = format!( r#"MATCH ( n{filter} ) SET n{changes}"#, filter = filter.into(), changes = changes.into() );
    self.client.run( query, None, None ).await?;
    self.client.pull( Some( pull_meta.clone() ) ).await?;

    Ok( () )
  }

  pub async fn delete< S >( &mut self, filter : S ) -> Result< (), Box< dyn std::error::Error > >
  where
    S : Into< String >
  {
    log::info!( "Delete" );
    let pull_meta = Metadata::from_iter( vec![("n", -1)] );

    let query = format!( "MATCH ( n{filter} ) DETACH DELETE n", filter = filter.into() );
    self.client.run( query, None, None ).await?;
    self.client.pull( Some( pull_meta ) ).await?;

    Ok( () )
  }
}

impl Drop for Db
{
  fn drop( &mut self )
  {
    futures::executor::block_on( async
    {
      self.client.goodbye().await.unwrap()
    })
  }
}