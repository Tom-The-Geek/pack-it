use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::fs;
use anyhow::Result;
use std::path::Path;
use std::io::Read;
use std::fmt::Debug;
use crate::sources::curseforge::CurseforgeModFile;
use crate::sources::modrinth::ModrinthVersion;

#[derive(Serialize, Deserialize, Debug)]
pub struct ModPack {
    pack_name: Option<String>,
    author: Option<String>,
    accepted_game_versions: Vec<String>,
    mod_loader: String,

    installed_mods: HashMap<String, ModMetadata>,
}

impl ModPack {
    pub fn read(file: &mut File) -> Result<Self> {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(toml::from_str(&*content)?)
    }

    pub fn new(name: String, author: String, game_versions: Vec<String>, mod_loader: String) -> Self {
        Self {
            pack_name: Some(name),
            author: Some(author),
            installed_mods: HashMap::new(),
            accepted_game_versions: game_versions,
            mod_loader,
        }
    }

    // pub fn create_test_pack() -> Self {
    //     let mut mods = HashMap::new();
    //     mods.insert("modmenu".to_string(), ModMetadata {
    //         name: "modmenu".to_string(),
    //         download_hash: "fe5d944e2925a608babf73890e6423ee655872048ed122f0336c4c697024440e19146d04dda4c448a317273fb70578a393cf04e7cb6a33e85bc7ec7807b1fd15".to_string(),
    //         download_url: "https://cdn.modrinth.com/data/mOgUt4GM/versions/1.16.9/modmenu-1.16.9.jar".to_string(),
    //         output_path: "mods/modmenu-1.16.9.jar".to_string(),
    //         update_info: Some(ModUpdateMetadata::Modrinth {
    //             project_id: "mOgUt4GM".to_string(),
    //             version_id: "bPE0GIoY".to_string(),
    //             staging: None,
    //         })
    //     });
    //     mods.insert("fabric-api".to_string(), ModMetadata {
    //         name: "fabric-api".to_string(),
    //         download_hash: "6c8c64aaec7ebeaec583cb99398d5d1c5044459e8f8da5ea2cfcaada304a84ea0c097cc0ae17a08589f381f1f8b1ac44134880d36a41c15294c1698833b2b723".to_string(),
    //         download_url: "https://cdn.modrinth.com/data/P7dR8mSH/versions/0.34.2+1.16/fabric-api-0.34.2+1.16.jar".to_string(),
    //         output_path: "mods/fabric-api-0.34.2+1.16.jar".to_string(),
    //         update_info: None,
    //     });
    //     Self {
    //         pack_name: Some("Test Pack".to_string()),
    //         author: Some("Tom_The_Geek".to_string()),
    //         mod_loader: "fabric".to_string(),
    //         accepted_game_versions: vec!["1.16.5".to_string(), "1.16.4".to_string()],
    //         installed_mods: mods,
    //     }
    // }

    pub fn add(&mut self, mod_info: ModMetadata) {
        self.installed_mods.insert(mod_info.name.clone(), mod_info);
    }

    pub fn save(&self, pack_file: &Path) -> Result<()> {
        fs::write(pack_file, toml::to_string(&self)?)?;
        Ok(())
    }

    pub fn resolve_curseforge_version(&self, files: &[CurseforgeModFile]) -> Option<CurseforgeModFile> {
        let mut filtered = files.iter().filter(|&file| {
            return file.game_version.iter().map(|v| v.to_lowercase()).any(|v| v.eq(&self.mod_loader))
                && file.game_version.iter().any(|v| self.accepted_game_versions.contains(v));
        }).collect::<Vec<&CurseforgeModFile>>();
        filtered.sort_by(|&v1, &v2| v1.file_date.cmp(&v2.file_date));
        // this is rust so I have no idea what I am doing, but this appears to work
        filtered.last().cloned().cloned()
    }

    pub fn supports(&self, version: &ModrinthVersion) -> bool {
        return version.loaders.contains(&self.mod_loader)
            && version.game_versions.iter().any(|v| self.accepted_game_versions.contains(v))
    }

    pub fn get_mods(&self) -> Vec<ModMetadata> {
        let mut mods: Vec<ModMetadata> = vec![];
        for m in self.installed_mods.values() {
            mods.push(m.clone());
        }
        mods
    }

    pub fn remove(&mut self, mod_name: &str) -> bool {
        self.installed_mods.remove(mod_name).is_some()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModMetadata {
    pub name: String,
    pub download_url: String,
    pub download_hash: String,
    pub output_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_info: Option<ModUpdateMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ModUpdateMetadata {
    #[serde(rename = "cf")]
    Curseforge {
        addon_id: i32,
        file_id: i32,
    },
    #[serde(rename = "mr")]
    Modrinth {
        project_id: String,
        version_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        staging: Option<bool>,
    },
    #[serde(rename = "gh")]
    GitHub {
        owner: String,
        repo: String,
        tag: String,
    }
}
