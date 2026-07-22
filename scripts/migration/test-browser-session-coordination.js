#!/usr/bin/env bun

import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import vm from "node:vm";

const root = resolve(import.meta.dir, "../..");
const source = readFileSync(resolve(root, "shared/rust/bff/src/browser_auth.rs"), "utf8");
const match = source.match(
  /pub fn browser_auth_script\(\) -> &'static str \{\s*r#"\n([\s\S]*?)\n"#\s*\}/,
);
if (!match) throw new Error("could not extract the browser bridge raw string");
const browserScript = match[1];

function response(status, state, body) {
  return {
    status,
    ok: status >= 200 && status < 300,
    headers: {
      get(name) {
        return name.toLowerCase() === "x-epsx-session-state" ? state : null;
      },
    },
    async text() {
      return body === undefined ? "" : JSON.stringify(body);
    },
  };
}

function lockManager() {
  const tails = new Map();
  let active = 0;
  let maximum = 0;
  const names = [];
  return {
    get maximum() {
      return maximum;
    },
    names,
    request(name, options, operation) {
      if (options?.mode !== "exclusive") throw new Error("lock must be exclusive");
      names.push(name);
      const previous = tails.get(name) || Promise.resolve();
      const current = previous
        .catch(() => {})
        .then(async () => {
          active += 1;
          maximum = Math.max(maximum, active);
          try {
            return await operation();
          } finally {
            active -= 1;
          }
        });
      tails.set(name, current);
      return current;
    },
  };
}

function channelBus() {
  const members = new Set();
  return {
    add(channel) {
      members.add(channel);
    },
    post(sender, message) {
      for (const channel of members) {
        if (channel !== sender && channel.name === sender.name && channel.onmessage) {
          channel.onmessage({ data: structuredClone(message) });
        }
      }
    },
  };
}

function realm({ locks, fetchImpl, bus = channelBus(), channelFailure = null }) {
  const calls = [];
  const messages = [];
  const events = [];
  const redirects = [];
  const listeners = new Map();

  class FakeChannel {
    constructor(name) {
      if (channelFailure === "constructor") throw new Error("channel denied");
      this.name = name;
      this.onmessage = null;
      bus.add(this);
    }
    postMessage(message) {
      if (channelFailure === "post") throw new Error("channel post denied");
      messages.push(structuredClone(message));
      bus.post(this, message);
    }
  }

  class FakeEvent {
    constructor(type, options) {
      this.type = type;
      this.detail = options?.detail;
    }
  }

  const document = {
    addEventListener(type, handler) {
      const handlers = listeners.get(type) || [];
      handlers.push(handler);
      listeners.set(type, handlers);
    },
    querySelectorAll() {
      return [];
    },
    dispatchEvent(event) {
      events.push({ type: event.type, detail: structuredClone(event.detail) });
      return true;
    },
  };
  const location = {
    origin: "https://app.test",
    search: "",
    assign(target) {
      redirects.push(target);
    },
    replace(target) {
      redirects.push(target);
    },
  };
  const window = { location, ethereum: undefined };
  const context = vm.createContext({
    window,
    document,
    navigator: locks ? { locks } : {},
    BroadcastChannel: FakeChannel,
    CustomEvent: FakeEvent,
    URL,
    URLSearchParams,
    Promise,
    JSON,
    fetch: async (path, options) => {
      calls.push({ path, method: options?.method || "GET" });
      return fetchImpl(path, options, calls.length);
    },
  });
  vm.runInContext(browserScript, context, { filename: "browser_auth.js" });
  return {
    auth: window.epsxAuth,
    calls,
    messages,
    events,
    redirects,
    click(target) {
      for (const handler of listeners.get("click") || []) {
        handler({ target, preventDefault() {} });
      }
    },
  };
}

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

async function rejects(promise, message) {
  let rejected = false;
  try {
    await promise;
  } catch (_) {
    rejected = true;
  }
  assert(rejected, message);
}

const tests = [];
function test(name, run) {
  tests.push({ name, run });
}

test("same-window refresh calls share one promise and one fetch", async () => {
  const locks = lockManager();
  const tab = realm({
    locks,
    fetchImpl: async () => response(200, "rotated", { success: true, authenticated: true }),
  });
  const first = tab.auth.refresh();
  const second = tab.auth.refresh();
  assert(first === second, "same-window refresh did not return the in-flight promise");
  await Promise.all([first, second]);
  assert(tab.calls.filter((call) => call.path.endsWith("/refresh")).length === 1, "refresh retried");
  assert(locks.maximum === 1, "more than one same-window mutation entered the lock");
});

test("two tab realms serialize refresh through one origin lock", async () => {
  const locks = lockManager();
  let activeFetches = 0;
  let maximumFetches = 0;
  const fetchImpl = async () => {
    activeFetches += 1;
    maximumFetches = Math.max(maximumFetches, activeFetches);
    await new Promise((resolveDelay) => setTimeout(resolveDelay, 5));
    activeFetches -= 1;
    return response(200, "rotated", { success: true, authenticated: true });
  };
  const first = realm({ locks, fetchImpl });
  const second = realm({ locks, fetchImpl });
  await Promise.all([first.auth.refresh(), second.auth.refresh()]);
  assert(maximumFetches === 1, "cross-tab refresh requests overlapped");
  assert(new Set(locks.names).size === 1, "tabs used different mutation lock names");
});

test("refresh and logout use the same exclusive lock", async () => {
  const locks = lockManager();
  let releaseRefresh;
  const refreshGate = new Promise((resolveGate) => {
    releaseRefresh = resolveGate;
  });
  const tab = realm({
    locks,
    fetchImpl: async (path) => {
      if (path.endsWith("/refresh")) {
        await refreshGate;
        return response(200, "rotated", { success: true, authenticated: true });
      }
      return response(502, "cleared", { success: false });
    },
  });
  const refreshing = tab.auth.refresh();
  await Promise.resolve();
  const loggingOut = tab.auth.logout();
  await new Promise((resolveDelay) => setTimeout(resolveDelay, 5));
  assert(tab.calls.length === 1, "logout overtook refresh inside the mutation lock");
  releaseRefresh();
  await Promise.all([refreshing, loggingOut]);
  assert(tab.calls.map((call) => call.path).join(",") === "/api/v1/auth/refresh,/api/v1/auth/logout", "mutation order drifted");
  assert(locks.maximum === 1, "refresh and logout overlapped");
});

test("missing Web Locks refuses refresh before network I/O", async () => {
  const tab = realm({
    locks: null,
    fetchImpl: async () => response(200, "rotated", { success: true }),
  });
  await rejects(tab.auth.refresh(), "unsupported refresh unexpectedly succeeded");
  assert(tab.calls.length === 0, "unsupported refresh reached the network");
});

test("cleared refresh failure broadcasts once and never retries", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => response(401, "cleared", { success: false, error: "refresh_rejected" }),
  });
  await rejects(tab.auth.refresh(), "rejected refresh unexpectedly succeeded");
  assert(tab.calls.length === 1, "rejected refresh retried");
  assert(tab.messages.length === 1, "rejected refresh did not publish exactly one event");
  assert(tab.messages[0].type === "session-ended", "wrong rejection event type");
  assert(tab.redirects.length === 1 && tab.redirects[0] === "/", "initiating tab did not leave stale UI");
});

test("cross-tab delivery reconstructs the closed event and redirects only the receiver", async () => {
  const bus = channelBus();
  const sender = realm({
    locks: lockManager(),
    bus,
    fetchImpl: async () => response(401, "cleared", { success: false }),
  });
  const receiver = realm({
    locks: lockManager(),
    bus,
    fetchImpl: async () => response(200, "rotated", { success: true }),
  });
  await rejects(sender.auth.refresh(), "rejected refresh unexpectedly succeeded");
  assert(sender.redirects.length === 1, "sender did not redirect exactly once");
  assert(receiver.redirects.length === 1 && receiver.redirects[0] === "/", "receiver did not redirect");
  const received = receiver.events.filter((event) => event.type === "epsx:auth:session");
  assert(received.length === 1, "receiver did not dispatch exactly one session event");
  assert(
    JSON.stringify(received[0].detail) === '{"version":1,"type":"session-ended","reason":"refresh_rejected"}',
    "receiver event schema drifted",
  );
  assert(receiver.messages.length === 0, "receiver echoed the broadcast");
});

test("preserved refresh failure does not end the session", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => response(503, "preserved", { success: false, error: "refresh_not_rotated" }),
  });
  await rejects(tab.auth.refresh(), "preserved failure unexpectedly succeeded");
  assert(tab.calls.length === 1, "preserved failure retried");
  assert(tab.messages.length === 0, "preserved failure broadcast session end");
  assert(tab.redirects.length === 0, "preserved failure redirected");
});

test("logout redirects only after local clearing is confirmed", async () => {
  const failed = realm({
    locks: lockManager(),
    fetchImpl: async () => {
      throw new Error("network down");
    },
  });
  await rejects(failed.auth.logout(), "unconfirmed logout unexpectedly succeeded");
  assert(failed.redirects.length === 0, "unconfirmed logout redirected as success");
  assert(failed.messages.length === 0, "unconfirmed logout broadcast session end");

  const cleared = realm({
    locks: lockManager(),
    fetchImpl: async () => response(502, "cleared", { success: false }),
  });
  await cleared.auth.logout();
  assert(cleared.redirects.length === 1 && cleared.redirects[0] === "/", "cleared logout did not redirect");
  assert(cleared.messages.length === 1 && cleared.messages[0].type === "session-ended", "cleared logout did not broadcast");
});

test("refresh transport ambiguity ends the session only after confirmed local clearing", async () => {
  const unreachable = realm({
    locks: lockManager(),
    fetchImpl: async () => {
      throw new Error("BFF unavailable");
    },
  });
  await rejects(unreachable.auth.refresh(), "unreachable refresh unexpectedly succeeded");
  assert(unreachable.calls.length === 2, "refresh ambiguity did not make exactly one clear attempt");
  assert(unreachable.messages.length === 0, "unconfirmed local clear broadcast session end");
  assert(unreachable.redirects.length === 0, "unconfirmed local clear redirected as success");

  let request = 0;
  const cleared = realm({
    locks: lockManager(),
    fetchImpl: async () => {
      request += 1;
      if (request === 1) throw new Error("refresh response lost");
      return response(502, "cleared", { success: false });
    },
  });
  await rejects(cleared.auth.refresh(), "lost refresh response unexpectedly succeeded");
  assert(cleared.calls.length === 2, "confirmed clear flow retried refresh or skipped logout");
  assert(cleared.messages.length === 1, "confirmed local clear did not broadcast exactly once");
  assert(cleared.redirects.length === 1, "confirmed local clear did not redirect exactly once");
});

test("BroadcastChannel failures degrade to same-tab events without breaking auth", async () => {
  const constructorFailure = realm({
    locks: lockManager(),
    channelFailure: "constructor",
    fetchImpl: async () => response(200, "rotated", { success: true, authenticated: true }),
  });
  await constructorFailure.auth.refresh();
  assert(constructorFailure.events.length === 1, "constructor failure suppressed same-tab event");

  const postFailure = realm({
    locks: lockManager(),
    channelFailure: "post",
    fetchImpl: async () => response(200, "rotated", { success: true, authenticated: true }),
  });
  await postFailure.auth.refresh();
  assert(postFailure.events.length === 1, "post failure suppressed same-tab event");
  assert(postFailure.redirects.length === 0, "successful refresh redirected after channel failure");
});

test("delegated authenticated-header logout click uses the shared controller", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => response(502, "cleared", { success: false }),
  });
  const button = {
    closest(selector) {
      return selector === "[data-epsx-logout]" ? this : null;
    },
    getAttribute(name) {
      return name === "data-epsx-logout-target" ? "/signed-out" : null;
    },
  };
  tab.click(button);
  await new Promise((resolveDelay) => setTimeout(resolveDelay, 0));
  assert(tab.calls.length === 1 && tab.calls[0].path.endsWith("/logout"), "header click bypassed logout");
  assert(tab.redirects.length === 1 && tab.redirects[0] === "/signed-out", "header target was not honored");
});

test("broadcast messages contain only the closed token-free schema", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => response(200, "rotated", { success: true, authenticated: true }),
  });
  await tab.auth.refresh();
  assert(tab.messages.length === 1, "successful refresh did not publish once");
  const serialized = JSON.stringify(tab.messages[0]).toLowerCase();
  for (const forbidden of ["access", "refresh_token", "bearer", "wallet", "user", "secret"]) {
    assert(!serialized.includes(forbidden), `broadcast leaked forbidden field: ${forbidden}`);
  }
  assert(serialized === '{"version":1,"type":"session-refreshed"}', "broadcast schema drifted");
});

let passed = 0;
for (const { name, run } of tests) {
  await run();
  passed += 1;
  console.log(`browser-session-coordination: PASS ${name}`);
}
console.log(`browser-session-coordination: PASS ${passed}/${tests.length}`);
