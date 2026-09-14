# EPSX QR checkout — dev

**Update:** Checkout now also offers optional MetaMask / WalletConnect. See [wallet checkout guide](wallet-checkout-dev.md) for the current release and configuration requirements.

หน้า Plan → EPSX Pay จ่ายด้วยการโอน USDT/USDC ตรงไปยัง address เฉพาะ checkout ไม่เรียก Connect wallet, approve หรือการลงนามจาก browser ผู้ซื้ออาจใช้ wallet ภายนอกที่ต่างจากบัญชี EPSX ได้ สิทธิ์ถูกผูกกับเจ้าของ order ใน backend

UI ใช้โลโก้ EPSX, สีและ dark theme จาก `apps/frontend/public/enterprise.css`, ฟอนต์หลักและหัวข้อ editorial จัดสรุป Plan และส่วนจ่ายเงินแยกกัน มี QR, ปุ่มคัดลอกยอด/address, เวลาหมดอายุ และสถานะชำระ/สิทธิ์แยกกัน

## ทดลอง

1. เปิด https://dev.epsx.io/plans แล้วเลือก Plan → Continue to Pay
2. หน้า Pay แสดง QR พร้อม token, network และยอดที่ backend กำหนด
3. โอน **ยอดเต็มในครั้งเดียว** ไปยัง address ที่แสดง ใช้ USDT/USDC จำลองบน Anvil 31337 เท่านั้น
4. หน้า Pay ตรวจสถานะทุก 4 วินาที หลังยืนยัน block และ webhook สำเร็จ ให้ตรวจสิทธิ์ที่ https://dev.epsx.io/account/payments
5. Admin ตรวจ order เดียวกันที่ https://dev-admin.epsx.io/payments/epsx

RPC ของ dev คือ `http://127.0.0.1:8545` ใช้ wallet บน Mac Mini นี้ โทรศัพท์สแกน QR แล้วจะยังเข้าถึง local chain ไม่ได้ เครือข่ายนี้ไม่มี public RPC

QR ใช้ [ERC-681 token transfer URI](https://eips.ethereum.org/EIPS/eip-681) สำหรับ wallet ที่รองรับ และมีการคัดลอกข้อมูลแยกสำหรับ wallet อื่น ไม่ใช้ QR บริการภายนอก ไม่ส่ง checkout token ให้ผู้สร้าง QR

## รับเงินและคืนเงิน

`QRCheckout` factory สร้าง address ด้วย CREATE2 โดยผูก merchant, token, amount และ salt ที่ไม่ซ้ำ เงินที่โอนจะอยู่ใน receiver ที่นำส่งได้เฉพาะ merchant/treasury ตาม contract ไม่มี private key สำหรับรับเงินใน backend

Pay scanner ตรวจ ERC-20 Transfer log จาก token ที่อนุญาต, receipt, canonical block และ confirmation ก่อนบันทึกสำเร็จ จากนั้น backend อ่าน Pay API และตรวจ snapshot ของ order ก่อนเปิดสิทธิ์ตาม webhook รายการที่ยอดผิดหรือเข้าหลังหมดอายุจะเป็น `verification_required` และไม่เปิดสิทธิ์อัตโนมัติ ไม่รวมยอดโอนหลายครั้งโดยอัตโนมัติ

Merchant เปิดรายละเอียด payment ใน Pay แล้วใช้ **Settle to merchant wallet** เพื่อนำเงินจาก receiver เข้ากระเป๋า มีค่าธรรมเนียม 0.5% ตอนเก็บเงิน ผู้ใดเรียก factory ได้ แต่เปลี่ยนผู้รับไม่ได้ ส่วน **Refund full amount** ใช้ยอดเต็มจาก merchant wallet และตรวจผู้รับเงินคืนกับ payer ที่มีหลักฐานการโอน

Escrow และ checkout contract เดิมยังเก็บประวัติและวิธีชำระเดิม การเพิ่ม QR ใช้ migration แบบเพิ่ม field/index และขยาย operation type ไม่ล้างข้อมูล

## ปฏิบัติการ

ใช้คำสั่ง `python3 infrastructure/native/dev-control.py status|restart|rollback` ตาม[คู่มือ dev เดิม](dev-checkout-2026-09-09.md) เฉพาะ `com.epsx.dev.*` เท่านั้น

- `EPSX_PAY_CHECKOUT_METHOD=transfer` ใน config ส่วนตัวของ backend เลือก QR สำหรับ order ใหม่
- `PAY_MERCHANT_NETWORKS[].qr` กำหนด factory และ deployment block สำหรับ Pay scanner
- สำรอง core/Pay ก่อน migration อยู่ `~/.config/epsx/dev/backups/*-before-qr-20260909.dump`
- address/transaction ของ factory อยู่ `~/.config/epsx/dev/state/qr-deployment.json`
- ต้องคง runtime ที่รองรับ QR ขณะมี QR orders; binary รุ่นก่อน QR จะไม่ตรวจรายการเหล่านี้ แม้ข้อมูลยังอยู่ครบ
- Dev origins ไม่ใช้ service worker เพื่อป้องกันแคช runtime เก่าขัดขวางการทดสอบหลัง deploy

## Verification

Foundry 33 tests ผ่าน รวม receiver isolation, fees, collection และ merchant refund
PostgreSQL/Anvil integration ผ่าน USDT/USDC, guest capability, unique invoice, confirmation threshold, duplicate scans, incorrect amount, late transfer, collection, refund และ reorg recovery พร้อม regression ของ merchant direct/escrow เดิม
Rust tests ของ Pay service/BFF, browser runtime และ shared UI ผ่าน; Clippy ทั้ง workspace, browser Wasm และ asset verification ตรวจในรอบ deploy

หลักฐาน [browser → token transfer → webhook → entitlement](evidence/dev-20260909/qr-browser-proof.json) ตรวจครบ: ผู้ใช้สร้าง order ผ่าน browser, generated test buyer โอน 5 USDT ด้วย CLI, หน้า Pay อัปเดต Payment received อัตโนมัติ, webhook HTTP 200 ครั้งเดียว, หน้าประวัติผู้ซื้อแสดง succeeded / granted และ admin API เห็น hash เดียวกัน ใช้เงินจำลองเท่านั้น

Release ปัจจุบัน `20260909-qr-v4`; rollback ที่เตรียมไว้คือ `20260909-qr-v3` ซึ่งรองรับ QR เช่นกัน Runtime JS/Wasm ใช้ revision จาก content hash เดียวกัน และตอบ no-store; ล้างเฉพาะ URL runtime dev เก่าที่ Cloudflare เพื่อแก้แคช JS/Wasm คนละรุ่น

ตรวจ desktop ใน Chrome และ mobile ใน in-app browser ที่ content width 375px แล้ว: QR, address, ปุ่มคัดลอก และหน้าชำระสำเร็จแสดงครบ ไม่มี horizontal overflow คืนค่า viewport หลังทดสอบแล้ว

Restart Anvil และบริการ dev ทั้งหมดแล้ว receipt เดิมยังอยู่ และ order/สิทธิ์ยังเป็น succeeded / granted ตามเดิม
