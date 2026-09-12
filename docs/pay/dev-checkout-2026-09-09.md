# EPSX dev checkout — 9 September 2026

ใช้เงินจำลองบน Anvil เครื่อง Mac Mini นี้เท่านั้น

**อัปเดต:** checkout รุ่นล่าสุดเปลี่ยนเป็น QR/token transfer โดยไม่เชื่อม wallet บน Pay ดู[คู่มือ QR checkout](qr-checkout-dev.md) ข้อมูล browser approve/pay และ release v7 ด้านล่างเป็นบันทึกของ flow เดิม

## เปิดใช้งาน

| บริการ | URL |
|---|---|
| เว็บหลัก / Plan | https://dev.epsx.io/plans |
| Admin catalog | https://dev-admin.epsx.io/plans |
| Admin รายการซื้อ | https://dev-admin.epsx.io/payments/epsx |
| Backend | https://dev-api.epsx.io/health |
| Pay merchant | https://dev-pay.epsx.io/ |
| Admin escrow / dispute | https://dev-admin.epsx.io/pay/merchant-escrows |
| ประวัติซื้อของผู้ใช้ | https://dev.epsx.io/account/payments |

## Wallet บนเครื่องนี้

เพิ่มเครือข่ายใน MetaMask: ชื่อ `EPSX Local Dev`, RPC `http://127.0.0.1:8545`, chain ID `31337`, native symbol `ETH` (Pay ใช้ชื่อ BNB สำหรับ native coin ของ test network นี้), ไม่ต้องใส่ explorer URL.
RPC bind เฉพาะ loopback จึงใช้ wallet ใน browser บน Mac Mini เครื่องนี้; การเปิดเว็บจากเครื่องอื่นไม่ทำให้เข้าถึง chain ได้

Anvil ใช้ mixed mining: ยืนยันเมื่อส่งธุรกรรม และสร้าง idle block ทุก 60 วินาทีเพื่อให้ expiry scanner เดินต่อเมื่อไม่มีการจ่ายเงิน หลังโหลด state ตัว runner `infrastructure/native/dev-anvil.py` ปรับเวลา block ให้ตรงนาฬิกาเครื่องก่อนให้บริการต่อ ไม่แก้ธุรกรรมเดิม ([Foundry time RPC](https://foundry-rs.github.io/foundry/anvil/eth/api/struct.EthApi.html#method.evm_set_time)).

Import private key จากไฟล์ส่วนตัว `/Users/fluke/.config/epsx/dev/keys/test-wallets.json` โดยเลือกตาม role ด้านล่าง ใช้ช่อง `private_key` ของ role นั้น อย่าส่งไฟล์นี้ผ่านเว็บหรือเก็บใน Git

| Role | Address |
|---|---|
| buyer | `0x7a2267d036084789486fE3c971F0f401610C5781` |
| merchant | `0x941C41e8271003317EFF87d4c78daf3416B12E2F` |
| admin | `0x4EF3EbE8EC8859AdD08E7A712405855DF96F2ba4` |
| treasury | `0x8C67dC6A934170fBaB52Bdc5CF95391712e729ac` |

- `buyer`: login เว็บหลัก เลือก Plan แล้วไป Pay; อนุมัติเหรียญและลงนาม Pay ทีละรายการ
- `merchant`: login Pay เพื่อดูธุรกรรมและลงนาม direct refund หรือ escrow refund
- `admin`: login dev-admin เพื่อแก้ catalog, ดู order, dispute resolution และ pause/resume
- `treasury`: รับค่าธรรมเนียมจาก contract

เริ่มต้นแต่ละ wallet มี native coin 10,000 และ mock USDT/USDC อย่างละ 1,000,000; ยอดหลังทดสอบจะต่างจากยอดเริ่มต้น
Token ทั้งสองมี **18 decimals** ดู address และ deployment blocks ใน [deployment.json](evidence/dev-20260909/deployment.json)

ราคา: 1 Day 5, 1 Month 99, API Personal 3999, API Company 6999, Lifetime 9999 USDT หรือ USDC ระยะเวลา 1/30/30/365 วัน และ Lifetime ไม่มีวันหมดอายุ หน้าเว็บหลักใช้ USDT เป็นค่าเริ่มต้น

เมื่อจ่ายแล้วกด `Back to EPSX purchases` ที่ Pay จากนั้น `Refresh status` ในประวัติซื้อ หากชำระสำเร็จแต่ webhook ยังไม่ยืนยันจะเห็นรอเปิดสิทธิ์ ให้ตรวจรายละเอียด order; backend เท่านั้นที่กำหนดราคาและให้สิทธิ์

## Native runtime

Config/keys/logs/state อยู่ใต้ `/Users/fluke/.config/epsx/dev/` โดย config และ keys เป็นไฟล์ 0600
Release อยู่ `/Users/fluke/.local/share/epsx/dev/releases/`; symlink `current` และ `previous` ใช้สลับรุ่น

```sh
cd /Users/fluke/Desktop/Work/epsx
python3 infrastructure/native/dev-control.py status
python3 infrastructure/native/dev-control.py restart bff-frontend
python3 infrastructure/native/dev-control.py restart epsx
python3 infrastructure/native/dev-control.py restart pay-service
python3 infrastructure/native/dev-control.py restart anvil
python3 infrastructure/native/dev-control.py restart tunnel
```

เริ่มหลัง unload: `python3 infrastructure/native/dev-control.py start all`
หยุด dev: `python3 infrastructure/native/dev-control.py stop all`
ดู logs: `tail -f /Users/fluke/.config/epsx/dev/logs/pay-service.log` (ชื่อ service อื่นใช้รูปแบบเดียวกัน)
LaunchAgents ใช้ชื่อ `com.epsx.dev.*`, RunAtLoad/KeepAlive และ throttle 10 วินาที

Anvil auto-mines เมื่อมี transaction, ใช้ 1 confirmation, บันทึก state ทุก 30 วินาทีและตอน graceful shutdown พร้อม historical states อย่าใช้ `kill -9` หรือสร้าง chain ใหม่ทับ state เดิม
MinIO dev แยกข้อมูลที่ `state/minio`, API `127.0.0.1:19100`, console `127.0.0.1:19101` และมี launchd ของ dev
PostgreSQL/Redis เป็น native host services เดิม; core ใช้ `epsx_dev`, Pay ใหม่ใช้ `epsx_payments_checkout_dev`

## Rollback

```sh
cd /Users/fluke/Desktop/Work/epsx
python3 infrastructure/native/dev-control.py rollback
```

สลับ native app release และ config snapshot รุ่นก่อน แล้ว restart เฉพาะแอป ไม่ย้อนฐานข้อมูล ไม่ย้อน Anvil และไม่แก้ DNS/production
Config snapshot อยู่ `~/.config/epsx/dev/config-snapshots/<release>/`
Backup ก่อน migrate อยู่ `~/.config/epsx/dev/backups/epsx_dev-before-checkout.dump` และ `epsx_payments_dev-before-checkout.dump`
การ restore DB เป็นงานแยก ต้องพิจารณาธุรกรรมที่เกิดหลัง backup ก่อนเสมอ

Cloudflare named tunnel `epsx-dev` (`db141d3c-cc9a-4785-bf2e-dcdffabd36c6`) ใช้ config `~/.config/epsx/dev/config/tunnel.yml`; ตรวจด้วย:

```sh
cloudflared tunnel --config /Users/fluke/.config/epsx/dev/config/tunnel.yml ingress validate
```

DNS เดิมที่สำรองไว้: `~/.config/epsx/dev/backups/dns-before.json` การ rollback แอปไม่คืน dev DNS ไปปลายทางเดิมโดยอัตโนมัติ

## หลักฐานและขอบเขตการทดสอบ

ดู [evidence](evidence/dev-20260909/) สำหรับ transaction hash, order/payment ID, webhook/entitlement status และ escrow scenarios
การทดสอบ chain เชิงลบ/reorg ใช้ PostgreSQL `epsx_merchant_check_*` และ Anvil port `38546` แยกจาก dev
SIWE/ธุรกรรมในหลักฐานอัตโนมัติลงนามด้วย private test wallets ผ่าน Foundry CLI; ไม่ใช่หลักฐานว่ากดยืนยันใน browser extension แล้ว
Browser login ผ่านแล้วด้วย wallet ที่เปิดอยู่ใน Chrome (`0x0305d127caaced896c7cf0e0579afc5384f97494`) ซึ่งเติมเหรียญจำลองบน Anvil แยกให้แล้ว; สถานะการทดสอบ approve/pay ใน browser จะบันทึกในรายงานส่งมอบ


## ผลการตรวจอัตโนมัติ

- Rust workspace: 2,219 passed, 0 failed; 35 integration cases ต้องตั้ง environment แยก จึง ignored ใน default suite
- PostgreSQL/Anvil merchant integration: passed (guest isolation, webhook signatures/retry, reorg/recovery)
- Core fulfillment integration: 4 passed (price units, HMAC, grants/refund, reordered webhook corrections)
- Catalog integration: passed (preserve metadata, preserve one-time duration, rollback all writes if permission update fails)
- Foundry: 28 passed, 0 failed
- Clippy all workspace/all targets/all features, formatting, browser Wasm package, asset verify และ no-node audit: passed
- HTTPS SIWE และ Secure/HttpOnly/host-only cookies: เว็บหลักและ admin passed; protected catalog/history HTML render passed
- Live dev: ทั้งห้า Plan, ราคา USDT/USDC, admin/owner isolation, direct refund, disabled Plan with frozen order, insufficient funds, checkout expiry และ escrow 12 scenarios + 4 pause/resume transactions passed

หลักฐาน default tests และ integration logs เก็บส่วนตัวที่ `/Users/fluke/.config/epsx/dev/logs/validation/`.


## จุดที่ยังไม่ผ่าน acceptance

Browser login → Plan → hosted Pay ผ่านจริงแล้ว และแก้ race ที่เคย redirect กลับ analytics หลังเปิด Pay แล้ว
ขั้น approve/pay, ยกเลิก wallet และเปลี่ยน chain ผ่าน browser extension ยังไม่เสร็จ: MetaMask รายงาน `wallet_requestPermissions` ค้างสำหรับ dev-pay และ Computer Use ถูก browser URL policy บล็อกไม่ให้เปิดหน้า MetaMask โดยตรง จึงไม่สามารถยืนยันคำขอนี้ผ่านเครื่องมือได้
ผู้ใช้ต้องเปิด MetaMask เองเพื่อตอบคำขอเชื่อมต่อ จากนั้นตรวจ Anvil 31337 แล้ว approve/pay ด้วยเหรียญจำลองที่เติมไว้ให้ wallet ใน Chrome; หน้า checkout ถูกเก็บไว้ใน browser สำหรับทำต่อ
ไม่ควรนับ CLI/Anvil transactions เป็นการทดสอบ wallet extension ผ่าน browser

ติดตามรอบ 07:58 น.: checkout เดิมหมดอายุผ่าน webhook HTTP 200 แล้ว และ browser สร้าง checkout ใหม่สำเร็จ ปุ่ม Pay ส่งคำขอไปถึงขั้นเพิ่ม Anvil 31337 (`wallet_addEthereumChain`) เพราะ MetaMask ยังไม่รู้จักเครือข่ายนี้ ยังไม่มีหลักฐาน approve/pay ของรอบใหม่ ดูหลักฐานการแก้ idle expiry และคง receipt เดิมใน `evidence/dev-20260909/e2e-idle-expiry-proof.json`.

Release ปัจจุบัน: `20260909-checkout-v7`; previous: `20260909-checkout-v6` (รุ่นก่อนแก้ hosted redirect). Rollback จะนำพฤติกรรมของรุ่นก่อนกลับมาด้วย
