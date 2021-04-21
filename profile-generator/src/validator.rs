pub fn correct_forge_version(val: &str) -> Result<(), String> {
    let url = format!(
        "https://meta.multimc.org/v1/net.minecraftforge/{version}.json",
        version = val
    );
    if reqwest::blocking::get(&url)
        .map(|r| r.status() == 200)
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(String::from("Incorrect forge version"))
    }
}

pub fn correct_fabric_version(val: &str) -> Result<(), String> {
    let url = format!("https://maven.fabricmc.net/net/fabricmc/fabric-loader/{version}/fabric-loader-{version}.json", version = val);
    if reqwest::blocking::get(&url)
        .map(|r| r.status() == 200)
        .unwrap_or(false)
    {
        Ok(())
    } else {
        Err(String::from("Incorrect fabroc version"))
    }
}
