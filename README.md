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
  cmd: "firefox",
  args: "-P",
  profile_dirs: [
    "~/.mozilla/firefox/"
  ],
  // regex to get display name from file. it will be coverted to title case, e.g.: a1s23df4.default -> Default
  profile_regex: r"^(?:.*\d+.*)\.(\S+)$", // "(?:.*\\d+.*))\\.(\\S+)$"
  // other entries to add
  opt_entries: Some([
    (
      name: "Manage Profiles",
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

until 0.2.0 & release:
- [ ] 🤪 get it working as a plugin

until 1.0.0:
- [ ] 🎓 nicer control flow & error handling (match, collect, map, etc.)
- [ ] 🔧 nicer logging & optional verbose logging
- [ ] 🏇 proper resource handling (less cloning, more Rc, Cow, Box and shit)

nice to have:
- [ ] 👥 when installed system-wide, also scan user config dirs
- [ ] 🗑️ ditch smol for tokio (as the pop/cosmmic guys did)


### Further Reading:

- [Pop Launcher ReadMe](https://github.com/pop-os/launcher/blob/master/README.md)
- [Launcher Rust Docs](https://docs.rs/pop-launcher/latest/pop_launcher/)
- [Ron - Rusty Object Notation](https://github.com/ron-rs/ron)
