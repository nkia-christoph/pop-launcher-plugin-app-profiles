use convert_case::{Case::Title, Casing};
use dirs;
use glob::{
  glob_with,
  MatchOptions,
  Paths,
  PatternError,
};
use regex::Regex;
use ron::de::from_reader;
use serde::Deserialize;
use serde_regex;
use std::{
  env::current_dir,
  error::Error,
  fs::{
    File,
    read_dir,
  },
  io::BufReader,
  path::PathBuf,
  rc::Rc,
};

const CONFIG: &str = "/config/*.ron";

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Ron {
  pub shorthand: String,
  cmd: String,
  args: String,
  profile_dirs: Vec<String>,

  #[serde(with = "serde_regex")]
  profile_regex: Regex,

  opt_entries: Option<Vec<OptEntry>>,
  pub icon: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct OptEntry{
  name: String,
  cmd: Option<String>,
  args: Option<String>,
}

pub type AppCataloge = Vec<Rc<AppConfig>>;
pub type AppEntries = Vec<Rc<AppEntry>>;

pub struct AppConfig {
  pub conf: Rc<Ron>,
  pub entries: AppEntries,
}

pub struct AppEntry {
  pub name: String,
  pub path: String,
  pub cmd: String,
}

fn load_ron(file_path: &str) -> Result<Ron, Box<dyn Error>> {
  info!("loading ron...");

  let file = File::open(file_path)?;
  let reader = BufReader::new(file);
  Ok(from_reader(reader)?)
}

fn get_rons() -> Result<Paths, Box<PatternError>> {

  let glob_pat = current_dir().unwrap().display().to_string() + CONFIG;
  info!("getting rons with glob: {}", glob_pat);
  let options = MatchOptions {
    case_sensitive: false,
    ..Default::default()
  };

  Ok(glob_with(&glob_pat, options)?)
}

pub fn load_catalogue() -> Result<AppCataloge, Box<dyn Error>> {
  let mut app_cat: AppCataloge = vec![];

  // TODO tidy up with collect https://doc.rust-lang.org/book/ch13-03-improving-our-io-project.html#making-code-clearer-with-iterator-adaptors
  for _entry in get_rons()? {
    if let Ok(path) = _entry {
      let path_str = path.display().to_string();

      info!("searching in path {}", &path_str);

      let /*mut*/ ron: Ron = load_ron(&path_str)?;
      let mut app_entries: AppEntries = vec![];

      // profiles
      for dir in ron.profile_dirs.iter() {
        let mut abs_dir = PathBuf::from(&dir);
        if dir.contains("~/") {
          abs_dir = dirs::home_dir()
            .expect("home directory required")
            .join(&dir[2..]);
        }

        info!("searching in dir {}", abs_dir.to_string_lossy());

        for profile in read_dir(abs_dir.to_string_lossy().as_ref())? {
          let os_fn = profile?.file_name();
          let str_fn = os_fn.to_str().unwrap();
          info!("matching file {}", &str_fn);

          let m = ron.profile_regex.captures(&str_fn);
          match &m {
            Some(p) => {
              let name = p
                .get(1)
                .expect((format!("no group matched for name! check regex in {}", &path_str)).as_str())
                .as_str()
                .to_case(Title);
              app_entries.push(Rc::new(AppEntry{
                name: name.clone(),
                path: String::from(&path_str),
                cmd: [
                  ron.cmd.clone(),
                  ron.args.clone(),
                  p.get(0).unwrap().as_str().to_string()
                ].join(" "),
              }));
              info!("[OK] matched file {}/{} as {}", abs_dir.to_string_lossy(), &str_fn, &name);
            },
            _ => (),
          }
        }
      }

      // optional entries
      match &ron.opt_entries {
        Some(optional) => for opt in optional.iter() {
          info!("adding optional entry: {}", &opt.name);

          match &opt.cmd {
            Some(cmd) => {
              app_entries.push(Rc::new(AppEntry{
                name: opt.name.to_case(Title),
                path: "optional".to_string(),
                cmd: [
                  cmd.clone(),
                  opt.args.clone().unwrap_or("".to_string())
                ].join(" "),
              }));
              info!("found cmd {}", &cmd);
            },
            None => {
              app_entries.push(Rc::new(AppEntry{
                name: opt.name.to_case(Title),
                path: "optional".to_string(),
                cmd: [
                  ron.cmd.clone(),
                  opt.args.clone().unwrap_or("".to_string())
                ].join(" "),
              }));
              info!("found no cmd");
            },
          }
        },
        None => info!("no additional entries to add"),
      }

      app_cat.push(Rc::new(AppConfig{
        conf: Rc::new(ron),
        entries: app_entries,
      }))
    }
  }

  Ok(app_cat)
}
