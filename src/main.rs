use crate::pack::{ModPack, ModMetadata, ModUpdateMetadata};
use structopt::StructOpt;
use std::path::Path;
use crate::util::{error, complete, warning, hash_from_url, info};
use crate::sources::curseforge::CurseforgeClient;
use crate::sources::modrinth::ModrinthClient;
use crate::sources::github::{GithubClient, get_github_token};
use dialoguer::{Input, Select};
use dialoguer::theme::ColorfulTheme;

mod pack;
mod util;
mod sources;
mod download;

#[derive(StructOpt, Debug)]
#[structopt(name = "pack-it")]
enum Opt {
    #[structopt(help = "initialise a pack.toml file")]
    Init,
    // #[structopt(help = "generate a test pack.toml file")]
    // GenTest,

    #[structopt(help = "add a mod from CurseForge to the pack")]
    CurseforgeAdd {
        mod_identifiers: Vec<String>,
    },

    #[structopt(help = "add a mod from Modrinth to the pack")]
    ModrinthAdd {
        #[structopt(long, short, help = "use the staging instance of the modrinth api")]
        staging: bool,
        mod_identifiers: Vec<String>,
    },

    #[structopt(help = "add a mod from Github to the pack")]
    GithubAdd {
        owner: String,
        repo: String,
        tag: String,
    },

    #[structopt(help = "Download all mods specified in pack.toml")]
    DownloadMods,

    #[structopt(help = "remove mods from the pack")]
    Remove {
        mods: Vec<String>,
    },

    // #[structopt(help = "test curseforge lookup")]
    // CFTest,
    // #[structopt(help = "test modrinth lookup")]
    // MRTest,
}

async fn init_pack() -> anyhow::Result<()> {
    let pack_file_path = Path::new("pack.toml");
    if pack_file_path.exists() {
        error("A pack.toml file already exists in this directory!");
        return Ok(())
    }

    // let mut asker = QuestionAsker::new();
    let pack_name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Pack name")
        .interact_text()?;
    let pack_author = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Pack author")
        .interact_text()?;
    let supported_game_versions: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Game versions (separated with space or comma)")
        .interact_text()?;
    let mod_loaders = &["Fabric", "Forge"];
    let mod_loader = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Target modloader")
        .default(0)
        .items(&mod_loaders[..])
        .interact()?;
    let mod_loader = mod_loaders[mod_loader];
    let supported_game_versions: Vec<&str> = supported_game_versions.split(|c| c == ' ' || c == ',').collect();
    let supported_game_versions = supported_game_versions.iter().map(|s| s.to_string()).collect();
    let pack = ModPack::new(pack_name, pack_author, supported_game_versions, mod_loader.to_lowercase().to_string());

    if pack_file_path.exists() {
        error("A pack.toml file already exists in this directory!");
        return Ok(())
    }
    pack.save(pack_file_path)?;
    complete("Generated pack.toml!");

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt: Opt = Opt::from_args();

    match opt {
        Opt::Init => {
            util::print_hello();
            init_pack().await?;
        }

        // Opt::GenTest => {
        //     let pack = ModPack::create_test_pack();
        //     let path = std::path::Path::new("pack.toml");
        //     pack.save(path)?;
        // }
        // Opt::CFTest => {
        //     env_logger::init();
        //     let pack = ModPack::read(&mut std::fs::File::open("pack.toml")?)?;
        //     let cf_client = CurseforgeClient::new()?;
        //     let mod_data = cf_client.find_mod_by_slug(&"appleskin".to_string()).await?;
        //     println!("{:?}", pack.resolve_curseforge_version(&mod_data.files));
        // }
        // Opt::MRTest => {
        //     env_logger::init();
        //     let pack = ModPack::read(&mut std::fs::File::open("pack.toml")?)?;
        //     let mr_client = ModrinthClient::new(false)?;
        //     let mod_version = mr_client.resolve_mod(&"terra".to_string(), &|v| {
        //         return pack.supports(&v)
        //     }).await?;
        //     println!("{:?}", mod_version);
        // }

        Opt::CurseforgeAdd { mod_identifiers } => {
            let pack_path = std::path::Path::new("pack.toml");
            let mut pack = ModPack::read(&mut std::fs::File::open(pack_path)?)?;
            let cf_client = CurseforgeClient::new()?;
            for mod_slug in mod_identifiers {
                info(&*(format!("Resolving {}...", mod_slug)));
                let mod_data = cf_client.find_mod_by_slug(&mod_slug).await?;
                let version = pack.resolve_curseforge_version(&mod_data.files);
                if let Some(version) = version {
                    info(&*(format!("Hashing {}...", version.file_name)));
                    let hash = hash_from_url(&version.download_url).await?;
                    pack.add(ModMetadata {
                        name: mod_data.slug.clone(),
                        output_path: format!("./mods/{}", version.file_name),
                        download_url: version.download_url,
                        download_hash: hash,
                        update_info: Some(ModUpdateMetadata::Curseforge {
                            addon_id: mod_data.id,
                            file_id: version.id,
                        })
                    });
                    pack.save(&pack_path)?;
                    complete(&*format!("Added {} by {} to the pack!", mod_data.name, mod_data.format_authors()))
                } else {
                    warning(&*format!("No compatible version found for {}!", mod_slug))
                }
            }
        }

        Opt::ModrinthAdd { staging, mod_identifiers } => {
            let pack_path = std::path::Path::new("pack.toml");
            let mut pack = ModPack::read(&mut std::fs::File::open(pack_path)?)?;
            let mr_client = ModrinthClient::new(staging)?;
            for mod_id in mod_identifiers {
                info(&*(format!("Resolving {}...", mod_id)));
                let version = mr_client.resolve_mod(&mod_id, &|v| pack.supports(&v)).await?;
                if let Some((mod_data, version, file)) = version {
                    let update_metadata = if staging {
                        ModUpdateMetadata::Modrinth {
                            project_id: version.mod_id.clone(),
                            version_id: version.id,
                            staging: Some(true)
                        }
                    } else {
                        ModUpdateMetadata::Modrinth {
                            project_id: version.mod_id.clone(),
                            version_id: version.id,
                            staging: None
                        }
                    };

                    pack.add(ModMetadata {
                        name: mod_data.slug.clone(),
                        output_path: format!("./mods/{}", file.filename),
                        download_url: file.url,
                        download_hash: file.hashes.get("sha1").expect("Modrinth did not supply a sha1 hash").clone(),
                        update_info: Some(update_metadata)
                    });
                    pack.save(&pack_path)?;
                    complete(&*format!("Added {} to the pack!", mod_data.title))
                } else {
                    warning(&*format!("No compatible version found for {}!", mod_id))
                }
            }
        }

        Opt::GithubAdd { owner, repo, tag } => {
            let pack_path = std::path::Path::new("pack.toml");
            let mut pack = ModPack::read(&mut std::fs::File::open(pack_path)?)?;

            info(&*format!("Resolving {}/{}:{}...", owner, repo, tag));

            match GithubClient::new(get_github_token()).resolve_mod(&*owner, &*repo, &*tag).await? {
                None => warning(&*format!("No valid file found for {}/{}:{}", owner, repo, tag)),
                Some(asset) => {
                    info(&*format!("Hashing {}...", asset.name));
                    let hash = hash_from_url(&asset.browser_download_url.to_string()).await?;

                    pack.add(ModMetadata {
                        name: repo.clone(),
                        output_path: format!("./mods/{}", asset.name),
                        download_url: asset.browser_download_url.to_string(),
                        download_hash: hash,
                        update_info: Some(ModUpdateMetadata::GitHub {
                            owner,
                            repo,
                            tag,
                        })
                    });
                    pack.save(&pack_path)?;

                    complete(asset.browser_download_url.as_str())
                },
            }
        }

        Opt::DownloadMods { } => {
            let pack_path = std::path::Path::new("pack.toml");
            let pack = ModPack::read(&mut std::fs::File::open(pack_path)?)?;
            let downloader = download::Downloader::new();
            for mod_metadata in pack.get_mods() {
                info(&*format!("Processing {}...", mod_metadata.name));
                let path = std::path::Path::new(&mod_metadata.output_path);
                downloader.download_if_hash_invalid(path, &mod_metadata.download_url, &mod_metadata.download_hash).await?;
            }
        }

        Opt::Remove { mods } => {
            let pack_path = std::path::Path::new("pack.toml");
            let mut pack = ModPack::read(&mut std::fs::File::open(pack_path)?)?;
            let mut count = 0;
            for mod_name in mods {
                if !pack.remove(&mod_name) {
                    error(&*format!("No mod in pack called {}!", mod_name));
                } else {
                    info(&*format!("Removed {} from the pack!", mod_name));
                    count += 1;
                }
            }
            pack.save(&pack_path)?;
            complete(&*format!("Removed {} mods from the pack!", count));
        }
    }

    Ok(())
}
