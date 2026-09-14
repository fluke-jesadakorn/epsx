# คู่มือใช้งาน EPSX ฉบับภาพรวม

- PDF: `../../output/pdf/epsx-user-guide-th.pdf` (9 หน้า ภาษาไทย)
- ต้นฉบับสำหรับแก้ไข: `manual.html`
- ภาพจริงจากเบราว์เซอร์: `screenshots/` (8 ภาพ)
- จัดทำ: 14 กันยายน 2026 บน branch `development`

ครอบคลุมการเข้าสู่ระบบ, Company rankings, Saved companies, Plans,
Account/Developer, Admin และ EPSX Pay พร้อมข้อจำกัดและการแก้ปัญหาเบื้องต้น

## ขอบเขตหลักฐาน

ภาพมาจาก development servers ที่กำลังทำงานอยู่บน localhost:3000, :3001
และ :3002 ผ่าน Chromium/Playwright ขนาด 1440x1000 และเปิด JavaScript
ไม่ได้ดัดแปลง DOM หรือภาพ และไม่ได้ใช้ authentication bypass
ไม่มีการเข้าสู่ระบบ ทำธุรกรรม หรือเปลี่ยนข้อมูลแอปพลิเคชัน
ขั้นตอนที่ต้องเข้าสู่ระบบอ้างอิงจากโค้ดและหน้า merchant guide โดยระบุใน PDF

HEAD ขณะจัดทำ: `0d485028180cd96bc282256c9952c26254f96171`
มี uncommitted application changes อยู่ก่อนแล้ว ภาพจึงเป็นสถานะ dev server
ขณะ capture ไม่ใช่การรับรอง release ของ commit นี้

เส้นทางที่เปิดได้ HTTP 200:
- Frontend: `/`, `/analytics`, `/plans`, `/auth`, `/portfolio`, `/developer/docs`
- Admin: `/auth`
- Pay: `/`, `/docs/merchant?environment=test`, `/dashboard?environment=test`

`/portfolio` แสดงหน้าลงชื่อเข้าใช้ใน session ผู้เยี่ยมชม
ข้อความ rebuild ของ dev tooling อยู่ใน body ที่อ่านได้ แต่ไม่ปรากฏเป็น overlay
ในภาพที่เลือกใช้ และหน้า analytics หลัง hydration ไม่มี Preparing controls

## ส่งออก PDF ใหม่

ต้นฉบับ HTML ใช้ฟอนต์ Thonburi บน macOS เพื่อจัดรูปอักษรไทยผ่าน Chromium
ใช้ Playwright เป็นเครื่องมือเอกสารเท่านั้น ไม่ใช่ dependency ของแอป

```sh
mkdir -p output/pdf
NODE_PATH=/path/to/node_modules node docs/user-guide/export-pdf.cjs
pdftoppm -scale-to 1000 -png output/pdf/epsx-user-guide-th.pdf /tmp/epsx-manual-review
```

ติดตั้ง/เตรียม Playwright และ Chromium ในเครื่องมือภายนอก repository ก่อน
ตรวจภาพทุกหน้าหลังส่งออก โดยเฉพาะตำแหน่งสระ วรรณยุกต์ ตาราง และส่วนท้ายหน้า
ฟอนต์สำรองบนระบบอื่นอาจเปลี่ยนการตัดบรรทัด

## ตรวจไฟล์ส่งมอบ

PDF 9 หน้า A4; ตรวจภาพ render ครบทุกหน้าและหน้า 8 แบบขยาย
ตรวจไม่มีเนื้อหาล้นกรอบ HTML และดึงข้อความจาก PDF ได้
ไม่รัน build/test ของแอป เพราะเปลี่ยนเฉพาะเอกสารและเครื่องมือส่งออกเอกสาร
