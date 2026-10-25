//! Signed-release model distribution is future work. F2 imports a locally
//! prepared, checksum-verified OPUS-MT EN->ID CTranslate2 pack only.
//! No downloads, Python, conversion, network or untrusted DLL loading at runtime.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs::{self, File}, io::Read, path::{Path, PathBuf}};

pub const MODEL_ID: &str = "Helsinki-NLP/opus-mt-en-id";
pub const LANGUAGE_PAIR: &str = "en-id";
const REQUIRED: &[&str] = &["model.bin", "config.json", "source.spm", "target.spm"];
const OPTIONAL: &[&str] = &["shared_vocabulary.json", "source_vocabulary.json", "target_vocabulary.json"];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub model_id: String,
    pub language_pair: String,
    pub format: String,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    Missing,
    InvalidManifest,
    UntrustedPath,
    CorruptFile,
    Io,
}
impl ModelError {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Missing => "model_missing",
            Self::InvalidManifest => "model_manifest_invalid",
            Self::UntrustedPath => "model_path_invalid",
            Self::CorruptFile => "model_hash_mismatch",
            Self::Io => "model_io_error",
        }
    }
}

pub fn default_pack_dir() -> Result<PathBuf, ModelError> {
    let app_data = std::env::var_os("LOCALAPPDATA").ok_or(ModelError::Missing)?;
    if app_data.is_empty() {
        return Err(ModelError::Missing);
    }
    Ok(PathBuf::from(app_data).join("ClipLingo").join("models").join("en-id"))
}

fn is_allowed(name: &str) -> bool {
    REQUIRED.contains(&name) || OPTIONAL.contains(&name)
}

pub fn verify_pack(dir: &Path) -> Result<Manifest, ModelError> {
    if !dir.is_dir() { return Err(ModelError::Missing); }
    let manifest_path = dir.join("manifest.json");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|_| ModelError::Missing)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 16 * 1024 {
        return Err(ModelError::InvalidManifest);
    }
    let bytes = fs::read(manifest_path).map_err(|_| ModelError::Io)?;
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|_| ModelError::InvalidManifest)?;
    if manifest.schema != 1 || manifest.model_id != MODEL_ID ||
        manifest.language_pair != LANGUAGE_PAIR || manifest.format != "ctranslate2-int8" ||
        manifest.files.len() < REQUIRED.len() || manifest.files.len() > REQUIRED.len() + OPTIONAL.len() ||
        REQUIRED.iter().any(|name| !manifest.files.contains_key(*name)) {
        return Err(ModelError::InvalidManifest);
    }
    for (name, expected_hash) in &manifest.files {
        if !is_allowed(name) || expected_hash.len() != 64 ||
           !expected_hash.as_bytes().iter().all(|b| b.is_ascii_hexdigit()) {
            return Err(ModelError::InvalidManifest);
        }
        let path = dir.join(name);
        let metadata = fs::symlink_metadata(&path).map_err(|_| ModelError::CorruptFile)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() == 0 ||
            metadata.len() > 1_000_000_000 {
            return Err(ModelError::UntrustedPath);
        }
        let mut file = File::open(path).map_err(|_| ModelError::Io)?;
        let mut hasher = Sha256::new();
        let mut block = [0u8; 64 * 1024];
        loop {
            let count = file.read(&mut block).map_err(|_| ModelError::Io)?;
            if count == 0 { break; }
            hasher.update(&block[..count]);
        }
        let hash = format!("{:x}", hasher.finalize());
        if !hash.eq_ignore_ascii_case(expected_hash) { return Err(ModelError::CorruptFile); }
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn unique_dir() -> PathBuf {
        std::env::temp_dir().join(format!("cliplingo-pack-test-{}-{}", std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()))
    }
    fn fixture(path: &Path) {
        fs::create_dir_all(path).unwrap();
        let mut files = BTreeMap::new();
        for name in REQUIRED {
            let bytes = format!("fixture:{name}");
            fs::write(path.join(name), bytes.as_bytes()).unwrap();
            files.insert((*name).to_owned(), format!("{:x}", Sha256::digest(bytes.as_bytes())));
        }
        let manifest = Manifest {schema:1,model_id:MODEL_ID.into(),language_pair:LANGUAGE_PAIR.into(),
            format:"ctranslate2-int8".into(),files};
        fs::write(path.join("manifest.json"),serde_json::to_vec(&manifest).unwrap()).unwrap();
    }
    #[test]
    fn missing_and_modified_pack_fail_closed() {
        let root=unique_dir();
        assert_eq!(verify_pack(&root).unwrap_err(),ModelError::Missing);
        fixture(&root);
        assert_eq!(verify_pack(&root).unwrap().language_pair,LANGUAGE_PAIR);
        fs::write(root.join("model.bin"),b"tampered").unwrap();
        assert_eq!(verify_pack(&root).unwrap_err(),ModelError::CorruptFile);
        fs::remove_dir_all(&root).unwrap();
    }
    #[test]
    fn reject_manifest_path_traversal_and_unexpected_files() {
        let root=unique_dir();fixture(&root);
        let mut manifest=verify_pack(&root).unwrap();
        manifest.files.insert("../outside".into(),"a".repeat(64));
        fs::write(root.join("manifest.json"),serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert_eq!(verify_pack(&root).unwrap_err(),ModelError::InvalidManifest);
        fs::remove_dir_all(root).unwrap();
    }
}
