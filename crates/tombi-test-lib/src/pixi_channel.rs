//! A local conda channel served over HTTP for pixi extension tests.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
    sync::{Arc, LazyLock, Mutex},
};

use rattler_conda_types::{
    PackageName, PackageRecord, RepodataRevisions, Shard, ShardedRepodata, ShardedSubdirInfo,
    Version, package::BuildString, package::DistArchiveIdentifier,
};

/// The package served by every [`PixiTestChannel`].
pub const PIXI_TEST_PACKAGE: &str = "tombi-pixi-test";

const SUBDIRS: &[&str] = &["noarch", "linux-64", "osx-arm64", "osx-64", "win-64"];

pub struct PixiTestChannel {
    url: String,
    requested_paths: Arc<Mutex<Vec<String>>>,
}

impl PixiTestChannel {
    /// A channel that serves sharded repodata (CEP-16) only.
    pub fn sharded() -> &'static Self {
        static CHANNEL: LazyLock<PixiTestChannel> =
            LazyLock::new(|| PixiTestChannel::start(sharded_files()));
        &CHANNEL
    }

    /// A channel that serves only a full `repodata.json` and no shard index.
    pub fn without_shards() -> &'static Self {
        static CHANNEL: LazyLock<PixiTestChannel> =
            LazyLock::new(|| PixiTestChannel::start(full_repodata_files()));
        &CHANNEL
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Replaces `{{CHANNEL}}` in the given `pixi.toml` source with this channel's URL.
    pub fn pixi_toml(&self, source: &str) -> String {
        source.replace("{{CHANNEL}}", &self.url)
    }

    pub fn requested_paths(&self) -> Vec<String> {
        self.requested_paths
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
    }

    fn start(files: HashMap<String, Vec<u8>>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let requested_paths = Arc::new(Mutex::new(Vec::new()));

        let paths = Arc::clone(&requested_paths);
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let _ = serve(stream, &files, &paths);
            }
        });

        Self {
            url,
            requested_paths,
        }
    }
}

fn serve(
    mut stream: TcpStream,
    files: &HashMap<String, Vec<u8>>,
    requested_paths: &Mutex<Vec<String>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 || header == "\r\n" {
            break;
        }
    }

    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default()
        .to_string();
    requested_paths
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .push(path.clone());

    match files.get(&path) {
        Some(body) => {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nCache-Control: max-age=3600\r\nConnection: close\r\n\r\n",
                body.len()
            )?;
            stream.write_all(body)?;
        }
        None => {
            stream.write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )?;
        }
    }
    stream.flush()
}

/// Versions of [`PIXI_TEST_PACKAGE`] per subdir.
fn package_versions(subdir: &str) -> &'static [&'static str] {
    match subdir {
        "noarch" => &["1.0.0", "1.2.0", "1.10.0", "2.0.0"],
        "linux-64" => &["0.9.0", "1.10.0"],
        _ => &[],
    }
}

fn package_records(subdir: &str) -> Vec<(DistArchiveIdentifier, PackageRecord)> {
    package_versions(subdir)
        .iter()
        .map(|version| {
            let mut record = PackageRecord::new(
                PackageName::from_str(PIXI_TEST_PACKAGE).unwrap(),
                Version::from_str(version).unwrap(),
                BuildString::from_str("0").unwrap(),
            );
            record.subdir = subdir.to_string();
            let identifier =
                DistArchiveIdentifier::from_str(&format!("{PIXI_TEST_PACKAGE}-{version}-0.conda"))
                    .unwrap();
            (identifier, record)
        })
        .collect()
}

fn sharded_files() -> HashMap<String, Vec<u8>> {
    let mut files = HashMap::new();
    for subdir in SUBDIRS {
        let mut shards: Vec<(String, rattler_digest::Sha256Hash)> = Vec::new();
        let records = package_records(subdir);
        if !records.is_empty() {
            let mut shard = Shard::default();
            shard.conda_packages.extend(records);
            let shard_bytes =
                zstd::encode_all(rmp_serde::to_vec_named(&shard).unwrap().as_slice(), 3).unwrap();
            let shard_hash =
                rattler_digest::compute_bytes_digest::<rattler_digest::Sha256>(&shard_bytes);
            let shard_hex: String = shard_hash
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            files.insert(
                format!("/{subdir}/shards/{shard_hex}.msgpack.zst"),
                shard_bytes,
            );
            shards.push((PIXI_TEST_PACKAGE.to_string(), shard_hash));
        }

        let index = ShardedRepodata {
            info: ShardedSubdirInfo {
                subdir: subdir.to_string(),
                base_url: "./".to_string(),
                shards_base_url: "./shards/".to_string(),
                created_at: None,
                repodata_revisions: RepodataRevisions::default(),
                channel_relations: None,
                virtual_package_detectors: None,
            },
            shards: shards.into_iter().collect(),
        };
        files.insert(
            format!("/{subdir}/repodata_shards.msgpack.zst"),
            zstd::encode_all(rmp_serde::to_vec_named(&index).unwrap().as_slice(), 3).unwrap(),
        );
    }
    files
}

fn full_repodata_files() -> HashMap<String, Vec<u8>> {
    SUBDIRS
        .iter()
        .map(|subdir| {
            let packages = package_records(subdir)
                .into_iter()
                .map(|(identifier, record)| {
                    format!(
                        r#""{identifier}": {{"name": "{}", "version": "{}", "build": "0", "build_number": 0, "subdir": "{subdir}", "depends": []}}"#,
                        record.name.as_source(),
                        record.version,
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            (
                format!("/{subdir}/repodata.json"),
                format!(r#"{{"info": {{"subdir": "{subdir}"}}, "packages": {{}}, "packages.conda": {{{packages}}}}}"#)
                    .into_bytes(),
            )
        })
        .collect()
}
