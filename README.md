# Hoyo Activity Tracker & Direct Game Launcher 🎮 (Rust + Tauri)

Aplikasi Windows minimalis dan modern berbasis **Rust** dan **Tauri v2** untuk memantau waktu terakhir bermain serta **meluncurkan game secara langsung tanpa membuka HoYoPlay**:
* **Genshin Impact**
* **Honkai: Star Rail (HSR)**
* **Zenless Zone Zero (ZZZ)**

---

## ✨ Fitur Utama

1. **Buka Game Langsung Tanpa HoYoPlay**:
   * Setiap kartu game yang terdeteksi memiliki tombol **`▶ Mainkan`**.
   * Mengeklik tombol ini akan meluncurkan game secara langsung (`GenshinImpact.exe`, `StarRail.exe`, `ZenlessZoneZero.exe`) tanpa membuka launcher HoYoPlay sama sekali.
   * Tombol folder untuk membuka direktori instalasi game di File Explorer.

2. **Fitur Relokasi Folder (Jika Game Tidak Terdeteksi)**:
   * Jika game belum terdeteksi (atau dipindahkan ke SSD/drive lain), tombol pada kartu game akan otomatis berubah menjadi **`📂 Relokasi Folder`**.
   * Mengeklik tombol ini akan membuka **dialog pemilihan folder native Windows**:
     * Kamu cukup memilih folder instalasi game tersebut.
     * Aplikasi otomatis memeriksa keberadaan file executable utama (termasuk di dalam subfolder seperti `Star Rail Games`, `Genshin Impact Game`, dll).
     * Lokasi kustom otomatis disimpan permanen di `%APPDATA%\hoyo-last-play\custom_paths.json`.
     * Begitu folder valid dipilih, kartu game langsung menampilkan tombol **`▶ Mainkan`** dan label **`Kustom`**.
   * Untuk game yang sudah terdeteksi pun, terdapat tombol **`↻`** kecil di samping kanan jika sewaktu-waktu ingin memindahkan lokasi foldernya.

3. **Jendela Minimalis Modern**:
   * Desain dark mode frameless berestetika tinggi dengan sudut membulat.
   * Menampilkan nama game, tanggal/jam terakhir bermain (*Hari ini, 05:45*, *Kemarin, 07:07*), dan waktu relatif (*16 jam lalu*).
   * Status real-time: jika game sedang berjalan di komputer, kartu otomatis menampilkan badge **`🟢 AKTIF`**.

4. **Ikon Game Resmi & Jendela Aplikasi**:
   * Setiap kartu game menampilkan ikon resmi game (Genshin Impact, Honkai: Star Rail, Zenless Zone Zero).
   * Jendela aplikasi menggunakan ikon resmi default bawaan framework Tauri / Rust.

5. **Siap untuk Windows Startup**:
   * Dilengkapi opsi centang **"Buka saat startup"** di bagian bawah jendela agar aplikasi langsung terbuka otomatis saat komputer kamu dinyalakan.

---

## 🚀 Cara Menjalankan

1. **Jalankan Aplikasi**:
   * Cukup klik ganda [hoyo-last-play.exe](file:///c:/Files/Project/last-play-checker/hoyo-last-play.exe) di folder proyek ini, ATAU
   * Buka shortcut **`Hoyo Activity`** di Desktop kamu.

2. **Relokasi Folder (Jika Belum Terdeteksi)**:
   * Klik tombol **`📂 Relokasi Folder`** pada kartu game.
   * Pilih folder tempat game terpasang di hard drive kamu.

3. **Meluncurkan Game**:
   * Klik tombol **`Mainkan`** di sebelah game yang ingin kamu mainkan.

---

## 🛠️ Kompilasi Ulang (Build)

Jika kamu ingin memodifikasi kode:
* Jalankan `build.bat` atau perintah PowerShell:
  ```powershell
  .\build.ps1
  ```
* Binary hasil build otomatis diletakkan di `./hoyo-last-play.exe`.
