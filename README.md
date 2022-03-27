# App Profiles

a **Pop!_OS Launcher Plugin** to handle multiple **App Profiles** written in Rust.


## Usage

- Start Launcher
- Type *2_char_app_shorthand* followed by `space`
  - Pick a listed profile
- Type *profile_name* + `enter`



## Installation Directories

- User-local plugin: `~/.local/share/pop-launcher/plugins/app-profiles/`
- System-wide install: `/etc/pop-launcher/plugins/app-profiles/`


## Configuration

Create [ron](https://github.com/ron-rs/ron) for each app that allows use of multiple profiles or workspaces or be started with different sets of arguments.


### Example: default **firefox.ron**

```rust
Firefox( // class name is optional
  cmd: "firefox",
  args: "-P",
  profileDirs: [
    "~/.mozilla/firefox/"
  ],
  // regex to get display name from file
  profileRegEx: ".*\\.(\\S+)$",
  // other entries to add
  addEntries: [
    (
      name: "Manage Profiles",
      args: "-ProfileManager",
    ),
  ]
  // icon name (if standard) or path
  icon: "firefox",
  // shorthand to trigger search and view of profiles
  shorthand: "ff",
)
```


### Further Reading:

- [Pop Launcher ReadMe](https://github.com/pop-os/launcher/blob/master/README.md)
- [Launcher Rust Docs](https://docs.rs/pop-launcher/latest/pop_launcher/)
