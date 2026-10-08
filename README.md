# Local File Sharer

แอปพลิเคชัน Desktop สำหรับแชร์ไฟล์ตรงจากเครื่องคอมพิวเตอร์ (Windows PC) เป็นลิงก์ดาวน์โหลดความเร็วสูง โดยไม่ต้องอัปโหลดไฟล์ขึ้นเซิร์ฟเวอร์ พัฒนาด้วย **Tauri Framework (Tauri v2)**, **Rust backend**, และ **Svelte 4 + TypeScript + Tailwind CSS**

---

## ✨ ฟีเจอร์หลัก (Key Features)

1. **แชร์ตรงจากเครื่อง ไม่ต้องอัปโหลด (Zero-Upload Direct Stream):**
   - ผู้รับดาวน์โหลดไฟล์ตรงจากเครื่องคอมพิวเตอร์ของคุณแบบเรียลไทม์
   - รองรับไฟล์ทุกประเภท ทุกนามสกุล และไม่จำกัดขนาดไฟล์ (KB ไปจนถึง 100GB+)
   - ประมวลผลแบบ Chunked Streaming ไม่กินแรม

2. **เมนูคลิกขวาใน Windows ("แชร์ลิ้ง"):**
   - คลิกขวาที่ไฟล์ใดๆ ใน File Explorer แล้วเลือก **"แชร์ลิ้ง" (Share Link)**
   - ระบบจะเปิดโปรแกรม เพิ่มไฟล์เข้าสู่รายการแชร์ และคัดลอกลิงก์ลงคลิปบอร์ดให้ทันทีแบบ Single-Instance

3. **HTTP 206 Partial Content (Pause / Resume & IDM Support):**
   - รองรับ HTTP Byte-Range requests เต็มรูปแบบ
   - ผู้รับสามารถใช้โปรแกรมเร่งความเร็วดาวน์โหลด เช่น **Internet Download Manager (IDM)** โหลดหลายท่อเต็มสปีด
   - สามารถกดหยุดชั่วคราว (Pause) และดาวน์โหลดต่อ (Resume) ได้เมื่อเน็ตหลุด ไม่ต้องเริ่มใหม่จาก 0%

4. **Dual-Link (ลิงก์สาธารณะ & ลิงก์ LAN):**
   - **Cloudflare Quick Tunnel (HTTPS):** เชื่อมต่ออินเทอร์เน็ตสาธารณะอัตโนมัติ ไม่ต้องตั้งค่า Router / Port Forward
   - **Local LAN URL:** สำหรับแชร์ให้คนในบ้านหรือออฟฟิศเดียวกันด้วยสปีดเต็มความเร็ว Wi-Fi / Gigabit LAN

5. **สถิติและการปรับแต่ง (Analytics & Customization):**
   - ติดตามจำนวนครั้งที่คนดาวน์โหลดไฟล์ (`ดาวน์โหลด N ครั้ง`) และจำนวนครั้งที่เข้าชม (`เข้าชม N ครั้ง`)
   - แก้ไขชื่อตัวอย่าง (Title) และคำอธิบาย (Description) สำหรับแสดงผลบน Discord / Facebook / LINE
   - ปุ่มย่อลิงก์ (Shorten URL) ผ่าน da.gd / spoo.me ในคลิกเดียว
   - ปุ่มยกเลิกการแชร์ (Revoke) ทันที ตัดการเชื่อมต่อทันทีแบบ 0ms

6. **ระบบการตั้งค่า (Settings):**
   - สลับภาษา **ไทย / English**
   - ทำงานในถาดระบบ (**System Tray**) ทำงานต่อเนื่องในพื้นหลังเมื่อปิดหน้าต่าง
   - เริ่มทำงานอัตโนมัติเมื่อเปิดเครื่องคอมพิวเตอร์ (**Auto Start**)
   - คัดลอกลิงก์อัตโนมัติเมื่อแชร์ไฟล์ (**Auto Copy Link**)
   - เปิด/ปิดเมนูคลิกขวาใน Windows (**Context Menu Toggle**)

---

## 🛠️ โครงสร้างเทคโนโลยี (Tech Stack)

- **Desktop Framework:** Tauri v2
- **Backend / Core Engine:** Rust 2021 + Axum + Tokio + Winreg + Cloudflared
- **Frontend / UI:** Svelte 4 + TypeScript
- **Styling:** Tailwind CSS + PostCSS
- **Bundler:** esbuild

---

## 🚀 วิธีการติดตั้งและรันในโหมดพัฒนา (Development)

### ติดตั้ง Dependencies
```bash
npm install
```

### รัน Frontend Watch Mode
```bash
npm run dev
```

### รัน Tauri Desktop App
```bash
npm run tauri dev
```

### บิลด์ไฟล์ติดตั้ง / Release Executable
```bash
npm run build
npm run tauri build
```
ไฟล์ `.exe` จะถูกสร้างในโฟลเดอร์ `src-tauri/target/release/`

---

## 🧪 การทดสอบระบบ (Automated Tests)
รัน Unit & Integration Tests ทั้งหมด:
```bash
cd src-tauri
cargo test
```
- `registry_test.rs`: ทดสอบการบันทึก, ค้นหา, อัปเดต, บันทึกลง Disk (`shares.json`), และการยกเลิกแชร์
- `stream_test.rs`: ทดสอบ HTTP 206 Byte-Range parsing และ streaming response
- `server_test.rs`: ทดสอบหน้า Landing Page (`/d/:id`) และ endpoint ดาวน์โหลดตรง (`/api/download/:id`)
- `settings_test.rs`: ทดสอบการบันทึกการตั้งค่า และการลงทะเบียน Windows Context Menu ใน Registry
- `shortener_test.rs`: ทดสอบระบบย่อลิงก์และ Tunnel Manager
