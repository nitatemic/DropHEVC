const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

async function openDialog(options) {
  return await invoke("plugin:dialog|open", { options });
}

let queue = [];
let isEncoding = false;
let customOutputDir = null;

const dropzone = document.getElementById("dropzone");
const selectBtn = document.getElementById("select-files-btn");
const queueList = document.getElementById("queue-list");
const startBtn = document.getElementById("start-queue-btn");
const sameFolderCheck = document.getElementById("same-folder-check");
const outputFolderGroup = document.getElementById("output-folder-group");
const selectOutputBtn = document.getElementById("select-output-btn");
const outputPathSpan = document.getElementById("output-folder-path");
const missingFfmpegBanner = document.getElementById("missing-ffmpeg-banner");
const downloadFfmpegBtn = document.getElementById("download-ffmpeg-btn");

// Initialize
async function init() {
  try {
    const hasFfmpeg = await invoke("check_ffmpeg");
    if (!hasFfmpeg) {
      missingFfmpegBanner.classList.remove("hidden");
    }
  } catch (e) {
    console.error("Failed to check ffmpeg", e);
  }

  // Setup drag & drop for Tauri v2 using global events
  listen("tauri://drag-enter", () => {
    dropzone.classList.add("dragover");
  });
  
  listen("tauri://drag-leave", () => {
    dropzone.classList.remove("dragover");
  });
  
  listen("tauri://drag-drop", (event) => {
    dropzone.classList.remove("dragover");
    // Depending on Tauri v2 exact version, payload might be an array of paths or an object { paths: [...] }
    let paths = [];
    if (Array.isArray(event.payload)) {
      paths = event.payload;
    } else if (event.payload && event.payload.paths) {
      paths = event.payload.paths;
    }
    if (paths && paths.length > 0) {
      handleFiles(paths);
    }
  });

  // Since onDragDropEvent might be tricky depending on version, also add standard web drop event to dropzone just in case
  dropzone.addEventListener("dragover", (e) => {
    e.preventDefault();
    dropzone.classList.add("dragover");
  });
  dropzone.addEventListener("dragleave", () => dropzone.classList.remove("dragover"));
  dropzone.addEventListener("drop", (e) => {
    e.preventDefault();
    dropzone.classList.remove("dragover");
    // Web drop event doesn't give absolute paths easily in Tauri, rely on Tauri's event above.
    // However, if we are in browser fallback, this would be empty.
  });
}

downloadFfmpegBtn.addEventListener("click", async () => {
  downloadFfmpegBtn.disabled = true;
  downloadFfmpegBtn.innerText = "Téléchargement en cours...";
  try {
    await invoke("download_ffmpeg_command");
    missingFfmpegBanner.classList.add("hidden");
    alert("FFmpeg a été téléchargé et installé avec succès !");
  } catch (e) {
    alert("Erreur lors du téléchargement de FFmpeg: " + e);
    downloadFfmpegBtn.disabled = false;
    downloadFfmpegBtn.innerText = "Réessayer";
  }
});

selectBtn.addEventListener("click", async () => {
  const selected = await openDialog({
    multiple: true,
    filters: [{ name: "Videos", extensions: ["mp4", "mkv", "avi", "mov", "webm"] }]
  });
  if (selected) {
    handleFiles(Array.isArray(selected) ? selected : [selected]);
  }
});

sameFolderCheck.addEventListener("change", (e) => {
  if (e.target.checked) {
    outputFolderGroup.classList.add("hidden");
  } else {
    outputFolderGroup.classList.remove("hidden");
  }
});

selectOutputBtn.addEventListener("click", async () => {
  const selected = await openDialog({
    directory: true,
    multiple: false
  });
  if (selected) {
    customOutputDir = selected;
    outputPathSpan.innerText = selected;
  }
});

function handleFiles(paths) {
  for (const path of paths) {
    if (!queue.find(item => item.path === path)) {
      const id = Math.random().toString(36).substring(7);
      const filename = path.split(/[\\/]/).pop();
      const item = { id, path, filename, status: 'pending', progress: 0 };
      queue.push(item);
      addQueueItemDOM(item);
      
      // Async get duration
      invoke("get_duration", { path }).then(duration => {
        item.duration = duration;
      }).catch(console.error);
    }
  }
  updateStartButton();
}

function addQueueItemDOM(item) {
  const div = document.createElement("div");
  div.className = "queue-item";
  div.id = `item-${item.id}`;
  div.innerHTML = `
    <div class="info">
      <div>
        <div class="filename">${item.filename}</div>
        <div class="details">${item.path}</div>
      </div>
      <div>
        <span class="status" id="status-${item.id}">En attente</span>
        <button class="remove-btn" onclick="removeFromQueue('${item.id}')">X</button>
      </div>
    </div>
    <div class="progress-container">
      <div class="progress-bar" id="progress-${item.id}"></div>
    </div>
    <div class="stats" id="stats-${item.id}">
      <span>Vitesse: --</span>
      <span>Temps: --</span>
    </div>
  `;
  queueList.appendChild(div);
}

window.removeFromQueue = (id) => {
  if (isEncoding) return; // Prevent removal while encoding for now
  queue = queue.filter(i => i.id !== id);
  const el = document.getElementById(`item-${id}`);
  if (el) el.remove();
  updateStartButton();
}

function updateStartButton() {
  const pending = queue.some(i => i.status === 'pending' || i.status === 'error');
  startBtn.disabled = !pending || isEncoding;
}

startBtn.addEventListener("click", async () => {
  if (isEncoding) return;
  isEncoding = true;
  updateStartButton();

  for (const item of queue) {
    if (item.status === 'done') continue;

    item.status = 'encoding';
    updateItemDOM(item.id, 'encoding');

    let unlisten = null;
    try {
      unlisten = await listen(`progress_${item.id}`, (event) => {
        const { progress, speed, time } = event.payload;
        const progressEl = document.getElementById(`progress-${item.id}`);
        const statsEl = document.getElementById(`stats-${item.id}`);
        if (progressEl) progressEl.style.width = `${progress}%`;
        if (statsEl) {
          statsEl.innerHTML = `<span>Vitesse: ${speed}</span><span>Temps cur: ${time}</span>`;
        }
      });

      const outDir = sameFolderCheck.checked ? null : customOutputDir;
      const res = await invoke("convert_video", { 
        id: item.id, 
        inputPath: item.path,
        outputDir: outDir 
      });

      item.status = 'done';
      updateItemDOM(item.id, 'done');
      const progressEl = document.getElementById(`progress-${item.id}`);
      if (progressEl) progressEl.style.width = `100%`;

    } catch (e) {
      console.error(e);
      item.status = 'error';
      updateItemDOM(item.id, 'error');
      const statsEl = document.getElementById(`stats-${item.id}`);
      if (statsEl) {
        statsEl.innerHTML = `<span style="color:var(--error)">Erreur: ${e}</span>`;
      }
    } finally {
      if (unlisten) unlisten();
    }
  }

  isEncoding = false;
  updateStartButton();
  
  // Notification visuelle dans la fenêtre principale
  const successBanner = document.getElementById("success-banner");
  if (successBanner) {
    successBanner.classList.remove("hidden");
  }
});

const closeSuccessBtn = document.getElementById("close-success-btn");
if (closeSuccessBtn) {
  closeSuccessBtn.addEventListener("click", () => {
    document.getElementById("success-banner").classList.add("hidden");
  });
}

function updateItemDOM(id, status) {
  const el = document.getElementById(`item-${id}`);
  const statusSpan = document.getElementById(`status-${id}`);
  if (!el || !statusSpan) return;

  el.className = `queue-item status-${status}`;
  if (status === 'encoding') statusSpan.innerText = 'Encodage...';
  if (status === 'done') statusSpan.innerText = 'Terminé';
  if (status === 'error') statusSpan.innerText = 'Erreur';
}

init();
