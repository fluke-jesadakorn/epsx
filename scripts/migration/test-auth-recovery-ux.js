#!/usr/bin/env bun

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import vm from "node:vm";

const root = resolve(import.meta.dir, "../..");
const pageSource = readFileSync(
  resolve(root, "shared/rust/dioxus_ui/src/pages/auth_page.rs"),
  "utf8",
);
const browserSource = readFileSync(
  resolve(root, "shared/rust/bff/src/browser_auth.rs"),
  "utf8",
);

const listenerMatch = pageSource.match(
  /const WALLET_STATUS_LISTENER_SCRIPT: &str = r#"([\s\S]*?)"#;/,
);
if (!listenerMatch) throw new Error("could not extract the auth-page listener");
const listenerScript = listenerMatch[1];

const bootstrapMatch = browserSource.match(
  /pub fn browser_session_recovery_script\(\) -> &'static str \{\s*"([^"\n]+)"\s*\}/,
);
if (!bootstrapMatch) throw new Error("could not extract the recovery bootstrap");
const recoveryBootstrap = bootstrapMatch[1];
const browserMatch = browserSource.match(
  /pub fn browser_auth_script\(\) -> &'static str \{\s*r#"\n([\s\S]*?)\n"#\s*\}/,
);
if (!browserMatch) throw new Error("could not extract the browser auth bridge");
const browserScript = browserMatch[1];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

class FakeElement {
  constructor({ hidden = false, disabled = false, text = "" } = {}) {
    this.hidden = hidden;
    this.disabled = disabled;
    this.textContent = text;
    this.focused = false;
    this.attributes = new Map();
    this.listeners = new Map();
    this.label = null;
  }

  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) || [];
    listeners.push(listener);
    this.listeners.set(type, listeners);
  }

  querySelector(selector) {
    return selector === ".connect-btn-label" ? this.label : null;
  }

  setAttribute(name, value) {
    this.attributes.set(name, String(value));
  }

  getAttribute(name) {
    return this.attributes.get(name) ?? null;
  }

  focus() {
    this.focused = true;
  }

  click() {
    for (const listener of this.listeners.get("click") || []) {
      listener({ preventDefault() {} });
    }
  }
}

class FakeCustomEvent {
  constructor(type, options = {}) {
    this.type = type;
    this.detail = options.detail;
  }
}

function response(status, sessionState, body) {
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: {
      get(name) {
        return name.toLowerCase() === "x-epsx-session-state" ? sessionState : null;
      },
    },
    async text() {
      return body === undefined ? "" : JSON.stringify(body);
    },
  };
}

function pageRealm({ fetchImpl = async () => { throw new Error("unexpected fetch"); } } = {}) {
  const cta = new FakeElement({ disabled: true });
  cta.label = new FakeElement({ text: "Connect Wallet" });
  const authPage = new FakeElement();
  authPage.setAttribute("data-auth-session-state", "recovering");
  authPage.setAttribute("aria-busy", "true");
  const status = new FakeElement({ hidden: false });
  const statusMessage = new FakeElement({ text: "Restoring your session..." });
  const error = new FakeElement({ hidden: true });
  const errorTitle = new FakeElement({ text: "Sign-in failed" });
  const errorMessage = new FakeElement();
  const elements = new Map([
    ["auth-card-status", status],
    ["auth-card-status-msg", statusMessage],
    ["auth-card-error", error],
    ["auth-card-error-title", errorTitle],
    ["auth-card-error-msg", errorMessage],
  ]);
  const listeners = new Map();
  const events = [];
  const calls = [];
  const reloads = [];
  const redirects = [];
  let lockCalls = 0;

  const document = {
    getElementById(id) {
      return elements.get(id) || null;
    },
    querySelector(selector) {
      if (selector === "[data-connect-wallet]") return cta;
      if (selector === "[data-auth-session-state]") return authPage;
      return null;
    },
    querySelectorAll() {
      return [];
    },
    addEventListener(type, listener) {
      const current = listeners.get(type) || [];
      current.push(listener);
      listeners.set(type, current);
    },
    dispatchEvent(event) {
      events.push({ type: event.type, detail: structuredClone(event.detail) });
      for (const listener of listeners.get(event.type) || []) listener(event);
      return true;
    },
  };

  class FakeBroadcastChannel {
    constructor(name) {
      this.name = name;
      this.onmessage = null;
    }

    postMessage() {}
  }

  const location = {
    origin: "https://app.test",
    search: "",
    assign(target) {
      redirects.push(target);
    },
    replace(target) {
      redirects.push(target);
    },
    reload() {
      reloads.push("reload");
    },
  };
  const window = { location, ethereum: undefined };

  const context = vm.createContext({
    window,
    document,
    navigator: {
      locks: {
        request(_name, options, operation) {
          assert(options?.mode === "exclusive", "session mutation lock was not exclusive");
          lockCalls += 1;
          return operation();
        },
      },
    },
    BroadcastChannel: FakeBroadcastChannel,
    CustomEvent: FakeCustomEvent,
    URL,
    URLSearchParams,
    Promise,
    JSON,
    fetch: async (path, options = {}) => {
      calls.push({ path, options: structuredClone(options) });
      return fetchImpl(path, options, calls.length);
    },
  });
  vm.runInContext(listenerScript, context, { filename: "auth_page_listener.js" });
  vm.runInContext(browserScript, context, { filename: "browser_auth.js" });

  return {
    context,
    document,
    events,
    cta,
    authPage,
    status,
    statusMessage,
    error,
    errorTitle,
    errorMessage,
    calls,
    reloads,
    redirects,
    get lockCalls() {
      return lockCalls;
    },
  };
}

async function settle() {
  await Promise.resolve();
  await Promise.resolve();
}

const tests = [];
function test(name, run) {
  tests.push({ name, run });
}

test("rejected recovery emits one exact closed event", async () => {
  const realm = pageRealm({
    fetchImpl: async () =>
      response(503, "preserved", {
        success: false,
        error: "hostile verifier detail",
      }),
  });
  vm.runInContext(recoveryBootstrap, realm.context, {
    filename: "auth_recovery_bootstrap.js",
  });
  await settle();
  await realm.context.window.epsxAuth.recover().catch(() => {});

  const recoveryEvents = realm.events.filter(
    (event) => event.type === "epsx:auth:recovery",
  );
  assert(realm.calls.length === 1, "rejected recovery retried the refresh request");
  assert(realm.calls[0].path === "/api/v1/auth/refresh", "recovery used a non-BFF path");
  assert(realm.calls[0].options.method === "POST", "recovery did not use POST");
  assert(
    realm.calls[0].options.credentials === "same-origin",
    "recovery did not use same-origin credentials",
  );
  assert(realm.lockCalls === 1, "recovery did not use one exclusive session lock");
  assert(realm.reloads.length === 0 && realm.redirects.length === 0, "rejected recovery navigated");
  assert(recoveryEvents.length === 1, "rejected recovery did not emit exactly one event");
  assert(
    JSON.stringify(recoveryEvents[0].detail) ===
      JSON.stringify({ version: 1, state: "failed" }),
    "recovery event included non-closed detail",
  );
});

test("resolved recovery emits no failure event", async () => {
  const realm = pageRealm({
    fetchImpl: async () =>
      response(200, "rotated", { success: true, authenticated: true }),
  });
  vm.runInContext(recoveryBootstrap, realm.context, {
    filename: "auth_recovery_bootstrap.js",
  });
  await settle();
  await realm.context.window.epsxAuth.recover();

  assert(realm.calls.length === 1, "successful recovery did not stay one-shot");
  assert(realm.lockCalls === 1, "successful recovery did not use one exclusive lock");
  assert(realm.reloads.length === 1, "rotated recovery did not reload exactly once");
  assert(realm.redirects.length === 0, "rotated recovery redirected instead of reloading");
  assert(
    realm.events.every((event) => event.type !== "epsx:auth:recovery"),
    "successful recovery emitted a failure event",
  );
});

test("invalid recovery events leave the recovering page closed", () => {
  const realm = pageRealm();
  for (const detail of [
    { version: 2, state: "failed", message: "hostile" },
    { version: 1, state: "success", message: "hostile" },
    { state: "failed", message: "hostile" },
  ]) {
    realm.document.dispatchEvent(new FakeCustomEvent("epsx:auth:recovery", { detail }));
  }

  assert(realm.cta.disabled, "invalid recovery event enabled the wallet action");
  assert(!realm.status.hidden, "invalid recovery event hid recovery progress");
  assert(realm.error.hidden, "invalid recovery event exposed an error state");
  assert(
    realm.authPage.attributes.get("data-auth-session-state") === "recovering",
    "invalid recovery event changed the closed state",
  );

  realm.authPage.setAttribute("data-auth-session-state", "verifier_unavailable");
  realm.status.hidden = true;
  realm.error.hidden = false;
  realm.errorTitle.textContent = "Sign-in temporarily unavailable";
  realm.document.dispatchEvent(
    new FakeCustomEvent("epsx:wallet:status", {
      detail: { status: "error", message: "stale wallet failure" },
    }),
  );
  realm.document.dispatchEvent(
    new FakeCustomEvent("epsx:wallet:status", { detail: { status: "idle" } }),
  );
  realm.cta.click();
  realm.document.dispatchEvent(
    new FakeCustomEvent("epsx:auth:recovery", {
      detail: { version: 1, state: "failed" },
    }),
  );
  assert(realm.cta.disabled, "a recovery event enabled the verifier-outage page");
  assert(realm.status.hidden, "a stale wallet event exposed progress on the verifier-outage page");
  assert(!realm.error.hidden, "a stale event hid the verifier-outage error");
  assert(
    realm.errorTitle.textContent === "Sign-in temporarily unavailable",
    "a stale event replaced the verifier-outage copy",
  );
  assert(
    realm.authPage.getAttribute("data-auth-session-state") === "verifier_unavailable",
    "a recovery event escaped the verifier-outage state",
  );
});

test("valid failure becomes fixed actionable UI without reflecting payload", () => {
  const realm = pageRealm();
  realm.document.dispatchEvent(
    new FakeCustomEvent("epsx:auth:recovery", {
      detail: {
        version: 1,
        state: "failed",
        message: "<img src=x onerror=alert(1)>",
        token: "do-not-reflect",
      },
    }),
  );

  assert(realm.status.hidden, "valid failure left recovery progress visible");
  assert(!realm.error.hidden, "valid failure did not expose the fixed error");
  assert(realm.error.focused, "valid failure did not focus the alert");
  assert(!realm.cta.disabled, "valid failure did not re-enable the wallet action");
  assert(realm.cta.label.textContent === "Try Again", "retry label did not become actionable");
  assert(
    realm.errorTitle.textContent === "Session recovery failed" &&
      realm.errorMessage.textContent ===
        "We could not restore your session. Try connecting your wallet again.",
    "valid failure did not use fixed copy",
  );
  const rendered = `${realm.errorTitle.textContent} ${realm.errorMessage.textContent}`;
  assert(!rendered.includes("<img") && !rendered.includes("do-not-reflect"), "payload leaked into UI");
  assert(
    realm.authPage.attributes.get("aria-busy") === "false" &&
      realm.authPage.attributes.get("data-auth-session-state") === "recovery_failed",
    "valid failure did not leave the recovering state",
  );

  realm.cta.click();
  assert(realm.cta.disabled, "retry click did not immediately disable the wallet action");
  assert(!realm.status.hidden, "retry click did not expose progress");
  assert(realm.statusMessage.textContent === "Opening wallet...", "retry click lacks immediate feedback");
});

let passed = 0;
for (const { name, run } of tests) {
  try {
    await run();
    passed += 1;
    console.log(`auth-recovery-ux: PASS ${name}`);
  } catch (error) {
    console.error(`auth-recovery-ux: FAIL ${name}`);
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
    break;
  }
}

if (passed === tests.length) {
  console.log(`auth-recovery-ux: PASS ${passed}/${tests.length}`);
}
