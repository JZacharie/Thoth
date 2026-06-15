use anyhow::Result;
use keyring::Entry;

const SERVICE_NAME: &str = "thoth";
const PYLOS_SECRET_KEY: &str = "pylos_secret";

pub fn store_secret(secret: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, PYLOS_SECRET_KEY)?;
    entry.set_password(secret)?;
    tracing::info!("secret stored securely via keyring");
    Ok(())
}

pub fn get_secret() -> Result<Option<String>> {
    let entry = Entry::new(SERVICE_NAME, PYLOS_SECRET_KEY)?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_secret() -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, PYLOS_SECRET_KEY)?;
    entry.delete_credential()?;
    Ok(())
}
