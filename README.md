## pasuwado

a password management tool in the CLI

## Prerequisite
- [rust](https://rustup.rs/)

## Installing
```
cargo install --git https://github.com/jcesar-betrnk/pasuwado
```

## Usage

Adding an entry

```sh
pasuwado add --domain <DOMAIN> --user <USER> --password <PASSWORD>
pasuwado add --domain "github" --user jcesar-betrnk --password "hunter2"
```

Alternatively, you can directly edit the configuration file:
- Linux: `~/.config/pasuwado.toml`
- Windows: `{FOLDERID_RoamingAppData}/pasuwado.toml`
    ie: `<HOME>/AppData/Roaming/pasuwado/config.toml`
- Mac: `$HOME/Library/Application Support`

The format follows a `key` = `value` and line separated for each entry

Example:

```toml
[github]
"jwick" = "hunter2"
"invoker" = "cold forge"
"teburu@gmail.com" = "pwd123"
```

Getting the password from a domain and user combination would be:

```sh
pasuwado get --domain <DOMAIN> --user <USER>
pasuwado get --domain "github" --user "jwick"
```

The password for the domain with the user specified is then copied into your clipboard.


## Advance usage
For additional convenience, you can set a local directory in your machine which contains a separate
script for each specific domains

Example:
In your `~/scripts/pwd/` directory you'll create a file for each of the domain

```sh
ls -lah
```
```sh
fb.sh
gmail.sh
github.sh
```
Make sure all the script file as an executable permission:

```sh
cd ~/scripts/pwd
chmod u+x *.sh
```

```sh
cat github.sh
```

```sh
#!/bin/bash
pasuwado get --domain "github" --user "jwick"
```

