#[macro_use] extern crate log;

use crate::config::*;

use async_std::{
  io::{
    Error as AsyncError,
    ErrorKind::InvalidInput,
    Result as AsyncResult,
  },
  //println,
};
use futures_lite::{
  AsyncWrite,
  AsyncWriteExt,
  StreamExt,
};
use pop_launcher::{
  async_stdin,
  async_stdout,
  //IconSource,
  json_input_stream,
  PluginResponse,
  //PluginSearchResult,
  Request,
};
use serde_json;
use simplelog::*;
use std::{
  fs::File,
  fs::OpenOptions,
  io::Result,
};

mod config;


#[async_std::main]
pub async fn main() {

  CombinedLogger::init(vec![
    WriteLogger::new(LevelFilter::Info, Config::default(), log_file("info.log").unwrap()),
    WriteLogger::new(LevelFilter::Warn, Config::default(), log_file("warn.log").unwrap()),
    WriteLogger::new(LevelFilter::Error, Config::default(), log_file("error.log").unwrap()),
  ]).unwrap();

  //error!("test error");
  //warn!("test warn");
  //info!("test info");
  
  let mut app = App::new(async_stdout());
  app.reload().await.unwrap();

  let mut requests = json_input_stream(async_stdin());

  while let Some(result) = requests.next().await {
    info!("incoming event...");

    match result {
      Ok(request) => match request {
        Request::Activate(id) => app.activate(id).await.unwrap(),
        // Request::ActivateContext { id, context } => app.activate_context(id, context).await,
        // Request::Context(id) => app.context(id).await,
        Request::Search(query) => app.search(&query).await.unwrap(),
        Request::Exit => break,
        _ => (),
      },

      Err(why) => {
        error!("malformed JSON request: {}", why);
      }
    }
  }
}

struct App<W> {
  out: W,
  catalogue: AppCataloge,
}

impl <W: AsyncWrite + Unpin> App<W> {
  fn new(out: W) -> Self {
    Self {
      out,
      catalogue: vec![],
    }
  }

  async fn reload(&mut self) -> Result<()> {
    info!("reloading...");

    self.catalogue.clear();
    self.catalogue = load_catalogue().unwrap();
    info!("reloaded");

    Ok(())
  }

  async fn activate(&mut self, _id: u32) -> AsyncResult<()> {
    info!("activate");
    // open App;
    self.send(PluginResponse::Close).await
  }

  async fn search(&mut self, query: &str) -> AsyncResult<()> {
    info!("searching...");

    let query = query.to_ascii_lowercase();

    let mut m: bool = false;

    for app_config in self.catalogue.iter() {
      let shorthand = app_config.conf.shorthand.to_ascii_lowercase();
      let match_app = query.contains(&shorthand);

      if match_app {
        for (_id, profile) in app_config.entries.iter().enumerate() {
          let match_profile = profile.name.contains(&query);

          if match_profile {
            /*
            self.send(PluginResponse::Append(PluginSearchResult {
              id: id as u32,
              name: profile.name,
              description: profile.path.clone(),
              icon: app_config.conf.icon.as_ref()
                .map(|icon| IconSource::Name(icon.clone().into())),
              ..Default::default()
            })).await;
            */

            m = true;
          }
        }
      }
    }

    if m == false {
      let mut _id = 0;

      for app_config in self.catalogue.iter() {
        for _entry in app_config.entries.iter() {
          /*
          self.send(PluginResponse::Append(PluginSearchResult {
            id: id as u32,
            name: /*app_conf.name +*/entry.name.clone(),
            description: entry.path.clone(),
            icon: app_config.conf.icon.as_ref()
              .map(|icon| IconSource::Name(icon.clone().into())),
            ..Default::default()
          })).await;
          */

          _id += 1;
        }
      }
    }

    self.send(PluginResponse::Finished).await
  }

  async fn send(&mut self, response: PluginResponse) -> Result<()> {
    let mut response_json = serde_json::to_string(&response)
      .map_err(|err| AsyncError::new(InvalidInput, err))?;

    response_json.push('\n');

    info!("sending... {}", response_json);

    self.out.write_all(response_json.as_bytes()).await?;
    self.out.flush().await?;

    info!("sent");

    Ok(())
  }
}

fn log_file(file_name: &str) -> Result<File> {
  OpenOptions::new().create(true).append(true).open(file_name)
}
