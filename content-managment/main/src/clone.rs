use content_managment_datamodel::datamodel::Resources;
use std::collections::HashSet;
use crate::datamodel::resource_file_manager::ResourceFileManager;

#[derive(Debug)]
pub struct CloneError {
    source: Option<Box<dyn std::error::Error>>,
    error: Option<String>
}

impl From<reqwest::Error> for CloneError
where
{
    fn from(e: reqwest::Error) -> CloneError
    {
        CloneError {
            source: Some(Box::new(e)),
            error: None
        }
    }
}
impl std::fmt::Display for CloneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(source) = &self.source
        {
             write!(f,"CloneError (Nested) : {}",*source)
        }
        else if let Some(error) = &self.error
        {
            write!(f,"CloneError {}",error)
        }
        else {
            write!(f,"CloneError")
        }
    }
}

impl From<serde_yaml::Error> for CloneError
where
{
    fn from(e: serde_yaml::Error) -> CloneError
    {
        CloneError {
            source: Some(Box::new(e)),
            error: None
        }
    }
}
impl From<std::io::Error> for CloneError
where
{
    fn from(e: std::io::Error) -> CloneError
    {
        CloneError {
            source: Some(Box::new(e)),
            error: None
        }
    }
}
impl From<&str> for CloneError
{
    fn from(e: &str) -> CloneError
    {
        CloneError {
            source: None,
            error: Some(String::from(e))
        }
    }
}
impl From<String> for CloneError
{
    fn from(e: String) -> CloneError
    {
        CloneError {
            source: None,
            error: Some(e)
        }
    }
}

pub async fn fetch_file(client: &reqwest::Client, website: &str, web_path: &str,   path: std::path::PathBuf) -> Result<(),CloneError>
{
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::File::create(&path).await?;
    let url = format!("{}/{}",website,web_path);
    let mut resp = client.get(&url).send().await?;
    if !resp.status().is_success()
    {
        return Err(CloneError::from(format!("Failed to fetch {}",url)))
    }

    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }
    Ok(())
}


pub async fn clone_node(path: &str, local_resources: Resources) -> Result<Resources,CloneError>
{

    let client = reqwest::Client::new();
    let manifest_path =format!("{}/manifest.yaml",path);
    println!("Cloning \"{}\"",manifest_path);
    let resp = client.get(manifest_path).send().await?;
    if resp.status().is_success()
    {
        let body = resp.text().await?;
        let remote_resources : Resources = serde_yaml::from_str(&body)?;

        let remote_keys : HashSet<&String> = remote_resources.resources.keys().collect();
        let local_keys :  HashSet<&String> = local_resources.resources.keys().collect();

        for key_to_remove in local_keys.difference(&remote_keys)
        {
            println!("clone: Remove {}",key_to_remove);
            local_resources.resources.get(*key_to_remove).unwrap().delete_resource();
        }

        for key_to_add in remote_keys.difference(&local_keys)
        {
            println!("clone: Adding {}",key_to_add);
            let resource  = remote_resources.resources.get(*key_to_add).unwrap();

            fetch_file(&client, path, &resource.url_path(), resource.file_path()).await?;
        }

        for key_to_update in remote_keys.intersection(&local_keys)
        {
            let local = local_resources.resources.get(*key_to_update).unwrap();
            let remote = remote_resources.resources.get(*key_to_update).unwrap();

            if local.content_hash != remote.content_hash
            {
                println!("clone: Updating {}",key_to_update);
                fetch_file(&client,path, &remote.url_path(), remote.file_path()).await?;
            }
        }

        use crate::datamodel::resource_file_manager::ResourcesFileManager;
        remote_resources.write_resources();
        Ok(remote_resources)

    }
    else 
    {
        Err(CloneError::from("Could not fetch manifest.yaml"))
    }

}