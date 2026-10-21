/*!
The larvae language server in Zed.

The extension starts `larvae lsp` for Luau and Lua files. The server sends
the lint findings, formats, and offers the fixes; luau-lsp beside it serves
the types. The project configures the server in `larvae.toml`, so the
extension has no settings of its own beyond the binary.
*/

use std::fs;

use zed_extension_api::{self as zed, LanguageServerId, Result, settings::LspSettings};

struct Larvae {
    /// The downloaded binary, once this session has found or fetched it
    cached: Option<String>,
}

impl Larvae {
    /*
    The `larvae` binary to run.

    A `larvae` on PATH comes first, because a user who installed one chose
    its version. Without one, the extension downloads the latest release for
    this platform into its own directory, and keeps one version there.
    */
    fn binary(&mut self, id: &LanguageServerId, worktree: &zed::Worktree) -> Result<String> {
        if let Some(path) = worktree.which("larvae") {
            return Ok(path);
        }

        if let Some(path) = &self.cached
            && is_file(path)
        {
            return Ok(path.clone());
        }

        zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "larvae-luau/larvae",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        // The asset names come from std::env::consts on the build side.
        let (os, arch) = zed::current_platform();

        let os = match os {
            zed::Os::Mac => "macos",
            zed::Os::Linux => "linux",
            zed::Os::Windows => "windows",
        };

        let arch = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X8664 => "x86_64",
            zed::Architecture::X86 => return Err("larvae ships no 32-bit x86 build".into()),
        };

        let asset_name = format!("larvae-{os}-{arch}.zip");

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("release {} has no asset {asset_name}", release.version))?;

        let dir = format!("larvae-{}", release.version);

        let path = match os {
            "windows" => format!("{dir}/larvae.exe"),
            _ => format!("{dir}/larvae"),
        };

        if !is_file(&path) {
            zed::set_language_server_installation_status(
                id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(&asset.download_url, &dir, zed::DownloadedFileType::Zip)
                .map_err(|e| format!("cannot download {asset_name}: {e}"))?;

            zed::make_file_executable(&path)?;

            // An older download is dead weight once the new one runs.
            for entry in fs::read_dir(".").map_err(|e| e.to_string())?.flatten() {
                let name = entry.file_name();

                if name.to_string_lossy().starts_with("larvae-") && *name != *dir {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached = Some(path.clone());

        Ok(path)
    }
}

impl zed::Extension for Larvae {
    fn new() -> Self {
        Self { cached: None }
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // `lsp.larvae.binary` in the Zed settings can name the binary and its arguments.
        let binary = LspSettings::for_worktree(id.as_ref(), worktree)
            .ok()
            .and_then(|settings| settings.binary);

        let command = match binary.as_ref().and_then(|b| b.path.clone()) {
            Some(path) => path,

            None => self.binary(id, worktree)?,
        };

        let args = binary
            .and_then(|b| b.arguments)
            .unwrap_or_else(|| vec!["lsp".into()]);

        Ok(zed::Command {
            command,
            args,
            env: worktree.shell_env(),
        })
    }
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file())
}

zed::register_extension!(Larvae);
