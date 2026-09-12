#!/usr/bin/env python3
"""Non-custodial local Anvil deployment console. Never signs or broadcasts.

Build contracts with forge first. Browser submits reviewed transactions through
MetaMask; this process independently checks receipts and runtime bytecode.
Evidence lives outside the checkout. No mainnet mode is intentionally provided.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import threading
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ADMIN = "0x0305d127caaced896c7cf0e0579afc5384f97494"
RPC = "http://127.0.0.1:8545"
CONTRACTS = {
    "MockUSDT": "BEP20Mock.sol", "MockUSDC": "BEP20Mock.sol",
    "DealEscrow": "DealEscrow.sol", "DirectPayments": "MerchantPayments.sol",
    "MerchantEscrow": "MerchantPayments.sol", "QRCheckout": "QRCheckout.sol",
}


def rpc(method, params):
    req = urllib.request.Request(RPC, json.dumps({"jsonrpc": "2.0", "id": 1,
        "method": method, "params": params}).encode(), {"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as response:
        result = json.load(response)
    if "error" in result:
        raise ValueError(f"RPC {method}: {result['error'].get('message', 'failed')}")
    return result["result"]


def require_local_anvil():
    if int(rpc("eth_chainId", []), 16) != 31337 or not rpc("web3_clientVersion", []).lower().startswith("anvil/"):
        raise ValueError("RPC is not local Anvil 31337")


def encode(signature, *args):
    return subprocess.check_output(["cast", "abi-encode", signature, *args], text=True).strip()[2:]


def runtime_matches(actual, artifact):
    actual = bytearray.fromhex(actual.removeprefix("0x"))
    expected = bytearray.fromhex(artifact["deployedBytecode"]["object"].removeprefix("0x"))
    if len(actual) != len(expected) or not actual:
        return False
    for refs in artifact["deployedBytecode"].get("immutableReferences", {}).values():
        for ref in refs:
            start, end = ref["start"], ref["start"] + ref["length"]
            actual[start:end] = expected[start:end]
    return actual == expected


class Deployment:
    def __init__(self, root, evidence):
        self.evidence = evidence
        self.lock = threading.Lock()
        self.artifacts = {}
        for name, source in CONTRACTS.items():
            artifact = json.loads((root / "out" / source / f"{name}.json").read_text())
            metadata = artifact["metadata"]
            if isinstance(metadata, str):
                metadata = json.loads(metadata)
            settings = metadata["settings"]
            if not metadata["compiler"]["version"].startswith("0.8.30+") or settings["evmVersion"] != "paris" or settings["optimizer"] != {"enabled": True, "runs": 200}:
                raise ValueError(f"{name}: unexpected compiler settings; rebuild pinned contracts")
            self.artifacts[name] = artifact
        self.records = json.loads(evidence.read_text()) if evidence.exists() else {}

    def save(self):
        tmp = self.evidence.with_suffix(".tmp")
        with open(tmp, "w", opener=lambda p, f: os.open(p, f, 0o600)) as out:
            json.dump(self.records, out, indent=2)
            out.flush()
            os.fsync(out.fileno())
        tmp.replace(self.evidence)

    def data(self, name):
        if name not in CONTRACTS:
            raise ValueError("Unknown contract")
        args = ""
        if name == "QRCheckout":
            args = encode("constructor(address)", ADMIN)
        elif name not in ("MockUSDT", "MockUSDC"):
            tokens = []
            for token in ("MockUSDT", "MockUSDC"):
                record = self.records.get(token, {})
                if record.get("status") != "verified":
                    raise ValueError("Verify both test tokens first")
                self.verify(token)
                tokens.append(record["address"])
            args = encode("constructor(address,address,address[])", ADMIN, ADMIN, "[" + ",".join(tokens) + "]")
        return self.artifacts[name]["bytecode"]["object"] + args

    def prepare(self, name):
        require_local_anvil()
        if name in self.records:
            raise ValueError("A transaction is already recorded; verify it before any retry")
        data = self.data(name)
        tx = {"from": ADMIN, "data": data, "value": "0x0", "chainId": "0x7a69"}
        gas = int(rpc("eth_estimateGas", [tx]), 16)
        price = int(rpc("eth_gasPrice", []), 16)
        tx["gas"] = hex((gas * 120 + 99) // 100)
        tx["gasPrice"] = hex(price)
        balance = int(rpc("eth_getBalance", [ADMIN, "latest"]), 16)
        cost = int(tx["gas"], 16) * price
        if balance < cost:
            raise ValueError("Insufficient local test BNB in the configured Admin wallet")
        return {"name": name, "transaction": tx, "maximumGasCostTBNB": str(cost / 10**18),
            "creationSha256": hashlib.sha256(bytes.fromhex(data[2:])).hexdigest()}

    def submitted(self, name, txhash):
        if name not in CONTRACTS or not re.fullmatch(r"0x[0-9a-fA-F]{64}", txhash):
            raise ValueError("Invalid transaction record")
        if name in self.records and self.records[name]["transactionHash"] != txhash:
            raise ValueError("A different transaction is already recorded")
        # Persist before RPC calls so a transient outage cannot lose a sent hash.
        self.records.setdefault(name, {"transactionHash": txhash, "status": "pending"})
        self.save()
        return self.verify(name)

    def verify(self, name):
        require_local_anvil()
        record = self.records[name]
        receipt = rpc("eth_getTransactionReceipt", [record["transactionHash"]])
        if receipt is None:
            return {**record, "status": "pending"}
        if int(receipt["status"], 16) != 1:
            raise ValueError("Transaction reverted; retained for explicit review")
        tx = rpc("eth_getTransactionByHash", [record["transactionHash"]])
        if tx is None or tx.get("to") is not None or tx["from"].lower() != ADMIN.lower() or int(tx.get("chainId", "0x0"), 16) != 31337 or int(tx["value"], 16) != 0 or tx["input"].lower() != self.data(name).lower():
            raise ValueError("Transaction does not match the reviewed local deployment")
        block = rpc("eth_getBlockByNumber", [receipt["blockNumber"], False])
        if block["hash"].lower() != receipt["blockHash"].lower():
            raise ValueError("Receipt is no longer canonical")
        confirmations = int(rpc("eth_blockNumber", []), 16) - int(receipt["blockNumber"], 16) + 1
        if confirmations < 1:
            return {**record, "status": "confirming", "confirmations": confirmations}
        address = receipt["contractAddress"]
        code = rpc("eth_getCode", [address, "latest"])
        if not runtime_matches(code, self.artifacts[name]):
            raise ValueError("Deployed runtime does not match the pinned build")
        record.update(status="verified", chainId=31337, address=address, blockNumber=int(receipt["blockNumber"], 16),
            blockHash=receipt["blockHash"], confirmations=confirmations,
            creationSha256=hashlib.sha256(bytes.fromhex(tx["input"][2:])).hexdigest(),
            runtimeSha256=hashlib.sha256(bytes.fromhex(code[2:])).hexdigest())
        self.save()
        return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True)
    parser.add_argument("--port", type=int, default=49197)
    args = parser.parse_args()
    args.evidence.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    root = Path(__file__).resolve().parents[2]
    deployment = Deployment(root / "apps/contracts", args.evidence)
    origin = f"http://127.0.0.1:{args.port}"

    class Handler(BaseHTTPRequestHandler):
        def reply(self, status, payload, kind="application/json"):
            body = payload if isinstance(payload, bytes) else json.dumps(payload).encode()
            self.send_response(status)
            self.send_header("Content-Type", kind)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.send_header("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'")
            self.end_headers()
            self.wfile.write(body)

        def trusted(self):
            return self.headers.get("Host") == f"127.0.0.1:{args.port}"

        def do_GET(self):
            if not self.trusted():
                return self.reply(403, {"error": "Loopback host required"})
            if self.path == "/state":
                return self.reply(200, {"chainId": 31337, "admin": ADMIN, "treasury": ADMIN,
                    "rpc": RPC, "contracts": list(CONTRACTS), "records": deployment.records})
            files = {"/": ("local-deploy.html", "text/html; charset=utf-8"),
                     "/app.js": ("local-deploy.js", "application/javascript"),
                     "/style.css": ("local-deploy.css", "text/css")}
            if self.path not in files:
                return self.reply(404, {"error": "Not found"})
            name, kind = files[self.path]
            self.reply(200, Path(__file__).with_name(name).read_bytes(), kind)

        def do_POST(self):
            if not self.trusted() or self.headers.get("Origin") != origin or self.headers.get("Content-Type") != "application/json":
                return self.reply(403, {"error": "Same-origin JSON required"})
            try:
                size = int(self.headers.get("Content-Length", "0"))
                if not 0 < size <= 1024:
                    raise ValueError("Invalid request size")
                body = json.loads(self.rfile.read(size))
                with deployment.lock:
                    if self.path == "/prepare":
                        result = deployment.prepare(body["name"])
                    elif self.path == "/submitted":
                        result = deployment.submitted(body["name"], body["hash"])
                    elif self.path == "/verify":
                        result = deployment.verify(body["name"])
                    else:
                        return self.reply(404, {"error": "Not found"})
                self.reply(200, result)
            except Exception as error:
                self.reply(400, {"error": str(error)})

    require_local_anvil()
    print(f"Local deployment console: {origin}; user signs in MetaMask", flush=True)
    ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()


if __name__ == "__main__":
    main()
