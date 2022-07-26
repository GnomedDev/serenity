use std::borrow::Cow;
#[cfg(not(feature = "http"))]
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(feature = "http")]
use reqwest::Client;
#[cfg(feature = "http")]
use tokio::{fs::File, io::AsyncReadExt};
use url::Url;

#[cfg(feature = "http")]
use crate::error::{Error, Result};

/// Enum that allows a user to pass a [`Path`] or a [`File`] type to [`send_files`]
///
/// [`send_files`]: crate::model::id::ChannelId::send_files
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum AttachmentType<'a> {
    /// Indicates that the [`AttachmentType`] is a byte slice with a filename.
    Bytes { data: Cow<'static, [u8]>, filename: Cow<'static, str> },
    /// Indicates that the [`AttachmentType`] is a [`File`]
    File { file: &'a File, filename: Cow<'static, str> },
    /// Indicates that the [`AttachmentType`] is a [`Path`]
    Path(&'a Path),
    /// Indicates that the [`AttachmentType`] is an image URL.
    Image(Url),
}

async fn data_path(path: &Path) -> Result<Vec<u8>> {
    tokio::fs::read(path).await.map_err(Into::into)
}

async fn data_file(file: &File) -> Result<Vec<u8>> {
    let mut data_buf = Vec::new();
    file.try_clone().await?.read_to_end(&mut data_buf).await?;
    Ok(data_buf)
}

async fn data_image(client: &Client, url: Url) -> Result<Vec<u8>> {
    let response = client.get(url).send().await?;
    Ok(response.bytes().await?.to_vec())
}

#[cfg(feature = "http")]
impl<'a> AttachmentType<'a> {
    pub(crate) async fn deconstruct(
        self,
        client: &Client,
    ) -> Result<(Cow<'static, [u8]>, Option<Cow<'static, str>>)> {
        Ok(match self {
            Self::Bytes {
                data,
                filename,
            } => (data, Some(filename)),
            Self::File {
                file,
                filename,
            } => {
                let data = data_file(file).await?;
                (data.into(), Some(filename))
            },
            Self::Path(path) => {
                let filename =
                    path.file_name().map(|filename| filename.to_string_lossy().to_string());
                let data = data_path(path).await?;

                (data.into(), filename.map(Cow::from))
            },
            Self::Image(url) => {
                let filename = match url.path_segments().and_then(Iterator::last) {
                    Some(filename) => filename.to_string().into(),
                    None => return Err(Error::Url(url.to_string())),
                };

                let data = data_image(client, url).await?;
                (data.into(), Some(filename))
            },
        })
    }

    pub(crate) async fn data(self, client: &Client) -> Result<Cow<'static, [u8]>> {
        Ok(match self {
            Self::Bytes {
                data, ..
            } => data,
            Self::File {
                file, ..
            } => data_file(file).await?.into(),

            Self::Path(path) => data_path(path).await?.into(),
            Self::Image(url) => data_image(client, url).await?.into(),
        })
    }
}

impl From<(Cow<'static, [u8]>, Cow<'static, str>)> for AttachmentType<'static> {
    fn from((data, filename): (Cow<'static, [u8]>, Cow<'static, str>)) -> Self {
        AttachmentType::Bytes {
            data,
            filename,
        }
    }
}

impl<'a> From<&'a str> for AttachmentType<'a> {
    /// Constructs an [`AttachmentType`] from a string.
    /// This string may refer to the path of a file on disk, or the http url to an image on the internet.
    fn from(s: &'a str) -> Self {
        match Url::parse(s) {
            Ok(url) => AttachmentType::Image(url),
            Err(_) => AttachmentType::Path(Path::new(s)),
        }
    }
}

impl<'a> From<&'a Path> for AttachmentType<'a> {
    fn from(path: &'a Path) -> AttachmentType<'_> {
        AttachmentType::Path(path)
    }
}

impl<'a> From<&'a PathBuf> for AttachmentType<'a> {
    fn from(pathbuf: &'a PathBuf) -> AttachmentType<'_> {
        AttachmentType::Path(pathbuf.as_path())
    }
}

impl<'a> From<(&'a File, Cow<'static, str>)> for AttachmentType<'a> {
    fn from((file, filename): (&'a File, Cow<'static, str>)) -> AttachmentType<'a> {
        AttachmentType::File {
            file,
            filename,
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::AttachmentType;

    #[test]
    fn test_attachment_type() {
        assert!(matches!(
            AttachmentType::from(Path::new("./dogs/corgis/kona.png")),
            AttachmentType::Path(_)
        ));
        assert!(matches!(
            AttachmentType::from(Path::new("./cats/copycat.png")),
            AttachmentType::Path(_)
        ));
        assert!(matches!(
            AttachmentType::from("./mascots/crabs/ferris.png"),
            AttachmentType::Path(_)
        ));
        assert!(matches!(
            AttachmentType::from("https://test.url/test.jpg"),
            AttachmentType::Image(_)
        ));
    }
}
