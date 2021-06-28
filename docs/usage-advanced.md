# Advanced usage

## Editing `pack.toml` manually

!!! warning
    Any comments added to the `pack.toml` file WILL BE LOST if you use any `pack-it` command that updates the file; eg. `*-add`, `remove`, etc.

### File layout

#### Modrinth metadata
```toml
# Basic metadata about the pack
pack_name = "Example Pack"
author = "Tom_The_Geek"
accepted_game_versions = ["1.16.5", "1.16.4"]
mod_loader = "fabric"

# Each mod is a value under installed_mods
[installed_mods.terra]
# The name of the mod, as used in console output. For mods from Modrinth, this will be the slug
name = "terra"
# The URL where the file can be downloaded
download_url = "https://cdn.modrinth.com/data/FIlZB9L0/versions/fabric-5.3.3-BETA+5dd00db8/Terra-fabric-5.3.3-BETA+5dd00db8-shaded-mapped.jar"
# The SHA-1 hash of the file to download
download_hash = "5ffed3a47cf09f192c52fb6476ad7bbca406794e"
# Where to save the file after it is downloaded
output_path = "./mods/Terra-fabric-5.3.3-BETA+5dd00db8-shaded-mapped.jar"

# Mods that are added from a supported service also attach some metadata about where they are from,
# so they can be updated automatically. This is not required, and can be omitted
[installed_mods.terra.update_info]
# This is always 'mr' for mods downloaded from Modrinth
type = "mr"
# The ID of the project on Modrinth
project_id = "FIlZB9L0"
# The ID of the particular version on Modrinth
version_id = "9DWPUHbr"
```

#### CurseForge metadata

All the fields are the same as those above, except for the `[installed_mods.<mod name>.update_info]`
```toml
[installed_mods.simplex-terrain-generation.update_info]
# This is always 'cf' for mods downloaded from CurseForge
type = "cf"
# The addon (mod) ID on CurseForge
addon_id = 352997
# The file ID on CurseForge
file_id = 3133994
```

#### GitHub metadata
All the fields are the same as those above, except for the `[installed_mods.<mod name>.update_info]`
```toml
[installed_mods.leukocyte.update_info]
# This is always 'gh' for mods downloaded from GitHub
type = "gh"
# This is the username or organisation name that owns the repository
owner = "NucleoidMC"
# This is the name of the repository to download the release from
repo = "leukocyte"
# This is the git tag that the release is tied to
tag = "v0.3.0"
```

## Adding other mods
Sometimes you may want to include a mod that isn't hosted on one of the supported services, and might want to manually add it. Fortunately this isn't too difficult, and some steps will be outlined below:

1. Find the download URL of the mod JAR, for example `https://example.com/mod-1.0.0.jar`
2. In order for pack-it to verify that the JAR has downloaded correctly, you need to calculate the SHA-1 hash of the file. A quick way to do this on linux is with the following command:
```bash
curl <download url> | sha1sum
```

    ??? question "What does that command do?"
        If you are curious as to what the command does does (good for you :D), then you can have a look at an explanation [on explainshell](https://explainshell.com/explain?cmd=curl+https%3A%2F%2Fexample.com+%7C+sha1sum)

    It will output something like the following:
    ```
    4a3ce8ee11e091dd7923f4d8c6e5b5e41ec7c047  -
    ```
    The first part of this (the 40 hex characters) is the hash that you need.

3. Add the following new section to your `pack.toml` file, being sure to replace all the values
```toml
[installed_mods.<mod name>]
name = "<mod name>"
download_url = "<download url>"
download_hash = "<hash that was calculated in step 2>"
output_path = "./mods/<file name>.jar"
```

## Adding things other than mods
pack-it can be used to automatically download other files as well as just your mods; eg. resource packs or config files.
As long as you can host the file on a webserver somewhere (eg. [GitHub Pages](https://pages.github.com) or [Vercel](https://vercel.com)), you can distribute them with the pack simply by including them as an `installed_mods`, with a different `output_path` set. See [Adding other mods](#adding-other-mods) for more details on the process
