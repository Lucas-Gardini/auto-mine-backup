use std::sync::Arc;

use tauri::{AppHandle, Emitter, Wry};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_store::Store;

use std::fs::File;
use std::io::{Read, Write};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use crate::enums::WorldStatus;

pub fn log_to_file_and_emit<S: AsRef<str>>(app_handle: &AppHandle, msg: S) {
    use std::fs::{File, OpenOptions};
    use std::io::{BufRead, BufReader, Write};

    let log_path = "auto-mine-backup.log";
    if std::path::Path::new(log_path).exists() {
        let file = File::open(log_path).unwrap();
        let reader = BufReader::new(file);
        let lines: Vec<_> = reader.lines().collect::<Result<_, _>>().unwrap();
        if lines.len() >= 1000 {
            let new_lines = &lines[200..];
            let mut file = File::create(log_path).unwrap();
            for line in new_lines {
                writeln!(file, "{}", line).unwrap();
            }
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .unwrap();
    writeln!(file, "{}", msg.as_ref()).unwrap();
    println!("{}", msg.as_ref());

    app_handle
        .emit("log_event", &msg.as_ref().to_string())
        .unwrap();
}

pub async fn check_minecraft_running(app_handle: &AppHandle) -> WorldStatus {
    let pwsh_script = r#"
        $process = Get-CimInstance Win32_Process -Filter "Name = 'javaw.exe'" | Where-Object { $_.CommandLine -like "*net.minecraft.client.main.Main*" }
		if (-not $process) {
			Write-Output "STOPPED"
			return
		}

		$commandLine = $process.CommandLine

		if (-not ($commandLine -match '--gameDir "([^"]+)"')) {
			if ($commandLine -match '--quickPlayPath "?([^"]+\.json)"?') {
				$quickPlayPath = $matches[1]
				$quickPlayJsonData = Get-Content -Path $quickPlayPath | ConvertFrom-Json
				$worldId = $quickPlayJsonData[0].id

				$isMultiplayer = $quickPlayJsonData[0].type -eq "multiplayer"

				if ($isMultiplayer) {
					Write-Output "MULTIPLAYER:$worldId"
					return
				}

				if (-not $worldId) {
					# Não achou o id no quickPlayPath (estranho), assuma Menu
					Write-Output "NO_WORLDID:$quickPlayPath"
					return
				}	

				Write-Output "WORLD:$worldId"
				return
			}
		else {
				# Não achou o gameDir (estranho), assuma Menu
				Write-Output "NO_GAMEDIR:$commandLine"
				return
			}
		}
		else {
			$gameDir = $matches[1]
		}

		$logPath = Join-Path -Path $gameDir -ChildPath "logs\latest.log"
		if (-not (Test-Path $logPath)) {
			# Achou o jogo mas não o log, assuma Menu
			Write-Output "NO_LOG:$gameDir"
			return
		}

		$patterns = "Connecting to", "Starting integrated server for world"
		$lastMatch = Select-String -Path $logPath -Pattern $patterns | Select-Object -Last 1

		if (-not $lastMatch) {
			Write-Output "NO_MATCH"
			return
		}

		if ($lastMatch.Pattern -eq "Connecting to") {
			$worldName = ($lastMatch.Line -split ",")[0] -replace ".*Connecting to ", ""
			Write-Output ("WORLD:" + $worldName.Trim())
		}
		elseif ($lastMatch.Pattern -eq "Starting integrated server for world") {
			$worldName = $lastMatch.Line -replace ".*Starting integrated server for world '", "" -replace "'$", ""
			Write-Output ("WORLD:" + $worldName.Trim())
		}
		else {
			Write-Output "FALLBACK"
		}
    "#;

    let shell = app_handle.shell();
    let output_result = shell
        .command("powershell")
        .args(["-Command", pwsh_script])
        .output()
        .await;

    match output_result {
        Ok(output) => {
            if !output.status.success() {
                log_to_file_and_emit(
                    &app_handle,
                    format!(
                        "[check_minecraft_running] Erro ao executar script PowerShell: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ),
                );
                return WorldStatus::Stopped;
            }

            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if stdout.starts_with("NO_GAMEDIR") {
                let command_line = stdout.replace("NO_GAMEDIR:", "");
                log_to_file_and_emit(&app_handle, format!("[check_minecraft_running] Jogo em execução, mas não foi possível determinar o gameDir. Linha de comando: {}", command_line));
                return WorldStatus::MainMenu;
            }

            if stdout.starts_with("NO_WORLDID") {
                let quickplay_path = stdout.replace("NO_WORLDID:", "");
                log_to_file_and_emit(&app_handle, format!("[check_minecraft_running] Jogo em execução, mas não foi possível determinar o ID do mundo a partir do quickPlayPath: {}", quickplay_path));
                return WorldStatus::MainMenu;
            }

            if stdout.starts_with("NO_WORLDDIR") {
                let world_dir = stdout.replace("NO_WORLDDIR:", "");
                log_to_file_and_emit(&app_handle, format!("[check_minecraft_running] Jogo em execução, mas não foi possível determinar o diretório do mundo em: {}", world_dir));
                return WorldStatus::MainMenu;
            }

            if stdout.starts_with("NO_LOG") {
                let game_dir = stdout.replace("NO_LOG:", "");
                log_to_file_and_emit(&app_handle, format!("[check_minecraft_running] Jogo em execução, mas não foi possível encontrar o log em: {}", game_dir));
                return WorldStatus::MainMenu;
            }

            if stdout.starts_with("NO_MATCH") {
                log_to_file_and_emit(&app_handle, "[check_minecraft_running] Jogo em execução, mas nenhuma entrada de mundo encontrada no log.");
                return WorldStatus::MainMenu;
            }

            if stdout.starts_with("STOPPED") {
                return WorldStatus::Stopped;
            }

            if stdout.starts_with("MAIN_MENU") {
                return WorldStatus::MainMenu;
            }

            if stdout.starts_with("MULTIPLAYER:") {
                let server_address = stdout.replace("MULTIPLAYER:", "").trim().to_string();
                log_to_file_and_emit(
                    &app_handle,
                    format!(
                        "[check_minecraft_running] Minecraft está em multiplayer no servidor: {}",
                        server_address
                    ),
                );
                return WorldStatus::Multiplayer(server_address);
            }

            if stdout.starts_with("WORLD:") {
                let world_name = stdout.replace("WORLD:", "").trim().to_string();
                return WorldStatus::InWorld(world_name);
            }

            // Fallback
            log_to_file_and_emit(
                &app_handle,
                format!(
                    "[check_minecraft_running] Status: Desconhecido (Saída: {})",
                    stdout
                ),
            );
            return WorldStatus::Stopped;
        }
        Err(e) => {
            log_to_file_and_emit(
                &app_handle,
                format!("[check_minecraft_running] Falha ao invocar shell: {}", e),
            );
            return WorldStatus::Stopped;
        }
    }
}

pub async fn backup_minecraft_world(
    app_handle: &AppHandle,
    store: &Arc<Store<Wry>>,
    world_name: String,
) {
    let selected_path = store.get("selected_path").unwrap_or_default();
    let max_backups = store.get("max_backups").unwrap_or_default();

    if selected_path.is_null() {
        log_to_file_and_emit(
            &app_handle,
            "[backup_minecraft_world] Caminho de backup não está configurado.",
        );
        return;
    }

    if max_backups.is_null() {
        log_to_file_and_emit(
            &app_handle,
            "[backup_minecraft_world] Número máximo de backups não está configurado.",
        );
        return;
    }

    log_to_file_and_emit(
        &app_handle,
        format!(
            "[backup_minecraft_world] Iniciando backup do mundo: {} para o caminho: {}",
            world_name,
            selected_path.as_str().unwrap()
        ),
    );

    let folder_name = format!("backup_{}", world_name);
    let backup_folder_path =
        std::path::Path::new(selected_path.as_str().unwrap()).join(&folder_name);
    if !backup_folder_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&backup_folder_path) {
            log_to_file_and_emit(
                &app_handle,
                format!(
                    "[backup_minecraft_world] Falha ao criar diretório de backup: {:?} - {}",
                    backup_folder_path, e
                ),
            );
            return;
        }
    }

    let files_in_folder: Vec<_> = std::fs::read_dir(&backup_folder_path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(ext) = entry.path().extension() {
                ext == "zip"
            } else {
                false
            }
        })
        .collect();
    let max_backups_value = max_backups.as_i64().unwrap_or(5) as usize;
    if files_in_folder.len() >= max_backups_value {
        let mut files_with_dates: Vec<(std::fs::DirEntry, std::time::SystemTime)> = files_in_folder
            .into_iter()
            .filter_map(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified_time) = metadata.modified() {
                        return Some((entry, modified_time));
                    }
                }
                None
            })
            .collect();
        files_with_dates.sort_by_key(|&(_, modified_time)| modified_time);
        let files_to_delete = files_with_dates.len() + 1 - max_backups_value;
        for i in 0..files_to_delete {
            let file_to_delete = &files_with_dates[i].0;
            if let Err(e) = std::fs::remove_file(file_to_delete.path()) {
                log_to_file_and_emit(
                    &app_handle,
                    format!(
                        "[backup_minecraft_world] Falha ao deletar backup antigo: {:?} - {}",
                        file_to_delete.path(),
                        e
                    ),
                );
            } else {
                log_to_file_and_emit(
                    &app_handle,
                    format!(
                        "[backup_minecraft_world] Backup antigo deletado: {:?}",
                        file_to_delete.path()
                    ),
                );
            }
        }
    }

    let file_name = format!(
        "backup_{}_{}.zip",
        world_name,
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let destination_path = backup_folder_path.join(file_name);

    let dot_minecraft_path = dirs::home_dir()
        .unwrap()
        .join("AppData")
        .join("Roaming")
        .join(".minecraft");

    let saves_path = dot_minecraft_path.join("saves").join(&world_name);

    if !saves_path.exists() {
        log_to_file_and_emit(
            &app_handle,
            format!(
                "[backup_minecraft_world] Diretório do mundo não encontrado: {:?}",
                saves_path
            ),
        );
        return;
    }

    match zip_directory(&saves_path, &destination_path, &app_handle).await {
        Ok(_) => {
            log_to_file_and_emit(
                &app_handle,
                format!(
                    "[backup_minecraft_world] Backup concluído com sucesso: {:?}",
                    destination_path
                ),
            );
            store.set("last_backed_up_world", world_name);
        }
        Err(e) => {
            log_to_file_and_emit(
                &app_handle,
                format!("[backup_minecraft_world] Falha ao criar backup: {}", e),
            );
            store.delete("last_backed_up_world");
        }
    }
}

pub async fn zip_directory(
    src_dir: &std::path::Path,
    dst_file: &std::path::Path,
    app_handle: &tauri::AppHandle,
) -> zip::result::ZipResult<()> {
    const FILES_TO_IGNORE: [&str; 2] = ["session.lock", "level.dat_old"];

    let file = File::create(dst_file)?;
    let mut zip = zip::ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for entry in WalkDir::new(src_dir) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log_to_file_and_emit(
                    app_handle,
                    format!("[zip_directory] Falha ao acessar entrada: {}", e),
                );
                continue;
            }
        };
        let path = entry.path();
        let name = path.strip_prefix(src_dir).unwrap();

        // Ignorar arquivos da lista FILES_TO_IGNORE
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if FILES_TO_IGNORE.contains(&file_name) {
                    log_to_file_and_emit(
                        app_handle,
                        format!("[zip_directory] Ignorando arquivo: {:?}", path),
                    );
                    continue;
                }
            }
            if let Err(e) = zip.start_file(name.to_string_lossy(), options) {
                log_to_file_and_emit(
                    app_handle,
                    format!(
                        "[zip_directory] Falha ao iniciar arquivo no zip: {:?} - {}",
                        path, e
                    ),
                );
                continue;
            }
            let mut f = match File::open(path) {
                Ok(file) => file,
                Err(e) => {
                    log_to_file_and_emit(
                        app_handle,
                        format!("[zip_directory] Falha ao abrir arquivo: {:?} - {}", path, e),
                    );
                    continue;
                }
            };
            let mut buffer = Vec::new();
            if let Err(e) = f.read_to_end(&mut buffer) {
                log_to_file_and_emit(
                    app_handle,
                    format!("[zip_directory] Falha ao ler arquivo: {:?} - {}", path, e),
                );
                continue;
            }
            if let Err(e) = zip.write_all(&buffer) {
                log_to_file_and_emit(
                    app_handle,
                    format!(
                        "[zip_directory] Falha ao escrever arquivo no zip: {:?} - {}",
                        path, e
                    ),
                );
                continue;
            }
        } else if !name.as_os_str().is_empty() {
            if let Err(e) = zip.add_directory(name.to_string_lossy(), options) {
                log_to_file_and_emit(
                    app_handle,
                    format!(
                        "[zip_directory] Falha ao adicionar diretório ao zip: {:?} - {}",
                        path, e
                    ),
                );
                continue;
            }
        }
    }

    zip.finish()?;
    Ok(())
}
