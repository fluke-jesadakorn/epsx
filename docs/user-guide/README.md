# คู่มือใช้งาน EPSX ฉบับภาพรวม

- PDF ภาษาไทย: `../../output/pdf/epsx-user-guide-th.pdf` (13 หน้า)
- PDF English: `../../output/pdf/epsx-user-guide-en.pdf` (13 pages)
- ต้นฉบับสำหรับแก้ไข: `manual.html` และ `manual-en.html`
- ภาพจริงจากเบราว์เซอร์: `screenshots/` (12 ภาพ)
- จัดทำ: 14 กันยายน 2026 บน branch `development`

ครอบคลุมการเข้าสู่ระบบ, Company rankings, Saved companies, Plans,
Account/Developer, Admin และ EPSX Pay พร้อมข้อจำกัดและการแก้ปัญหาเบื้องต้น

## ขอบเขตหลักฐาน

ภาพมาจาก development servers ที่กำลังทำงานอยู่บน localhost:3000, :3001
และ :3002 ผ่าน Chromium/Playwright ขนาด 1440x1000 และเปิด JavaScript
ภาพ stock-card.png จับเฉพาะองค์ประกอบการ์ดหุ้น
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

PDF 13 หน้า A4; ตรวจภาพ render ครบทุกหน้า และหน้าที่เพิ่มแบบขยาย
ตรวจไม่มีเนื้อหาล้นกรอบ HTML และดึงข้อความจาก PDF ได้
การปรับปรุงครั้งนี้เพิ่มรายละเอียดส่วนดูหุ้น 4 หน้า: อ่านการ์ดหุ้น, เปิดรายละเอียด/บันทึกหุ้น,
กรองประเทศ และเปลี่ยนหน้า/ตรวจสิทธิ์ พร้อมลบข้อความช่วยเหลือที่ผู้ใช้ขอออกจาก UI
ตรวจ rustfmt เฉพาะไฟล์ Rust ที่แก้ไข และตรวจหน้า Analytics ของ dev server หลังอัปเดต

การลองเพิ่มเติม: เลือก United States แล้ว URL เปลี่ยนเป็น country=america;
กดหัวใจในสถานะผู้เยี่ยมชมแล้วเปิด /auth พร้อม return_url กลับ Analytics
ตรวจลิงก์ View Details และ target จาก DOM โดยไม่ได้ตรวจเนื้อหาเว็บไซต์ภายนอก

## Country filter regression

สาเหตุ: ผู้ให้บริการตอบ HTTP 400 เมื่อ request range เริ่มเลยจำนวนหุ้นของประเทศนั้น
backend เดิมแปลงเป็น 502 ตอนนี้รับรู้เฉพาะ error รูปแบบดังกล่าวที่ตรงกับ range
ที่ร้องขอ และคืน empty page โดยรักษาจำนวนจริงและสิทธิ์อันดับเดิม
error อื่นยังคงเป็น error; response body ยังคงมีขนาดจำกัด

ก่อนแก้: UK, Germany, Singapore, Hong Kong, Vietnam, Australia, Bahrain และ
Iceland ตอบ 502 ในคำขอ visitor limit=10 หลังแก้ตรวจ 12 ประเทศได้ HTTP 200
ดู `evidence/country-api-results.json` (ตัวเลขขึ้นกับข้อมูลและสิทธิ์ขณะตรวจ)

การทดสอบ: cache/ranking access 18 ผ่าน; REST adapter 7 ผ่าน, 1 ignored
รวม 25 ผ่าน โดยมี regression ใหม่ 3 ข้อ และตรวจ rustfmt ของไฟล์ที่แก้ผ่าน

ตรวจ UI หลัง build สำเร็จ: HTTP 200, ไม่พบข้อความช่วยเหลือที่ลบแล้ว
สลับ America → Thailand → Japan → Germany → Thailand ผ่านทั้งหมด
ตรวจรหัสหุ้นทุกตัวบนหน้าจอตรงกับ API ของประเทศนั้น ผล Germany เป็น empty
และเปลี่ยนกลับ Thailand ได้ ดู `evidence/country-ui-results.json`
ภาพ `evidence/country-empty.png` แสดง empty state ตามสิทธิ์ visitor
ภาพ Analytics ในคู่มือถ่ายใหม่หลังแก้แล้ว โดย Country และ pagination ใช้ Thailand

ตรวจผ่าน https://dev.epsx.io/analytics เพิ่มเติม: HTTP 200,
Thailand 10 cards → Germany 0 cards → Japan 10 cards และไม่พบข้อความช่วยเหลือเดิม
PDF ทั้งสองภาษา 13 หน้า ตรวจ render ล่าสุดครบทุกหน้า รวมหน้า 6 และ 12 แบบขยาย
