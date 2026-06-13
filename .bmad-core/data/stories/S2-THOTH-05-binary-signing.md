## Story: Signature du Binaire Windows

**ID**: S2-THOTH-05
**Épic**: EPIC-THOTH-02
**Points**: 2
**Statut**: BLOCKED

---

### User Story

**As a** Utilisateur
**I want** que l'exécutable Thoth soit signé numériquement
**So that** Windows et les antivirus ne le traitent pas comme un logiciel suspect

---

### Acceptance Criteria

- [ ] Given le binaire `thoth.exe`, when il est signé avec un certificat Authenticode, alors les propriétés Windows affichent "Signé par [Éditeur]"
- [ ] Given le binaire signé, when l'utilisateur le télécharge, alors Windows SmartScreen ne bloque pas l'exécution
- [ ] Given la signature est effectuée dans la CI, when le tag `v*` est pushé, alors le `.exe` est signé avant d'être uploadé dans la release
- [ ] Given le certificat est expiré ou invalide, when la signature échoue, alors la CI échoue explicitement

---

### Technical Notes

**Certificat**:
- Utiliser un certificat **Authenticode** (Code Signing) auprès d'une autorité (DigiCert, Sectigo, Let's Encrypt n'est pas supporté)
- Stocker le certificat dans GitHub Secrets (`CERTIFICATE_BASE64`, `CERTIFICATE_PASSWORD`)

**Commande de signature**:
```bash
# Déchiffrer le certificat
echo "$env:CERTIFICATE_BASE64" | base64 --decode > certificate.pfx

# Signer le binaire
signtool sign /fd SHA256 \
    /f certificate.pfx \
    /p "$env:CERTIFICATE_PASSWORD" \
    /tr http://timestamp.digicert.com \
    /td SHA256 \
    /v target/release/thoth.exe
```

**Job CI supplémentaire**:
```yaml
sign:
    name: Sign Binary
    needs: [build]
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Download build artifact
        uses: actions/download-artifact@v4
        with:
          name: thoth-windows-x86_64
          path: ./release
      - name: Sign binary
        env:
          CERTIFICATE_BASE64: ${{ secrets.CERTIFICATE_BASE64 }}
          CERTIFICATE_PASSWORD: ${{ secrets.CERTIFICATE_PASSWORD }}
        run: |
          echo "$env:CERTIFICATE_BASE64" | base64 --decode > cert.pfx
          & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\signtool.exe" sign `
            /fd SHA256 /f cert.pfx /p "$env:CERTIFICATE_PASSWORD" `
            /tr http://timestamp.digicert.com /td SHA256 /v ./release/thoth.exe
```

**Timestamp**: Toujours ajouter un timestamp RFC 3161 pour que la signature reste valide après expiration du certificat.

---

### Definition of Done

- [ ] Le binaire est signé Authenticode (SHA256) dans la CI
- [ ] Le timestamp est appliqué
- [ ] La CI échoue si la signature rate
- [ ] Les artefacts release sont signés
- [ ] `cargo clippy` et `cargo test` passent
