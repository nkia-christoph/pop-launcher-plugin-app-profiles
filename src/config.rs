use convert_case::{Case::Title, Casing};
use glob::{glob_with, MatchOptions};
use regex::Regex;
use ron::de::from_reader;
use serde::Deserialize;
use std::{
    error::Error,
    fs::{read_dir, read_to_string, File},
    io::BufReader,
    path::{Path, PathBuf},
};

const CONFIG_PAT: &str = "config/*.ron";

#[derive(Deserialize, Debug, Clone)]
pub struct Ron {
    pub shorthand: String,
    cmd: String,
    args: String,
    profile_dirs: Vec<String>,
    profile_filename: Option<String>,

    #[serde(with = "serde_regex")]
    profile_regex: Regex,

    opt_entries: Option<Vec<OptEntry>>,
    pub icon: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OptEntry {
    name: String,
    desc: Option<String>,
    cmd: Option<String>,
    args: Option<String>,
}

pub type AppCatalogue = Vec<AppConfig>;
pub type AppEntries = Vec<AppEntry>;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub name: String,
    pub conf: Ron,
    pub entries: AppEntries,
}

#[derive(Debug, Clone)]
pub struct AppEntry {
    pub name: String,
    pub desc: String,
    pub cmd: String,
}

fn load_ron(file_path: &Path) -> Result<Ron, Box<dyn Error>> {
    info!("loading ron from {}", file_path.display());

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    Ok(from_reader(reader)?)
}

fn get_rons(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let glob_pat = path.join(CONFIG_PAT);
    info!("getting rons with glob: {}", glob_pat.display());

    let options = MatchOptions {
        case_sensitive: false,
        ..Default::default()
    };

    Ok(glob_with(glob_pat.to_string_lossy().as_ref(), options)?
        .flatten()
        .collect())
}

pub fn load_catalogue(path: &Path) -> Result<AppCatalogue, Box<dyn Error>> {
    let mut app_cat: AppCatalogue = Vec::new();

    for path in get_rons(path)? {
        info!("searching in path {}", path.display());

        let ron = load_ron(&path)?;
        let mut app_entries: AppEntries = Vec::new();

        for dir in &ron.profile_dirs {
            let abs_dir = expand_home(dir)?;
            info!("searching in dir {}", abs_dir.display());

            if let Some(profile_filename) = &ron.profile_filename {
                scan_profile_file(&mut app_entries, &ron, &path, &abs_dir, profile_filename)?;
                continue;
            }

            scan_profile_dir(&mut app_entries, &ron, &path, &abs_dir)?;
        }

        append_optional_entries(&mut app_entries, &ron);

        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or("invalid config filename")?
            .to_owned();

        app_cat.push(AppConfig {
            name,
            conf: ron,
            entries: app_entries,
        });
    }

    Ok(app_cat)
}

fn scan_profile_file(
    app_entries: &mut AppEntries,
    ron: &Ron,
    path: &Path,
    abs_dir: &Path,
    profile_filename: &str,
) -> Result<(), Box<dyn Error>> {
    info!("filename: {profile_filename}, scanning file...");

    let profile_file = read_to_string(abs_dir.join(profile_filename))?;
    for capture in ron.profile_regex.captures_iter(&profile_file) {
        let name = capture
            .get(1)
            .ok_or_else(|| {
                format!(
                    "no group matched for name! check regex in {}",
                    path.display()
                )
            })?
            .as_str();

        app_entries.push(AppEntry {
            name: name.to_case(Title),
            desc: String::new(),
            cmd: format!("{} {} {}", ron.cmd, ron.args, name),
        });
        info!("[OK] matched profilefile {name}");
    }

    Ok(())
}

fn scan_profile_dir(
    app_entries: &mut AppEntries,
    ron: &Ron,
    path: &Path,
    abs_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    for profile in read_dir(abs_dir)? {
        let profile = profile?;
        let file_name = profile.file_name();
        let file_name = file_name.to_string_lossy();

        info!("matching file {file_name}");

        if let Some(captures) = ron.profile_regex.captures(&file_name) {
            let name = captures
                .get(1)
                .ok_or_else(|| {
                    format!(
                        "no group matched for name! check regex in {}",
                        path.display()
                    )
                })?
                .as_str()
                .to_case(Title);

            let matched = captures
                .get(0)
                .ok_or("expected full regex capture")?
                .as_str();

            app_entries.push(AppEntry {
                name: name.clone(),
                desc: String::new(),
                cmd: format!("{} {} {}/{}", ron.cmd, ron.args, abs_dir.display(), matched),
            });

            info!(
                "[OK] matched file {}/{} as {}",
                abs_dir.display(),
                file_name,
                name
            );
        }
    }

    Ok(())
}

fn append_optional_entries(app_entries: &mut AppEntries, ron: &Ron) {
    match &ron.opt_entries {
        Some(optional) => {
            for opt in optional {
                info!("adding optional entry: {}", opt.name);

                let cmd = opt.cmd.as_deref().unwrap_or(&ron.cmd);
                let args = opt.args.as_deref().unwrap_or("");

                app_entries.push(AppEntry {
                    name: opt.name.to_case(Title),
                    desc: opt.desc.clone().unwrap_or_else(|| "optional".to_owned()),
                    cmd: format!("{} {}", cmd, args).trim().to_owned(),
                });
            }
        }
        None => info!("no additional entries to add"),
    }
}

fn expand_home(dir: &str) -> Result<PathBuf, Box<dyn Error>> {
    if let Some(stripped) = dir.strip_prefix("~/") {
        let home = dirs::home_dir().ok_or("home directory required")?;
        Ok(home.join(stripped))
    } else {
        Ok(PathBuf::from(dir))
    }
}
