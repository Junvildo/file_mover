use std::fs;
use std::path::Path;
use std::process::Command;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn select_folder() -> Option<String> {
    tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new()
            .pick_folder()
            .map(|p| p.to_string_lossy().to_string())
    })
    .await
    .unwrap()
}

#[tauri::command]
fn move_files_from_directory(source_path: &str, destination_path: &str) -> Result<Vec<String>, String> {
    let source_dir = Path::new(source_path);
    let dest_dir = Path::new(destination_path);
    
    if !source_dir.exists() {
        return Err("Source directory does not exist".to_string());
    }
    
    if !dest_dir.exists() {
        return Err("Destination directory does not exist".to_string());
    }
    
    let mut moved_files = Vec::new();
    
    for entry in fs::read_dir(source_dir)
        .map_err(|e| format!("Failed to read source directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let file_path = entry.path();
        
        if file_path.is_file() {
            let file_name = file_path.file_name()
                .ok_or_else(|| "Failed to get file name".to_string())?;
            let dest_file_path = dest_dir.join(file_name);
            
            // Use fs::rename to actually move the file (not copy)
            fs::rename(&file_path, &dest_file_path)
                .map_err(|e| format!("Failed to move file {:?}: {}", file_path, e))?;
            
            moved_files.push(file_name.to_string_lossy().to_string());
        }
    }
    
    Ok(moved_files)
}

#[tauri::command]
fn list_files(path: &str) -> Result<Vec<String>, String> {
    let dir = Path::new(path);
    
    if !dir.exists() {
        return Err("Directory does not exist".to_string());
    }
    
    let mut files = Vec::new();
    
    for entry in fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let file_path = entry.path();
        
        if file_path.is_file() {
            if let Some(file_name) = file_path.file_name() {
                files.push(file_name.to_string_lossy().to_string());
            }
        }
    }
    
    Ok(files)
}

#[tauri::command]
#[cfg(windows)]
fn hide_files_in_directory(directory: &str, files: Vec<String>) -> Result<(), String> {
    use std::path::PathBuf;
    
    let dir_path = Path::new(directory);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }
    
    for file in files {
        let file_path = dir_path.join(&file);
        if file_path.exists() {
            let cmd = format!("attrib +h \"{}\"", file_path.to_string_lossy());
            let output = Command::new("cmd")
                .args(&["/C", &cmd])
                .output()
                .map_err(|e| format!("Failed to execute attrib command: {}", e))?;
                
            if !output.status.success() {
                return Err(format!("Failed to hide file {}: {}", file, String::from_utf8_lossy(&output.stderr)));
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
#[cfg(windows)]
fn show_files_in_directory(directory: &str, files: Vec<String>) -> Result<(), String> {
    use std::path::PathBuf;
    
    let dir_path = Path::new(directory);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }
    
    for file in files {
        let file_path = dir_path.join(&file);
        if file_path.exists() {
            let cmd = format!("attrib -h \"{}\"", file_path.to_string_lossy());
            let output = Command::new("cmd")
                .args(&["/C", &cmd])
                .output()
                .map_err(|e| format!("Failed to execute attrib command: {}", e))?;
                
            if !output.status.success() {
                return Err(format!("Failed to show file {}: {}", file, String::from_utf8_lossy(&output.stderr)));
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
#[cfg(unix)]
fn hide_files_in_directory(directory: &str, files: Vec<String>) -> Result<(), String> {
    use std::path::PathBuf;
    
    let dir_path = Path::new(directory);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }
    
    for file in files {
        let original_path = dir_path.join(&file);
        if original_path.exists() {
            // Add a dot at the beginning to make it hidden on Unix systems
            let hidden_name = format!(".{}", file);
            let hidden_path = dir_path.join(&hidden_name);
            
            // Check if the hidden file already exists to avoid conflicts
            if hidden_path.exists() {
                return Err(format!("Hidden file {} already exists", hidden_name));
            }
            
            fs::rename(&original_path, &hidden_path)
                .map_err(|e| format!("Failed to rename file {} to {}: {}", file, hidden_name, e))?;
        }
    }
    
    Ok(())
}

#[tauri::command]
#[cfg(unix)]
fn show_files_in_directory(directory: &str, files: Vec<String>) -> Result<(), String> {
    use std::path::PathBuf;
    
    let dir_path = Path::new(directory);
    if !dir_path.exists() {
        return Err("Directory does not exist".to_string());
    }
    
    for file in files {
        // The visible file name would be the original (without the dot)
        let visible_name = file.trim_start_matches('.');
        let hidden_name = format!(".{}", visible_name);
        let hidden_path = dir_path.join(&hidden_name);
        
        if hidden_path.exists() {
            let visible_path = dir_path.join(visible_name);
            
            // Check if the visible file already exists to avoid conflicts
            if visible_path.exists() {
                return Err(format!("Visible file {} already exists", visible_name));
            }
            
            fs::rename(&hidden_path, &visible_path)
                .map_err(|e| format!("Failed to rename file {} to {}: {}", hidden_name, visible_name, e))?;
        }
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            select_folder,
            move_files_from_directory,
            list_files,
            hide_files_in_directory,
            show_files_in_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
