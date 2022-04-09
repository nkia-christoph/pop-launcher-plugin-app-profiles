#[macro_use] extern crate log;

use crate::config::*;
use crate::utils::*;

use async_std::{
  io::{
    Error as AsyncError,
    ErrorKind::InvalidInput,
    Result as AsyncResult,
  },
};
use futures_lite::{
  AsyncWrite,
  AsyncWriteExt,
  StreamExt,
};
use pop_launcher::{
  async_stdin,
  async_stdout,
  IconSource,
  json_input_stream,
  PluginResponse,
  PluginSearchResult,
  Request,
};
use serde_json;
use simplelog::*;
use std::{
  borrow::Cow,
  io::Result,
  path::PathBuf,
  process::Command,
};

mod config;
mod utils;


#[async_std::main]
pub async fn main() {

  let exec_path = std::env::current_exe().expect("couldn't get exec path");
  let exec_dir: PathBuf = exec_path.parent().unwrap().to_path_buf();
  let log_dir: PathBuf = exec_dir.join(PathBuf::from("log"));

  CombinedLogger::init(vec![
    WriteLogger::new(LevelFilter::Info, Config::default(),
      log_file(&log_dir, "info.log").unwrap()
    ),
    //WriteLogger::new(LevelFilter::Warn, Config::default(), log_file("warn.log").unwrap()),
    //WriteLogger::new(LevelFilter::Error, Config::default(), log_file("error.log").unwrap()),
  ]).unwrap();

  //error!("test error log verbose? {}", log_verbose());
  //warn!("test warn");
  //info!("test info");
  
  let mut app = App::new(async_stdout(), exec_dir);
  app.reload().await.unwrap();

  let mut requests = json_input_stream(async_stdin());

  while let Some(result) = requests.next().await {
    info!("incoming event...");

    match result {
      Ok(request) => match request {
        Request::Activate(id) => app.activate(id).await.unwrap(),
        // Request::ActivateContext { id, context } => app.activate_context(id, context).await,
        Request::Complete(id) => app.complete(id).await.unwrap(),
        // Request::Context(id) => app.context(id).await,
        Request::Search(query) => app.search(&query).await.unwrap(),
        Request::Exit => break,
        _ => (),
      },
      Err(why) => {
        app.send_err("malformed JSON request:").await.unwrap();
        error!("{}", why);
      },
    }
  }
}

struct App<W> {
  results: Vec<String>,
  path: PathBuf,
  out: W,
  catalogue: AppCataloge,
}

impl <W: AsyncWrite + Unpin> App<W> {
  fn new(out: W, path: PathBuf) -> Self {
    Self {
      results: Vec::with_capacity(128),
      path,
      out,
      catalogue: vec![],
    }
  }

  async fn reload(&mut self) -> Result<()> {
    info!("reloading...");

    self.catalogue.clear();
    self.catalogue = load_catalogue(self.path.clone()).unwrap();
    info!("reloaded");

    Ok(())
  }

  async fn activate(&mut self, id: u32) -> AsyncResult<()> {
    info!("activate: {}", &id);
    if let Some(line) = self.results.get(id as usize) {
      info!("launching result {}: {}", &id, &line);
      match line.split_once(' ') {
        Some((_cmd, _arg)) => {
          info!("{}###{}", &_cmd, &_arg);
          let args: Vec<String> = line.split(' ').map(|a| a.to_string()).collect();
          args.iter().for_each(|a| info!("{}", a));
          let _ = Command::new("sh").arg("-c").args(args).spawn();
        },
        None => {
          let _ = Command::new(line).spawn();
        },
      }
    }
    self.send(PluginResponse::Close).await
  }
  
  async fn complete(&mut self, _id: u32) -> AsyncResult<()> {
    info!("complete");
    // do stuff;
    self.send(PluginResponse::Close).await
  }

  async fn search(&mut self, query: &str) -> AsyncResult<()> {
    info!("searching...");

    let query = query.to_ascii_lowercase();
    info!("query: {}", &query);
    let q_shorthand = query[0..2].to_owned();
    let q_profile = query[3..].to_owned();
    info!("q_shorthand: {}, q_profile: {}", &q_shorthand, &q_profile);

    let mut id = 0;

    for app_config in self.catalogue.clone().iter() {
      let app_shorthand = app_config.conf.shorthand.to_ascii_lowercase();
      let match_app = q_shorthand == app_shorthand;
      info!("shorthand: {}, match_app: {}", &app_shorthand, &match_app);

      if match_app {
        for profile in app_config.entries.iter() {
          let match_profile = profile.name.to_ascii_lowercase().contains(&q_profile);
          info!("{}, query: {}, match_profile: {}", &profile.name, &query, &match_profile);

          if match_profile {
            self.send(PluginResponse::Append(PluginSearchResult {
              id: id as u32,
              name: profile.name.clone().into(),
              description: format!("open {} in new window", &profile.name),
              icon: app_config.conf.icon.as_ref()
                .map(|icon| IconSource::Name(icon.clone().into())),
              ..Default::default()
            })).await.unwrap();
            self.results.push(profile.cmd.clone().into());
            id += 1;
          }
        }

        if id == 0 {
          self.send(PluginResponse::Append(PluginSearchResult {
            id: id as u32,
            name: format!("Unknown profile - try one of the following"),
            description: format!("...or add a profile of that name."),
            icon: Some(IconSource::Name(Cow::Borrowed("dialog-error"))),
            ..Default::default()
          })).await.unwrap();
          self.results.push(format!(""));
          id += 1;

          for profile in app_config.entries.iter() {
            self.send(PluginResponse::Append(PluginSearchResult {
              id: id as u32,
              name: profile.name.clone().into(),
              description: format!("Open {} profile in new window", &profile.name),
              icon: app_config.conf.icon.as_ref()
                .map(|icon| IconSource::Name(icon.clone().into())),
              ..Default::default()
            })).await.unwrap();
            self.results.push(profile.cmd.clone().into());
            id += 1;
          }
        }
        break;
      }
    }

    if id == 0 {
      /*
      append!({
        id: id as u32,
        name: format!("Unknown shorthand - try one of the following"),
        description: format!("...or add/edit a ron file in the config folder"),
        icon: Some(IconSource::Name(Cow::Borrowed("system-help-symbolic"))),
        ..Default::default()
      }).await.unwrap();
      */
      self.send(PluginResponse::Append(PluginSearchResult {
        id: id as u32,
        name: format!("Unknown shorthand - try one of the following"),
        description: format!("...or add/edit a ron file in the config folder"),
        icon: Some(IconSource::Name(Cow::Borrowed("system-help-symbolic"))),
        ..Default::default()
      })).await.unwrap();
      self.results.push(format!(""));
      id += 1;

      for app_config in self.catalogue.clone().iter() {
        self.send(PluginResponse::Append(PluginSearchResult {
          id: id as u32,
          name: app_config.conf.shorthand.clone().into(),
          description: format!("Try the shorthand for {}!", &app_config.name),
          icon: app_config.conf.icon.as_ref()
            .map(|icon| IconSource::Name(icon.clone().into())),
          ..Default::default()
        })).await.unwrap();
        self.results.push(app_config.conf.shorthand.clone().into());
        id += 1;
      }

      if id == 1 {
        self.send_err("no profiles").await.unwrap();
      }
    }

    self.send(PluginResponse::Finished).await
  }

  async fn send(&mut self, response: PluginResponse) -> Result<()> {
    let mut response_json = serde_json::to_string(&response)
      .map_err(|err| AsyncError::new(InvalidInput, err))?;

    response_json.push('\n');

    info!("sending... {}", &response_json);

    self.out.write_all(response_json.as_bytes()).await?;
    self.out.flush().await?;

    info!("sent");

    Ok(())
  }

  async fn send_err(&mut self, e: &str) -> Result<()> {
    error!("{}", &e);
    self.send(PluginResponse::Append(PluginSearchResult {
      id: 0,
      name: "Error".to_owned(),
      description: e.to_owned(),
      icon: None,
      ..Default::default()
    }),).await?;
    self.send(PluginResponse::Finished).await?;

    Ok(())
  }
}
