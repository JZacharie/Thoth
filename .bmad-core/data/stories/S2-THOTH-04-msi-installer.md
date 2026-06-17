## Story: Installateur MSI

**ID**: S2-THOTH-04
**Épic**: EPIC-THOTH-02
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** installer Thoth via un programme d'installation Windows standard
**So que** l'installation soit propre, avec ajout au PATH et dans la liste des programmes installés

---

### Acceptance Criteria

- [ ] Given l'installateur MSI est exécuté, when l'installation termine, alors Thoth est disponible dans le menu Démarrer
- [ ] Given l'installateur MSI, when l'installation termine, alors Thoth est dans "Ajout/Suppression de programmes"
- [ ] Given l'installateur MSI, when l'utilisateur choisit le dossier d'installation, alors Thoth est installé dans ce dossier
- [ ] Given l'installateur MSI, when l'utilisateur coche "Démarrage automatique", alors l'entrée registry est créée pendant l'installation
- [ ] Given l'utilisateur lance "Désinstaller", when la désinstallation est exécutée, alors Thoth est complètement supprimé

---

### Technical Notes

**Approche**: Utiliser **WiX Toolset v4** pour générer le MSI dans la CI/CD.

**Structure du package**:
```
- thoth.exe          → ProgramFiles64Folder\Thoth\
- config.toml (opt)  → AppDataFolder\Thoth\
```

**Pipeline CI - Job supplémentaire**:
```yaml
msi:
    name: Build MSI Installer
    needs: [build]
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v6
      - name: Download build artifact
        uses: actions/download-artifact@v4
        with:
          name: thoth-windows-x86_64
          path: ./release
      - name: Build MSI
        uses: wixtoolset/actions-wix@v1
        with:
          source: ./installer/Thoth.wxs
          output: ./Thoth-{version}.msi
          variables: "ThothBinary=./release/thoth.exe"
      - name: Upload MSI
        uses: actions/upload-artifact@v7
        with:
          name: Thoth-{version}.msi
          path: ./Thoth-*.msi
```

**WiX source** (`installer/Thoth.wxs`):
```xml
<Wix xmlns="http://wixtoolset.org/schemas/v4/wxs">
    <Package Name="Thoth" Manufacturer="Pylos" Version="$(version)" UpgradeCode="...">
        <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFiles64Folder">
                <Directory Id="INSTALLDIR" Name="Thoth" />
            </Directory>
        </Directory>
        <ComponentGroupRef Id="ProductComponents" />
        <Feature Id="Main" Title="Thoth" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>
    </Package>
</Wix>
```

**Release assets**: Ajouter le MSI à la GitHub Release aux côtés du `.exe` et du `.zip`.

---

### Definition of Done

- [ ] MSI généré dans la CI sur tag `v*`
- [ ] Installation dans `ProgramFiles64Folder`
- [ ] Entrée dans "Ajout/Suppression de programmes"
- [ ] Option de démarrage automatique pendant l'install
- [ ] Désinstallation complète
- [ ] `cargo clippy` et `cargo test` passent
