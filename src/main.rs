#[macro_use]
extern crate log;

use crate::config::{load_catalogue, AppCatalogue, AppConfig};
use crate::utils::log_file;

use futures_util::StreamExt;
use pop_launcher::{
    async_stdin, async_stdout, json_input_stream, IconSource, PluginResponse, PluginSearchResult,
    Request,
};
use shlex::split as shlex_split;
use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};
use std::{borrow::Cow, io, path::PathBuf, process::Command};
use tokio::io::{AsyncWrite, AsyncWriteExt};

mod config;
mod utils;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("app-profiles failed: {err}");
    }
}

async fn run() -> io::Result<()> {
    let exec_path = std::env::current_exe()?;
    let exec_dir = exec_path
        .parent()
        .ok_or_else(|| io::Error::other("couldn't get executable directory"))?
        .to_path_buf();

    init_logger(&exec_dir)?;

    let mut app = App::new(async_stdout(), exec_dir);
    app.reload().await?;

    let mut requests = json_input_stream(async_stdin());
    while let Some(result) = requests.next().await {
        info!("incoming event...");

        match result {
            Ok(request) => {
                if let Request::Exit = request {
                    break;
                }
                if let Err(err) = app.handle_request(request).await {
                    app.send_err(&format!("request failed: {err}")).await?;
                }
            }
            Err(err) => {
                app.send_err("malformed JSON request").await?;
                error!("{err}");
            }
        }
    }

    Ok(())
}

fn init_logger(exec_dir: &std::path::Path) -> io::Result<()> {
    let log_dir = exec_dir.join("log");
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        log_file(&log_dir, "info.log")?,
    )])
    .map_err(|err| io::Error::other(err.to_string()))
}

struct App<W> {
    results: Vec<String>,
    path: PathBuf,
    out: W,
    catalogue: AppCatalogue,
}

impl<W: AsyncWrite + Unpin> App<W> {
    fn new(out: W, path: PathBuf) -> Self {
        Self {
            results: Vec::with_capacity(128),
            path,
            out,
            catalogue: Vec::new(),
        }
    }

    async fn handle_request(&mut self, request: Request) -> io::Result<()> {
        match request {
            Request::Activate(id) => self.activate(id).await,
            Request::Complete(id) => self.complete(id).await,
            Request::Search(query) => self.search(&query).await,
            _ => Ok(()),
        }
    }

    async fn reload(&mut self) -> io::Result<()> {
        info!("reloading...");
        self.catalogue =
            load_catalogue(&self.path).map_err(|err| io::Error::other(err.to_string()))?;
        info!("reloaded");
        Ok(())
    }

    async fn activate(&mut self, id: u32) -> io::Result<()> {
        info!("activate: {id}");

        if let Some(line) = self.results.get(id as usize) {
            info!("launching result {id}: {line}");
            let parts = shlex_split(line)
                .unwrap_or_else(|| line.split_whitespace().map(str::to_owned).collect());
            if let Some((cmd, args)) = parts.split_first() {
                let _ = Command::new(cmd).args(args).spawn();
            }
        }

        self.send(PluginResponse::Close).await
    }

    async fn complete(&mut self, _id: u32) -> io::Result<()> {
        info!("complete");
        self.send(PluginResponse::Close).await
    }

    async fn search(&mut self, query: &str) -> io::Result<()> {
        info!("searching...");
        self.results.clear();

        let normalized = query.to_ascii_lowercase();
        let (q_shorthand, q_profile) = parse_query(&normalized);
        info!("query: {normalized}, shorthand: {q_shorthand}, profile: {q_profile}");

        let mut id = 0_u32;

        if let Some(app_config) = self
            .catalogue
            .iter()
            .find(|cfg| cfg.conf.shorthand.eq_ignore_ascii_case(q_shorthand))
            .cloned()
        {
            for profile in app_config
                .entries
                .iter()
                .filter(|entry| entry.name.to_ascii_lowercase().contains(q_profile))
            {
                self.send(PluginResponse::Append(PluginSearchResult {
                    id,
                    name: profile.name.clone(),
                    description: profile_description(profile),
                    icon: app_config
                        .conf
                        .icon
                        .as_ref()
                        .map(|icon| IconSource::Name(icon.clone().into())),
                    ..Default::default()
                }))
                .await?;
                self.results.push(profile.cmd.clone());
                id += 1;
            }

            if id == 0 {
                self.send_unknown_profile(&app_config, &mut id).await?;
            }
        } else {
            self.send_unknown_shorthand(&mut id).await?;
        }

        self.send(PluginResponse::Finished).await
    }

    async fn send_unknown_profile(
        &mut self,
        app_config: &AppConfig,
        id: &mut u32,
    ) -> io::Result<()> {
        self.send(PluginResponse::Append(PluginSearchResult {
            id: *id,
            name: "Unknown profile - try one of the following".into(),
            description: "...or add a profile of that name.".into(),
            icon: Some(IconSource::Name(Cow::Borrowed("dialog-error"))),
            ..Default::default()
        }))
        .await?;
        self.results.push(String::new());
        *id += 1;

        for profile in &app_config.entries {
            self.send(PluginResponse::Append(PluginSearchResult {
                id: *id,
                name: profile.name.clone(),
                description: profile_description(profile),
                icon: app_config
                    .conf
                    .icon
                    .as_ref()
                    .map(|icon| IconSource::Name(icon.clone().into())),
                ..Default::default()
            }))
            .await?;
            self.results.push(profile.cmd.clone());
            *id += 1;
        }

        Ok(())
    }

    async fn send_unknown_shorthand(&mut self, id: &mut u32) -> io::Result<()> {
        self.send(PluginResponse::Append(PluginSearchResult {
            id: *id,
            name: "Unknown shorthand - try one of the following".into(),
            description: "...or add/edit a ron file in the config folder".into(),
            icon: Some(IconSource::Name(Cow::Borrowed("system-help-symbolic"))),
            ..Default::default()
        }))
        .await?;
        self.results.push(String::new());
        *id += 1;

        let shorthand_results: Vec<(String, String, Option<String>)> = self
            .catalogue
            .iter()
            .map(|cfg| {
                (
                    cfg.conf.shorthand.clone(),
                    cfg.name.clone(),
                    cfg.conf.icon.clone(),
                )
            })
            .collect();

        for (shorthand, name, icon) in shorthand_results {
            self.send(PluginResponse::Append(PluginSearchResult {
                id: *id,
                name: shorthand.clone(),
                description: format!("Try the shorthand for {}!", name),
                icon: icon.map(|icon| IconSource::Name(icon.into())),
                ..Default::default()
            }))
            .await?;
            self.results.push(shorthand);
            *id += 1;
        }

        if *id == 1 {
            self.send_err("no profiles").await?;
        }

        Ok(())
    }

    async fn send(&mut self, response: PluginResponse) -> io::Result<()> {
        let mut response_json =
            serde_json::to_string(&response).map_err(|err| io::Error::other(err.to_string()))?;
        response_json.push('\n');

        info!("sending... {response_json}");

        self.out.write_all(response_json.as_bytes()).await?;
        self.out.flush().await?;

        info!("sent");
        Ok(())
    }

    async fn send_err(&mut self, err: &str) -> io::Result<()> {
        error!("{err}");
        self.send(PluginResponse::Append(PluginSearchResult {
            id: 0,
            name: "Error".to_owned(),
            description: err.to_owned(),
            icon: None,
            ..Default::default()
        }))
        .await?;
        self.send(PluginResponse::Finished).await
    }
}

fn profile_description(profile: &config::AppEntry) -> String {
    if profile.desc.trim().is_empty() {
        format!("open {} in new window", profile.name)
    } else {
        profile.desc.clone()
    }
}
fn parse_query(query: &str) -> (&str, &str) {
    let trimmed = query.trim();
    if let Some((shorthand, profile)) = trimmed.split_once(' ') {
        (shorthand.trim(), profile.trim())
    } else {
        (trimmed, "")
    }
}
