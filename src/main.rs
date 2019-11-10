#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate diesel;


use diesel::query_dsl::RunQueryDsl;
use diesel::sql_query;
use diesel::sql_types::Blob;

use rocket_contrib::serve::StaticFiles;

use rocket::http::Header;

#[database("census_geoms")]
struct CensusGeomsDbConn(diesel::SqliteConnection);


#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

struct ContentEncodingHeader{
    encoding:String
}

impl From<ContentEncodingHeader> for Header<'_>{
  fn from(c: ContentEncodingHeader)->Self{
    Header::new("Content-Encoding", c.encoding)
  }
}

#[derive(Responder)]
#[response(status = 200, content_type = "application/x-protobuf" )]
struct TileResponder{
    inner: Vec<u8>,
    header: ContentEncodingHeader
}


#[derive(QueryableByName, PartialEq, Debug)]
struct Tile{
  #[sql_type = "Blob"]
   tile_data: Vec<u8>
}


#[get("/test_tile/<z>/<x>/<y>")]
fn test_tile(conn: CensusGeomsDbConn, z: u32, x: u32, y: u32)->TileResponder{
    let y = (1 << z) - 1 - y;
    let query = format!(
"select tile_data from tiles where zoom_level = {} and tile_column = {} and tile_row = {}"
        ,z,x,y);
    println!("{}",query);

    let result: Tile  = sql_query(query).get_result(&*conn).expect("query failed to run");

    TileResponder{
        inner:result.tile_data,
        header: ContentEncodingHeader{encoding:String::from("gzip")}
    }
}

fn main() {
    rocket::ignite()
        .attach(CensusGeomsDbConn::fairing())
        .mount("/", routes![hello,test_tile])
        .mount("/", StaticFiles::from("./public")).launch();
}
