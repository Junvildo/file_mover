import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [sourcePath, setSourcePath] = useState<string>("");
  const [destinationPath, setDestinationPath] = useState<string>("");
  const [statusLog, setStatusLog] = useState<string[]>([]);
  const [isMoving, setIsMoving] = useState<boolean>(false);
  const [movedFiles, setMovedFiles] = useState<string[]>([]); // Track moved files
  const [showHidden, setShowHidden] = useState<boolean>(true); // Track if hidden files are shown

  // Function to select source directory
  const selectSourceDirectory = async () => {
    addStatusLog("Opening folder selection dialog...");
    try {
      const selected = await invoke<string | null>("select_folder");
      if (selected) {
        setSourcePath(selected);
        addStatusLog("Source directory selected");
        // Reset moved files list when source directory changes
        setMovedFiles([]);
      } else {
        addStatusLog("No source directory selected");
      }
    } catch (error) {
      console.error("Error selecting source directory:", error);
      addStatusLog("Error selecting source directory");
    }
  };

  // Function to select destination directory
  const selectDestinationDirectory = async () => {
    addStatusLog("Opening folder selection dialog...");
    try {
      const destination = await invoke<string | null>("select_folder");
      if (destination) {
        setDestinationPath(destination);
        addStatusLog("Destination directory selected");
      } else {
        addStatusLog("No destination directory selected");
      }
    } catch (error) {
      console.error("Error selecting destination directory:", error);
      addStatusLog("Error selecting destination directory");
    }
  };

  // Function to move files
  const moveFiles = async () => {
    if (!sourcePath) {
      addStatusLog("Please select source directory first");
      return;
    }
    if (!destinationPath) {
      addStatusLog("Please select destination directory");
      return;
    }

    setIsMoving(true);
    addStatusLog("Moving files from source to destination...");
    
    try {
      const movedFileNames = await invoke<string[]>("move_files_from_directory", { 
        sourcePath,
        destinationPath 
      });
      
      // Update the list of moved files
      setMovedFiles(prev => [...prev, ...movedFileNames]);
      addStatusLog(`Successfully moved ${movedFileNames.length} file(s) to destination`);
    } catch (error) {
      console.error("Error moving files:", error);
      addStatusLog("Error occurred while moving files");
    } finally {
      setIsMoving(false);
    }
  };

  // Function to toggle visibility of moved files in destination directory
  const toggleFilesVisibility = async () => {
    if (!destinationPath) {
      addStatusLog("Please select destination directory first");
      return;
    }

    if (movedFiles.length === 0) {
      addStatusLog("No files have been moved yet");
      return;
    }

    try {
      if (showHidden) {
        // Hide the moved files
        await invoke("hide_files_in_directory", { 
          directory: destinationPath,
          files: movedFiles
        });
        setShowHidden(false);
        addStatusLog(`Hidden ${movedFiles.length} moved file(s) in destination directory`);
      } else {
        // Show the moved files
        await invoke("show_files_in_directory", { 
          directory: destinationPath,
          files: movedFiles
        });
        setShowHidden(true);
        addStatusLog(`Shown ${movedFiles.length} moved file(s) in destination directory`);
      }
    } catch (error) {
      console.error("Error toggling file visibility:", error);
      addStatusLog("Error occurred while toggling file visibility");
    }
  };

  // Helper function to add timestamped status log
  const addStatusLog = (message: string) => {
    const timestamp = new Date().toISOString().replace('T', ' ').substring(0, 19);
    const logEntry = `[${timestamp}] ${message}`;
    setStatusLog(prev => [logEntry, ...prev]);
  };

  return (
    <main className="container">
      <h1>File Move Utility</h1>
      
      <div className="card">
        <div className="input-group">
          <label>Source Directory:</label>
          <div className="path-display">
            <span className="path-text">{sourcePath || "No source directory selected"}</span>
            <button onClick={selectSourceDirectory}>Browse</button>
          </div>
        </div>
        
        <div className="input-group">
          <label>Destination Directory:</label>
          <div className="path-display">
            <span className="path-text">{destinationPath || "No destination directory selected"}</span>
            <button onClick={selectDestinationDirectory}>Browse</button>
          </div>
        </div>
        
        <div className="button-group">
          <button 
            onClick={moveFiles} 
            disabled={isMoving || !sourcePath || !destinationPath}
            className="move-button"
          >
            {isMoving ? "Moving..." : "Move Files"}
          </button>
          <button 
            onClick={toggleFilesVisibility}
            disabled={!destinationPath || movedFiles.length === 0}
          >
            {showHidden ? "Hide Moved Files" : "Show Moved Files"}
          </button>
        </div>
        
        <div className="status-log">
          <h3>Status Log</h3>
          <div className="log-entries">
            {statusLog.length > 0 ? (
              statusLog.map((log, index) => (
                <div key={index} className="log-entry">{log}</div>
              ))
            ) : (
              <div className="log-entry">[Ready]</div>
            )}
          </div>
        </div>
      </div>
    </main>
  );
}

export default App;
