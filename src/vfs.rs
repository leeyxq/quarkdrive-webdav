use std::fmt::{Debug, Formatter};
use std::io::{SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bytes::{Buf, Bytes, BytesMut};
use dashmap::DashMap;
use dav_server::{
    davpath::DavPath,
    fs::{
        DavDirEntry, DavFile, DavFileSystem, DavMetaData, FsError, FsFuture, FsStream, OpenOptions,
        ReadDirMeta,
    },
};
use futures_util::future::{ready, FutureExt};
use tracing::{debug, error, trace};
use crate::{
    cache::Cache,
    drive::{QuarkDrive, QuarkFile},
};

#[derive(Clone)]
pub struct QuarkDriveFileSystem {
    drive: QuarkDrive,
    pub(crate) dir_cache: Cache,
    uploading: Arc<DashMap<String, Vec<QuarkFile>>>,
    root: PathBuf,
    no_trash: bool,
    read_only: bool,
    upload_buffer_size: usize,
    skip_upload_same_size: bool,
    prefer_http_download: bool,
}

impl QuarkDriveFileSystem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(drive: QuarkDrive, root: String, cache_size: u64, cache_ttl: u64) -> Result<Self> {
        let dir_cache = Cache::new(cache_size, cache_ttl, drive.clone());
        debug!("dir cache initialized");
        let root = if root.starts_with('/') {
            PathBuf::from(root)
        } else {
            Path::new("/").join(root)
        };
        Ok(Self {
            drive,
            dir_cache,
            uploading: Arc::new(DashMap::new()),
            root,
            no_trash: false,
            read_only: false,
            upload_buffer_size: 16 * 1024 * 1024,
            skip_upload_same_size: false,
            prefer_http_download: false,
        })
    }

    pub fn set_read_only(&mut self, read_only: bool) -> &mut Self {
        self.read_only = read_only;
        self
    }

    pub fn set_no_trash(&mut self, no_trash: bool) -> &mut Self {
        self.no_trash = no_trash;
        self
    }

    pub fn set_upload_buffer_size(&mut self, upload_buffer_size: usize) -> &mut Self {
        self.upload_buffer_size = upload_buffer_size;
        self
    }

    pub fn set_skip_upload_same_size(&mut self, skip_upload_same_size: bool) -> &mut Self {
        self.skip_upload_same_size = skip_upload_same_size;
        self
    }

    pub fn set_prefer_http_download(&mut self, prefer_http_download: bool) -> &mut Self {
        self.prefer_http_download = prefer_http_download;
        self
    }

    async fn find_in_cache(&self, path: &Path) -> Result<Option<QuarkFile>, FsError> {
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            let file_name = path
                .file_name()
                .ok_or(FsError::NotFound)?
                .to_string_lossy()
                .into_owned();
            let file = self.dir_cache.get_or_insert(&parent_str).await.and_then(|files| {
                for file in &files {
                    if file.file_name == file_name {
                        return Some(file.clone());
                    }
                }
                None
            });
            Ok(file)
        } else {
            let root = QuarkFile::new_root();
            Ok(Some(root))
        }
    }

    async fn get_file(&self, path: PathBuf) -> Result<Option<QuarkFile>, FsError> {
        let file = self.find_in_cache(&path).await?;
        if let Some(file) = file {
            trace!(path = %path.display(), file_id = %file.fid, "file found in cache");
            Ok(Some(file))
        } else {
            // find in drive
            Ok(None)
        }
    }


    fn normalize_dav_path(&self, dav_path: &DavPath) -> PathBuf {
        let path = dav_path.as_pathbuf();
        if self.root.parent().is_none() || path.starts_with(&self.root) {
            return path;
        }
        let rel_path = dav_path.as_rel_ospath();
        if rel_path == Path::new("") {
            return self.root.clone();
        }
        self.root.join(rel_path)
    }
}

impl DavFileSystem for QuarkDriveFileSystem {
    fn open<'a>(
        &'a self,
        dav_path: &'a DavPath,
        options: OpenOptions,
    ) -> FsFuture<'a, Box<dyn DavFile>> {
        let path = self.normalize_dav_path(dav_path);
        let mode = if options.write { "write" } else { "read" };
        debug!(path = %path.display(), mode = %mode, "fs: open");
        async move {
            if options.append {
                // Can't support open in write-append mode
                error!(path = %path.display(), "unsupported write-append mode");
                return Err(FsError::NotImplemented);
            }
            let parent_path = path.parent().ok_or(FsError::NotFound)?;
            let parent_file = self
                .get_file(parent_path.to_path_buf())
                .await?
                .ok_or(FsError::NotFound)?;
            let sha1 = options.checksum.and_then(|c| {
                if let Some((algo, hash)) = c.split_once(':') {
                    if algo.eq_ignore_ascii_case("sha1") {
                        Some(hash.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            });
            let mut dav_file = if let Some(file) = self.get_file(path.clone()).await? {
                if options.write && options.create_new {
                    return Err(FsError::Exists);
                }
                if options.write && self.read_only {
                    return Err(FsError::Forbidden);
                }
                QuarkDavFile::new(
                    self.clone(),
                    file,
                    parent_file.fid,
                    parent_path.to_path_buf(),
                    options.size.unwrap_or_default(),
                    sha1,
                )
            } else {
                return Err(FsError::NotFound);
            };
            dav_file.http_download = self.prefer_http_download;
            Ok(Box::new(dav_file) as Box<dyn DavFile>)
        }
            .boxed()
    }

    fn read_dir<'a>(
        &'a self,
        path: &'a DavPath,
        _meta: ReadDirMeta,
    ) -> FsFuture<'a, FsStream<Box<dyn DavDirEntry>>> {
        let path = self.normalize_dav_path(path);
        debug!(path = %path.display(), "fs: read_dir");
        async move {
            let files = self.dir_cache.get_or_insert(&path.to_string_lossy())
                .await
                .ok_or(FsError::NotFound)
                .and_then(|files| {
                    if files.is_empty() {
                        Err(FsError::NotFound)
                    } else {
                        Ok(files)
                    }
                })?;

            // 创建包含结果的向量
            let mut v: Vec<Result<Box<dyn DavDirEntry>, FsError>> = Vec::with_capacity(files.len());

            // 将每个文件转换为 trait 对象
            for file in files {
                v.push(Ok(Box::new(file))); // 现在类型匹配了
            }

            // 创建流并装箱
            let stream = futures_util::stream::iter(v);
            Ok(Box::pin(stream) as FsStream<Box<dyn DavDirEntry>>)
        }
            .boxed()
    }

    fn metadata<'a>(&'a self, path: &'a DavPath) -> FsFuture<'a, Box<dyn DavMetaData>> {
        let path = self.normalize_dav_path(path);
        debug!(path = %path.display(), "fs: metadata");
        async move {
            let file = self.get_file(path).await?.ok_or(FsError::NotFound)?;
            Ok(Box::new(file) as Box<dyn DavMetaData>)
        }
            .boxed()
    }
    fn have_props<'a>(
        &'a self,
        _path: &'a DavPath,
    ) -> std::pin::Pin<Box<dyn futures_util::Future<Output = bool> + Send + 'a>> {
        Box::pin(ready(true))
    }

    fn get_prop(&self, dav_path: &DavPath, prop: dav_server::fs::DavProp) -> FsFuture<Vec<u8>> {
        let path = self.normalize_dav_path(dav_path);
        let prop_name = match prop.prefix.as_ref() {
            Some(prefix) => format!("{}:{}", prefix, prop.name),
            None => prop.name.to_string(),
        };
        debug!(path = %path.display(), prop = %prop_name, "fs: get_prop");
        async move {
            Err(FsError::NotImplemented)
        }
            .boxed()
    }

    fn get_quota(&self) -> FsFuture<(u64, Option<u64>)> {
        debug!("fs: get_quota");
        async move {
            Err(FsError::NotImplemented)
        }
            .boxed()
    }
}

#[derive(Debug, Clone)]
struct UploadState {
    size: u64,
    buffer: BytesMut,
    chunk_count: u64,
    chunk: u64,
    upload_id: String,
    upload_urls: Vec<String>,
    sha1: Option<String>,
}

impl Default for UploadState {
    fn default() -> Self {
        Self {
            size: 0,
            buffer: BytesMut::new(),
            chunk_count: 0,
            chunk: 1,
            upload_id: String::new(),
            upload_urls: Vec::new(),
            sha1: None,
        }
    }
}

struct QuarkDavFile {
    fs: QuarkDriveFileSystem,
    file: QuarkFile,
    parent_file_id: String,
    parent_dir: PathBuf,
    current_pos: u64,
    upload_state: UploadState,
    http_download: bool,
    
}

impl Debug for QuarkDavFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuarkDavFile")
            .field("file", &self.file)
            .field("parent_file_id", &self.parent_file_id)
            .field("current_pos", &self.current_pos)
            .field("upload_state", &self.upload_state)
            .finish()
    }
}

impl QuarkDavFile {
    
    fn new(
        fs: QuarkDriveFileSystem,
        file: QuarkFile,
        parent_file_id: String,
        parent_dir: PathBuf,
        size: u64,
        sha1: Option<String>,
    ) -> Self {
        Self {
            fs,
            file,
            parent_file_id,
            parent_dir,
            current_pos: 0,
            upload_state: UploadState {
                size,
                sha1,
                ..Default::default()
            },
            http_download: false,
        }
    }

    async fn get_download_url(&self) -> Result<String, FsError> {
        self.fs.drive.get_download_url(&self.file.fid).await.map_err(|err| {
            error!(file_id = %self.file.fid, file_name = %self.file.file_name, error = %err, "get download url failed");
            FsError::GeneralFailure
        })
    }

}

impl DavFile for QuarkDavFile {
    fn metadata(&'_ mut self) -> FsFuture<'_, Box<dyn DavMetaData>> {
        debug!(file_id = %self.file.fid, file_name = %self.file.file_name, "file: metadata");
        async move {
            let file = self.file.clone();
            Ok(Box::new(file) as Box<dyn DavMetaData>)
        }
            .boxed()
    }

    fn redirect_url(&mut self) -> FsFuture<Option<String>> {
        debug!(file_id = %self.file.fid, file_name = %self.file.file_name, "file: redirect_url");
        async move {
            if self.file.fid.is_empty() {
                return Err(FsError::NotFound);
            }
            let download_url = self.fs.drive.get_download_url(&self.file.fid).await.unwrap();

            return Ok(Some(download_url));
            
        }
            .boxed()
    }
    


    fn seek(&mut self, pos: SeekFrom) -> FsFuture<u64> {
        debug!(
            file_id = %self.file.fid,
            file_name = %self.file.file_name,
            pos = ?pos,
            "file: seek"
        );
        async move {
            let new_pos = match pos {
                SeekFrom::Start(pos) => pos,
                SeekFrom::End(pos) => (self.file.size as i64 + pos) as u64,
                SeekFrom::Current(size) => self.current_pos + size as u64,
            };
            self.current_pos = new_pos;
            Ok(new_pos)
        }
            .boxed()
    }

    fn write_buf(&mut self, buf: Box<dyn Buf + Send>) -> FsFuture<()> {
        todo!()
    }

    fn write_bytes(&mut self, buf: Bytes) -> FsFuture<()> {
        todo!()
    }

    fn read_bytes(&mut self, count: usize) -> FsFuture<Bytes> {
        debug!(
            file_id = %self.file.fid,
            file_name = %self.file.file_name,
            pos = self.current_pos,
            count = count,
            size = self.file.size,
            "file: read_bytes",
        );
        async move {
            if self.file.fid.is_empty() {
                // upload in progress
                return Err(FsError::NotFound);
            }
            // 检查现有 URL 是否有效
            let is_valid = self.file.download_url.as_ref()
                .map(|url| !is_url_expired(url))
                .unwrap_or(false);

            if !is_valid {
                let new_url = self.get_download_url().await.unwrap();
                self.file.download_url = Some(new_url);
            }
            let download_url = match self.file.download_url.as_ref() {
                Some(url) => url,
                None => {
                        // 详细记录文件信息
                        println!(
                        "文件缺少下载URL: {:?}\n文件元数据: {:#?}",
                        self.file.download_url,
                        self.file);
                    return Err(dav_server::fs::FsError::NotFound);
                }
            };

            if !download_url.is_empty() {
                let content = self.fs.drive.download(download_url, Some((self.current_pos, count))).await.unwrap();
                self.current_pos += content.len() as u64;
                return Ok(content);
            }else {
                return Err(FsError::NotFound);
            }
        }
            .boxed()
    }

    fn flush(&mut self) -> FsFuture<()> {
        todo!()
    }
}



fn is_url_expired(url: &str) -> bool {
    if let Ok(oss_url) = ::url::Url::parse(url) {
        let expires = oss_url.query_pairs().find_map(|(k, v)| {
            if k == "Expires" {
                if let Ok(expires) = v.parse::<u64>() {
                    return Some(expires);
                }
            }
            None
        });
        if let Some(expires) = expires {
            let current_ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();
            // 预留 1 分钟
            return current_ts >= expires - 60;
        }
    }
    false
}