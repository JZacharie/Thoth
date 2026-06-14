use anyhow::Result;

#[cfg(windows)]
pub fn show_prompt_dialog() -> Result<String> {
    use std::os::windows::process::CommandExt;
    let script = r#"
Add-Type -AssemblyName Microsoft.VisualBasic
$result = [Microsoft.VisualBasic.Interaction]::InputBox("Entrez votre instruction personnalisée :", "Thoth — Instruction personnalisée", "")
if ([string]::IsNullOrWhiteSpace($result)) { exit 1 }
Write-Output $result
exit 0
"#;

    let output = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .creation_flags(0x08000000)
        .output()?;

    if output.status.success() {
        let text = String::from_utf8(output.stdout)?.trim().to_string();
        if text.is_empty() {
            anyhow::bail!("empty input");
        }
        Ok(text)
    } else {
        anyhow::bail!("user cancelled or input was empty");
    }
}

#[cfg(not(windows))]
pub fn show_prompt_dialog() -> Result<String> {
    tracing::warn!("input dialog not supported on this platform");
    anyhow::bail!("input dialog not supported on this platform")
}
