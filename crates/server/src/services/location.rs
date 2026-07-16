use itertools::Itertools;
use object_store::ObjectStoreScheme;
use unitycatalog_common::{Error, Result};
use url::Url;

pub enum StorageLocationScheme {
    ObjectStore(ObjectStoreScheme),
    Azurite,
}

impl AsRef<str> for StorageLocationScheme {
    fn as_ref(&self) -> &str {
        match self {
            Self::ObjectStore(ObjectStoreScheme::Local) => "file",
            Self::ObjectStore(ObjectStoreScheme::Memory) => "memory",
            Self::ObjectStore(ObjectStoreScheme::AmazonS3) => "s3",
            Self::ObjectStore(ObjectStoreScheme::GoogleCloudStorage) => "gs",
            Self::ObjectStore(ObjectStoreScheme::MicrosoftAzure) => "az",
            Self::ObjectStore(ObjectStoreScheme::Http) => "http",
            // NB: ObjectStoreScheme is non exhaustive, so we need to handle the unknown case.
            Self::ObjectStore(_) => "unknown",

            // Custom schemes
            Self::Azurite => "azurite",
        }
    }
}

impl StorageLocationScheme {
    pub fn parse(url: &Url) -> Result<Self> {
        match ObjectStoreScheme::parse(url) {
            Ok((ObjectStoreScheme::Http, _)) if is_azurite(url) => Ok(Self::Azurite),
            Ok((scheme, _)) => Ok(Self::ObjectStore(scheme)),
            Err(e) => {
                if is_azurite(url) {
                    Ok(Self::Azurite)
                } else {
                    Err(Error::invalid_argument(e.to_string()))
                }
            }
        }
    }
}

/// A URL representing a storage location.
///
/// This struct provides a cerntalized place to parse various URLs into more specific
/// service references along with the semantic information that can be extracted from
/// the URL.
pub struct StorageLocationUrl {
    /// The raw, unaltered URL.
    url: Url,
    /// The normalized `scheme://authority/` base of the location, used to derive
    /// [`location`](Self::location). A plain [`Url`] — this is only ever built
    /// from an authority string and read back as a URL/str.
    store_url: Url,
    scheme: StorageLocationScheme,
    location: Url,
}

impl StorageLocationUrl {
    pub fn try_new(url: Url) -> Result<Self> {
        let (store_url, scheme, location) = get_store_url(&url)?;
        Ok(Self {
            url,
            store_url,
            scheme,
            location,
        })
    }

    pub fn parse(url: impl AsRef<str>) -> Result<Self> {
        let url = Url::parse(url.as_ref())?;
        Self::try_new(url)
    }

    pub fn raw(&self) -> &Url {
        &self.url
    }

    pub fn location(&self) -> &Url {
        &self.location
    }

    pub fn store_url(&self) -> &Url {
        &self.store_url
    }

    pub fn scheme(&self) -> &StorageLocationScheme {
        &self.scheme
    }

    /// Returns `(bucket_or_container, object_prefix)` extracted from the URL.
    ///
    /// - S3: `s3://bucket/prefix/path` → `("bucket", "prefix/path")`
    /// - Azure HTTPS: `https://account.blob.core.windows.net/container/path` → `("container", "path")`
    /// - Azurite HTTP: `http://localhost:10000/account/container/path` → `("container", "path")`
    /// - Azurite custom scheme: `azurite://container/path` → `("container", "path")`
    pub fn bucket_and_prefix(&self) -> Result<(String, String)> {
        match &self.scheme {
            StorageLocationScheme::ObjectStore(ObjectStoreScheme::AmazonS3) => {
                let bucket = self
                    .url
                    .host_str()
                    .ok_or_else(|| Error::invalid_argument("S3 URL missing bucket"))?
                    .to_owned();
                let prefix = self.url.path().trim_start_matches('/').to_owned();
                Ok((bucket, prefix))
            }
            StorageLocationScheme::ObjectStore(ObjectStoreScheme::MicrosoftAzure) => {
                // abfss://container@account.dfs.core.windows.net/path
                // or https://account.blob.core.windows.net/container/path
                let host = self.url.host_str().unwrap_or_default();
                if host.contains('@') {
                    // ABFSS: container@account.dfs.core.windows.net
                    let container = host
                        .split('@')
                        .next()
                        .ok_or_else(|| Error::invalid_argument("Invalid ABFSS URL"))?
                        .to_owned();
                    let prefix = self.url.path().trim_start_matches('/').to_owned();
                    Ok((container, prefix))
                } else {
                    // https://account.blob.core.windows.net/container/path
                    let mut segments = self
                        .url
                        .path_segments()
                        .ok_or_else(|| Error::invalid_argument("Azure URL has no path"))?;
                    let container = segments
                        .next()
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| Error::invalid_argument("Azure URL missing container"))?
                        .to_owned();
                    let prefix = segments.collect::<Vec<_>>().join("/");
                    Ok((container, prefix))
                }
            }
            StorageLocationScheme::Azurite => {
                if self.url.scheme() != "azurite" {
                    // http://localhost:10000/account/container/path
                    let parts: Vec<_> = self.url.path().splitn(4, '/').collect();
                    // parts: ["", "account", "container", "path"]
                    let container = parts
                        .get(2)
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| Error::invalid_argument("Azurite URL missing container"))?
                        .to_string();
                    let prefix = parts.get(3).copied().unwrap_or("").to_owned();
                    Ok((container, prefix))
                } else {
                    // azurite://container/path
                    let container = self
                        .url
                        .host_str()
                        .ok_or_else(|| {
                            Error::invalid_argument("Azurite URL missing container in host")
                        })?
                        .to_owned();
                    let prefix = self.url.path().trim_start_matches('/').to_owned();
                    Ok((container, prefix))
                }
            }
            _ => Err(Error::invalid_argument(
                "bucket_and_prefix not supported for this URL scheme",
            )),
        }
    }

    /// Returns the Azure storage account name parsed from the URL, if present.
    ///
    /// - HTTPS: `https://account.blob.core.windows.net/…` → `Some("account")`
    /// - ABFSS: `abfss://container@account.dfs.core.windows.net/…` → `Some("account")`
    /// - Azurite HTTP: `http://localhost:10000/account/container/path` → `Some("account")`
    /// - Azurite custom: `azurite://container/path` → `None` (no account concept)
    pub fn azure_account(&self) -> Option<String> {
        match &self.scheme {
            StorageLocationScheme::ObjectStore(ObjectStoreScheme::MicrosoftAzure) => {
                let host = self.url.host_str()?;
                if host.contains('@') {
                    // abfss://container@account.dfs.core.windows.net
                    host.split('@')
                        .nth(1)
                        .map(|h| h.split('.').next().unwrap_or(h).to_owned())
                } else {
                    // https://account.blob.core.windows.net
                    host.split('.').next().map(ToOwned::to_owned)
                }
            }
            StorageLocationScheme::Azurite if self.url.scheme() != "azurite" => {
                // http://localhost:10000/account/container/path
                self.url
                    .path_segments()
                    .and_then(|mut s| s.next())
                    .filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned)
            }
            _ => None,
        }
    }
}

fn is_azurite(url: &Url) -> bool {
    // for now we assume that azurite is using default values.
    // since this is only for local development, this may be indefinitely the case.
    url.scheme() == "azurite"
        || (url.scheme() == "http"
            && matches!(url.host_str(), Some("localhost") | Some("127.0.0.1"))
            && url.port() == Some(10000))
}

fn get_store_url(url: &url::Url) -> Result<(Url, StorageLocationScheme, Url)> {
    let scheme = StorageLocationScheme::parse(url)?;
    let store_url = match &scheme {
        StorageLocationScheme::ObjectStore(_) => Url::parse(&format!(
            "{}://{}",
            scheme.as_ref(),
            &url[url::Position::BeforeHost..url::Position::AfterPort]
        ))
        .map_err(|e| Error::Generic(e.to_string()))?,
        StorageLocationScheme::Azurite => {
            if url.scheme() != "azurite" {
                // split the path into 3 parts:
                // 1. account name
                // 2. container name
                // 3. path
                let parts = url.path().splitn(4, "/").collect::<Vec<_>>();
                if parts.len() != 4 {
                    return Err(Error::invalid_argument(format!(
                        "Invalid azurite path: {}",
                        url.path()
                    )));
                }
                Url::parse(&format!("azurite://{}", parts[2]))
                    .map_err(|e| Error::Generic(e.to_string()))?
            } else {
                Url::parse(&format!(
                    "{}://{}/",
                    url.scheme(),
                    &url[url::Position::BeforeHost..url::Position::AfterPort]
                ))
                .map_err(|e| Error::Generic(e.to_string()))?
            }
        }
    };
    let location = match &scheme {
        StorageLocationScheme::ObjectStore(_) => store_url.join(url.path())?,
        StorageLocationScheme::Azurite if url.scheme() != "azurite" => {
            let path = url
                .path_segments()
                .ok_or_else(|| Error::invalid_argument("Invalid azurite path"))?
                .skip(2)
                .join("/");
            url::Url::parse(&format!("{}{}", store_url.as_str(), path))?
        }
        StorageLocationScheme::Azurite => url.clone(),
    };
    Ok((store_url, scheme, location))
}
