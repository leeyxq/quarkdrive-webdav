use std::path::Path;
use std::time::Duration;

use moka::future::Cache as MokaCache;
use tracing::debug;
use crate::drive::{DriveConfig, QuarkDrive};
use crate::drive::model::QuarkFile;

#[derive(Clone)]
pub struct Cache {
    inner: MokaCache<String, Vec<QuarkFile>>,
    drive: QuarkDrive,
}

const ONE_PAGE: u32 = 3; 

impl Cache {
    pub fn new(max_capacity: u64, ttl: u64) -> Self {
        let inner = MokaCache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl))
            .build();

        let config = DriveConfig {
            api_base_url: "https://drive-pc.quark.cn".to_string(),
            cookie: Some(std::env::var("quark_cookie").unwrap()),
        };
        let drive = QuarkDrive::new(config).unwrap();

        Self { inner , drive}
    }

    pub async fn refresh_cache(&self) {
       // let files = self.drive.get_files_by_pdir_fid("0", 1, 50).await.unwrap();
        self.bfs(QuarkFile::new_root(), "").await;       
    }
    
    async fn bfs(&self, file: QuarkFile, path: &str) {
        if (file.dir) {
            let mut current_files = Vec::<QuarkFile>::new();
            for page_no in 1..=2 {
                let (files, total_page) = self.drive.get_files_by_pdir_fid(&file.fid, page_no, ONE_PAGE).await.unwrap();
                let files = files.unwrap();
                let size = files.list.len();
                current_files.extend(files.list);
                if size < ONE_PAGE as usize || page_no >= total_page   {
                    break;
                }
            }

            let p = if (path.ends_with("/") || file.file_name.starts_with("/")) {
                                format!("{}{}", path, file.file_name)
                            }else { 
                                format!("{}/{}", path, file.file_name)
                            };
            
            self.inner.insert(p.clone(), current_files.clone()).await;
            debug!("{} in cache", &p); 
            for f in current_files {
                Box::pin(self.bfs(f, &p)).await;

            }

          

        }

    }
    // async fn bfs0(&self, files: Vec<QuarkFile>, path: &str) {
    //     for file in files {
    //         let mut current_files = Vec::<QuarkFile>::new();
    //         if (file.dir) {
    //             for page_no in 1..=1000 {
    //                 let files = self.drive.get_files_by_pdir_fid(&file.fid, page_no, 50).await.unwrap().unwrap();
    //                 let size = files.list.len();
    //                 current_files.extend(files.list);
    //                 if size < 50 {
    //                     break;
    //                 }
    //             }
    //             
    //             let p = if file.pdir_fid != "0" {
    //                 format!("{}/{}", path, file.file_name)
    //             } else {
    //                 format!("/{}", file.file_name)
    //             };
    //             self.inner.insert(p.clone(), current_files.clone()).await;
    // 
    //             Box::pin(self.bfs(current_files, &p)).await;
    //            
    //         }
    //     }
    // }

    pub async fn get(&self, key: &str) -> Option<Vec<QuarkFile>> {
        debug!(key = %key, "cache: get");
        self.inner.get(key).await
    }

    pub async fn insert(&self, key: String, value: Vec<QuarkFile>) {
        debug!(key = %key, "cache: insert");
        self.inner.insert(key, value).await;
    }

    pub async fn invalidate(&self, path: &Path) {
        let key = path.to_string_lossy().into_owned();
        debug!(path = %path.display(), key = %key, "cache: invalidate");
        self.inner.invalidate(&key).await;
    }

    pub async fn invalidate_parent(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            self.invalidate(parent).await;
        }
    }

    pub fn invalidate_all(&self) {
        debug!("cache: invalidate all");
        self.inner.invalidate_all();
    }
}