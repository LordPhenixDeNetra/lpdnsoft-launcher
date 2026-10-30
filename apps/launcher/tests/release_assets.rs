//! Garde-fou de release : ce qui est publié porte le nom que le launcher cherche.
//!
//! Un paquet mal nommé n'est pas une erreur de compilation : le launcher ne le trouverait
//! simplement pas, et afficherait « aucun paquet pour cette plateforme » sur sa propre
//! version. Ces tests lisent les fichiers réels — `apps.json`, le workflow de release, le MSI
//! et l'`Info.plist` — pour que ce contrat ne puisse pas se rompre en silence.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use lpdnsoft_launcher_core::{Arch, Manifest, Os, Platform, Version};

const APPS: &str = include_str!("../../../apps.json");
const RELEASE: &str = include_str!("../../../.github/workflows/release.yml");
const PLIST: &str = include_str!("../../../packaging/macos/Info.plist");
const WXS: &str = include_str!("../../../packaging/windows/launcher.wxs");

fn launcher() -> lpdnsoft_launcher_core::AppSpec {
    Manifest::parse(APPS)
        .expect("apps.json doit rester valide")
        .apps
        .into_iter()
        .find(|app| app.id == "lpdnsoft-launcher")
        .expect("l'entrée du launcher doit exister")
}

fn version() -> Version {
    Version::new(0, 4, 0)
}

/// Nom rendu par le manifeste pour cette plateforme.
fn rendered(os: Os, arch: Arch, extension: &str) -> String {
    let app = launcher();
    let name = app.asset.render(&version(), Platform::new(os, arch));
    let stem = name
        .strip_suffix(".deb")
        .or_else(|| name.strip_suffix(".msi"))
        .or_else(|| name.strip_suffix(".pkg"))
        .unwrap_or(&name);
    format!("{stem}.{extension}")
}

/// Le même nom, avec le numéro de version remplacé par la variable du workflow.
fn in_workflow(name: &str) -> String {
    name.replace(&version().to_string(), "$version")
}

#[test]
fn the_manifest_names_the_four_packages() {
    assert_eq!(
        rendered(Os::Windows, Arch::X86_64, "msi"),
        "lpdnsoft-launcher-0.4.0-windows-x86_64.msi"
    );
    assert_eq!(
        rendered(Os::Linux, Arch::X86_64, "deb"),
        "lpdnsoft-launcher-0.4.0-linux-x86_64.deb"
    );
    assert_eq!(
        rendered(Os::Linux, Arch::X86_64, "rpm"),
        "lpdnsoft-launcher-0.4.0-linux-x86_64.rpm"
    );
    assert_eq!(
        rendered(Os::Macos, Arch::Aarch64, "pkg"),
        "lpdnsoft-launcher-0.4.0-macos-aarch64.pkg"
    );
    assert_eq!(
        rendered(Os::Macos, Arch::X86_64, "pkg"),
        "lpdnsoft-launcher-0.4.0-macos-x86_64.pkg"
    );
}

#[test]
fn the_release_workflow_builds_every_package_the_launcher_expects() {
    // macOS est construit par une matrice : le nom porte `${{ matrix.arch }}` dans le
    // workflow, pas l'architecture écrite en dur.
    let cases = [
        (rendered(Os::Windows, Arch::X86_64, "msi"), false),
        (rendered(Os::Linux, Arch::X86_64, "deb"), false),
        (rendered(Os::Linux, Arch::X86_64, "rpm"), false),
        (rendered(Os::Macos, Arch::Aarch64, "pkg"), true),
        (rendered(Os::Macos, Arch::X86_64, "pkg"), true),
    ];
    for (name, from_matrix) in cases {
        let mut expected = in_workflow(&name);
        if from_matrix {
            expected = expected
                .replace("aarch64", "${{ matrix.arch }}")
                .replace("x86_64", "${{ matrix.arch }}");
        }
        assert!(
            RELEASE.contains(&expected),
            "le workflow ne produit pas {expected}"
        );
    }
}

#[test]
fn the_release_publishes_the_checksums_file() {
    assert!(
        RELEASE.contains("SHA256SUMS"),
        "une release sans SHA256SUMS ne peut pas être installée : le launcher exige l'empreinte"
    );
    assert!(
        RELEASE.contains("sha256sum"),
        "les empreintes doivent être calculées"
    );
}

#[test]
fn the_macos_package_identifier_is_the_one_the_manifest_declares() {
    let app = launcher();
    assert!(PLIST.contains(app.pkg_id()), "{}", app.pkg_id());
    assert_eq!(app.pkg_id(), "fr.lpdnsoft.launcher");
}

#[test]
fn the_msi_installs_the_layout_the_detection_looks_for() {
    let app = launcher();
    // `DisplayName` du registre = nom de l'application : c'est la valeur que la détection
    // cherche dans les clés de désinstallation.
    assert!(
        WXS.contains(&format!(r#"Name="{}""#, app.name)),
        "le MSI doit s'appeler {}",
        app.name
    );
    // La résolution de chemin essaie `%ProgramFiles%\<nom>\<id>.exe` quand le registre ne
    // donne pas de chemin : le MSI doit poser exactement cette disposition.
    assert!(WXS.contains(r#"Source="$(BinDir)\lpdnsoft-launcher.exe""#));
    assert!(WXS.contains(&format!(
        r#"Directory Id="INSTALLFOLDER" Name="{}""#,
        app.name
    )));
}

#[test]
fn the_deb_and_rpm_names_stay_the_crate_name() {
    let app = launcher();
    // `dpkg-query -W` et `rpm -q` interrogent le nom du paquet : il doit rester celui du
    // crate, sinon la détection ne trouverait rien.
    assert_eq!(app.deb_package(), "lpdnsoft-launcher");
    assert_eq!(app.rpm_package(), "lpdnsoft-launcher");
}

#[test]
fn the_release_notes_are_generated_and_not_written_by_hand() {
    // Les notes de version décrivent des pull requests fusionnées : les rédiger à la main
    // les rend fausses dès la deuxième release. GitHub les génère, le launcher les affiche.
    assert!(
        RELEASE.contains("--generate-notes"),
        "les notes doivent être générées depuis les pull requests fusionnées"
    );
    assert!(
        !RELEASE.contains("--notes \""),
        "une note écrite en dur deviendrait fausse à la release suivante"
    );
}

#[test]
fn the_release_runs_when_a_version_reaches_main() {
    assert!(RELEASE.contains("branches: [main]"));
    assert!(
        RELEASE.contains("gh release view"),
        "la publication doit être sautée quand la version est déjà publiée"
    );
}

#[test]
fn every_build_job_fetches_the_sibling_repository() {
    // Le launcher dépend de `../lpdnsoft-libs` par chemin. Sans ce second checkout, la
    // compilation échoue sur le runner — c'est arrivé à la première publication, et le
    // workflow paraissait pourtant valide.
    let checkouts = RELEASE.matches("path: lpdnsoft-libs").count();
    let builds = RELEASE.matches("Compilation").count();
    assert!(builds >= 3, "trois systèmes compilent : {builds}");
    assert_eq!(
        checkouts, builds,
        "chaque job qui compile doit cloner le dépôt frère ({builds} attendus, {checkouts} trouvés)"
    );
    assert!(RELEASE.contains("repository: ${{ github.repository_owner }}/lpdnsoft-libs"));
}

#[test]
fn the_publish_job_works_outside_the_checkout() {
    // Il ne clone rien : si `defaults.run.working-directory` restait sur le dépôt, `cd dist`
    // échouerait faute de dossier.
    assert!(
        RELEASE.contains("working-directory: ."),
        "le job de publication doit travailler à la racine"
    );
}
