// Hoyo Activity - Tauri Frontend Controller
let invoke;

// Initialize Tauri IPC
function initTauri() {
  if (window.__TAURI__ && window.__TAURI__.core) {
    invoke = window.__TAURI__.core.invoke;
  } else {
    console.warn("Tauri IPC not found. Running in browser preview mode.");
    invoke = mockInvoke;
  }
}

// Mock invoke for browser preview if tested outside Tauri
async function mockInvoke(cmd, args) {
  console.log(`[Mock Invoke] ${cmd}`, args);
  if (cmd === "get_games") {
    return [
      {
        id: "gi",
        name: "Genshin Impact",
        short_name: "GI",
        icon: "icons/gi.png",
        accent_color: "#4EB9D0",
        last_play_formatted: "Hari ini, 05:28",
        last_play_relative: "16 jam lalu",
        is_running: false,
        is_detected: true,
        is_custom_path: false,
        install_path: "C:\\Program Files\\HoYoPlay\\games\\Genshin Impact Game",
        exe_path: "C:\\Program Files\\HoYoPlay\\games\\Genshin Impact Game\\GenshinImpact.exe",
      },
      {
        id: "hsr",
        name: "Honkai: Star Rail",
        short_name: "HSR",
        icon: "icons/hsr.png",
        accent_color: "#D3A158",
        last_play_formatted: "Kemarin, 07:07",
        last_play_relative: "1 hari lalu",
        is_running: false,
        is_detected: true,
        is_custom_path: false,
        install_path: "D:\\HoYoPlay\\Star Rail Games",
        exe_path: "D:\\HoYoPlay\\Star Rail Games\\StarRail.exe",
      },
      {
        id: "zzz",
        name: "Zenless Zone Zero",
        short_name: "ZZZ",
        icon: "icons/zzz.png",
        accent_color: "#E7EF2D",
        last_play_formatted: "Belum terdeteksi",
        last_play_relative: "Tidak ada log",
        is_running: false,
        is_detected: false,
        is_custom_path: false,
        install_path: null,
        exe_path: null,
      },
    ];
  }
  if (cmd === "is_autostart_enabled") return false;
  if (cmd === "relocate_game") return "D:\\Games\\ZenlessZoneZero Game";
  return null;
}

let toastTimer = null;
function showToast(message, duration = 3500) {
  const toast = document.getElementById("toast");
  toast.textContent = message;
  toast.classList.remove("hidden");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    toast.classList.add("hidden");
  }, duration);
}

async function loadGames() {
  const container = document.getElementById("games-container");
  try {
    const games = await invoke("get_games");
    container.innerHTML = "";

    games.forEach((game) => {
      const card = document.createElement("div");
      card.className = "game-card";

      // Color tints
      const hex = game.accent_color || "#3b82f6";

      let playButtonHtml = "";
      let relocateBtnHtml = "";
      let folderButtonHtml = "";

      if (game.is_running) {
        playButtonHtml = `<button class="btn-play running-state" disabled>🟢 Berjalan</button>`;
      } else if (!game.is_detected) {
        // Game is NOT detected: Prominent Relocate Button
        playButtonHtml = `<button class="btn-relocate primary" data-relocate-id="${game.id}">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/><line x1="12" y1="11" x2="12" y2="17"/><line x1="9" y1="14" x2="15" y2="14"/></svg>
          Relokasi Folder
        </button>`;
      } else {
        // Game IS detected: Normal Play Button
        playButtonHtml = `<button class="btn-play" data-game-id="${game.id}">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg>
          Mainkan
        </button>`;
      }

      if (game.install_path) {
        folderButtonHtml = `<button class="btn-folder" data-folder-path="${game.install_path}" title="Buka folder instalasi game: ${game.install_path}">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
        </button>`;
      }

      const customBadge = game.is_custom_path ? `<span class="custom-tag" title="Folder dipilih manual">Kustom</span>` : "";

      const timeRowHtml = game.is_detected
        ? `
            <span class="time-value">${game.last_play_formatted}</span>
            <span class="time-badge ${game.is_running ? "running" : ""}">${game.last_play_relative}</span>
            ${customBadge}
          `
        : `
            <span class="time-label" style="color: #f59e0b;">Folder game belum terdeteksi</span>
          `;

      const iconUrl = game.icon || `icons/${game.id}.png`;

      card.innerHTML = `
        <div class="game-left">
          <div class="game-icon-wrapper" style="border: 1px solid ${hex}40;">
            <img 
              src="${iconUrl}" 
              alt="${game.name}" 
              class="game-icon-img" 
              onerror="this.style.display='none'; this.nextElementSibling.style.display='flex';"
            />
            <div class="game-monogram" style="display: none; background: ${hex}18; color: ${hex};">
              ${game.short_name}
            </div>
          </div>
          <div class="game-details">
            <div class="game-name">${game.name}</div>
            <div class="game-time-row">
              ${timeRowHtml}
            </div>
          </div>
        </div>
        <div class="game-actions">
          ${playButtonHtml}
          ${folderButtonHtml}
          ${relocateBtnHtml}
        </div>
      `;

      container.appendChild(card);
    });

    // Attach event listeners: Play
    container.querySelectorAll(".btn-play[data-game-id]").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const gameId = btn.getAttribute("data-game-id");
        btn.disabled = true;
        btn.innerHTML = `<span style="font-size:11px;">Meluncurkan...</span>`;
        showToast("Meluncurkan game langsung...");

        try {
          const res = await invoke("launch_game", { gameId });
          showToast(res || "Game berhasil diluncurkan!");
          setTimeout(loadGames, 1500);
        } catch (err) {
          showToast(`Error: ${err}`);
          setTimeout(loadGames, 1000);
        }
      });
    });

    // Attach event listeners: Relocate
    container.querySelectorAll("[data-relocate-id]").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const gameId = btn.getAttribute("data-relocate-id");
        showToast("Pilih folder instalasi game pada jendela yang terbuka...");
        try {
          const newPath = await invoke("relocate_game", { gameId });
          if (newPath) {
            showToast(`Folder berhasil dihubungkan!`, 3000);
            loadGames();
          } else {
            showToast("Pemilihan folder dibatalkan.", 2000);
          }
        } catch (err) {
          showToast(`Error: ${err}`, 4500);
        }
      });
    });

    // Attach event listeners: Open Folder
    container.querySelectorAll(".btn-folder[data-folder-path]").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const path = btn.getAttribute("data-folder-path");
        try {
          await invoke("open_folder", { path });
        } catch (err) {
          showToast(`Gagal membuka folder: ${err}`);
        }
      });
    });
  } catch (err) {
    container.innerHTML = `<div style="color:#ef4444; padding:20px; text-align:center;">Gagal memuat data: ${err}</div>`;
  }
}

async function initAutostart() {
  const check = document.getElementById("check-autostart");
  try {
    const enabled = await invoke("is_autostart_enabled");
    check.checked = enabled;
  } catch {}

  check.addEventListener("change", async () => {
    try {
      await invoke("set_autostart", { enable: check.checked });
      showToast(check.checked ? "Auto-start diaktifkan saat Windows menyala" : "Auto-start dinonaktifkan");
    } catch (err) {
      showToast(`Gagal mengubah pengaturan autostart: ${err}`);
    }
  });
}

function initWindowControls() {
  document.getElementById("btn-refresh").addEventListener("click", () => {
    showToast("Memperbarui data...");
    loadGames();
  });

  document.getElementById("btn-minimize").addEventListener("click", () => {
    invoke("minimize_window");
  });

  document.getElementById("btn-close").addEventListener("click", () => {
    invoke("close_window");
  });
}

function initDrag() {
  const titlebar = document.querySelector(".titlebar");
  if (!titlebar) return;

  titlebar.addEventListener("mousedown", (e) => {
    // Only drag on left click (button 0)
    if (e.button !== 0) return;

    // Ignore interactive controls inside titlebar (window controls, buttons)
    if (e.target.closest("button, input, select, textarea, a, .btn-icon")) {
      return;
    }

    try {
      invoke("drag_window");
    } catch {}
  });
}

// Bootstrap
window.addEventListener("DOMContentLoaded", () => {
  initTauri();
  initWindowControls();
  initDrag();
  initAutostart();
  loadGames();

  // Auto-refresh every 10 seconds to update running states
  setInterval(loadGames, 10000);
});
