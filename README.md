# App Profiles

A **Pop!_OS Launcher Plugin** to launch **Apps with Profiles** written in Rust.


## Usage

- Start Launcher
- Type *2_char_app_shorthand* followed by `space`
  - Pick a listed profile
- Type *profile_name* + `enter`

![Example Firefox Gif](docs/example_firefox.gif)


## Installation Directories

- User-local plugin: `~/.local/share/pop-launcher/plugins/app-profiles/`
- System-wide install: `/etc/pop-launcher/plugins/app-profiles/`


## Configuration

In `app-profiles/config` create an `{app}.ron` file for each app you want to launch with multiple profiles/workspaces or be started with different files and/or sets of arguments.


### Example: default **firefox.ron**

```rust
( // class name leads to error
  // shorthand to trigger search and view of profiles
  shorthand: "ff",
  cmd: "/usr/bin/firefox", // "firefox"
  args: "-P",
  profile_dirs: [
    "~/.mozilla/firefox"
  ],
  // if profile_filename is given, profile_regex will be applied to the file contents instead of the filenames of the files in profile_dirs.
  profile_filename: Some("profiles.ini"),
  // regex to match profiles or profile names in file or directory - capture name with group!
  profile_regex: r"\[Profile\d+\]\nName=(.+)\n", // "\\[Profile\\d+\\]\\nName=(.+)\\n"
  // other entries to add
  opt_entries: Some([
    (
      name: "Manage Profiles",
      desc: Some("Manage Firefox Profiles"),
      // will use std cmd
      cmd: None,
      args: Some("-ProfileManager"),
    ),
  ])
  // icon name (if standard) or path
  icon: Some("firefox"),
)
```

See [Usage](#usage) for this example in action.


## Roadmap 🚀

until 1.0.0:
- [ ] 😎 fuzzyfind app entries
- [ ] 🤖 impl. autocompletion
- [ ] 🎓 make it rustier (nicer control flow & error handling, less cloning)
- [ ] 🗑️ ditch smol for tokio (as the pop/cosmmic guys did)
- [ ] 🚀 override regex in plugin.ron depending on app configs
- [ ] 🔧 integrate new pop launcher standard logging
- [ ] 🏇 proper resource handling (less cloning, more Rc, Cow, Box and shit)
- [ ] 👥 when installed system-wide, also scan user config dirs

future / nice to have:
- some basic concurrency
- find a way to integrate with open window list in launcher &
  display profile name on open window entry in launcher
- firefox specific: manage profiles from launcher:
  - create new
  - delete old
  - rename
  - copy existing
- vscode specific: add/remove available workspaces from launcher


### Further Reading:

- [Pop Launcher ReadMe](https://github.com/pop-os/launcher/blob/master/README.md)
- [Launcher Rust Docs](https://docs.rs/pop-launcher/latest/pop_launcher/)
- [Ron - Rusty Object Notation](https://github.com/ron-rs/ron)
