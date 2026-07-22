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
  const reloads = [];
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
    reload() {
      reloads.push("reload");
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
    reloads,
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

test("automatic recovery is one-shot and reloads once after rotation", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => response(200, "rotated", { success: true, authenticated: true }),
  });
  const first = tab.auth.recover();
  const second = tab.auth.recover();
  assert(first === second, "duplicate recovery callers did not share one promise");
  await Promise.all([first, second]);
  assert(tab.calls.length === 1, "automatic recovery retried refresh");
  assert(tab.reloads.length === 1, "successful automatic recovery did not reload exactly once");
  assert(tab.redirects.length === 0, "successful automatic recovery redirected instead of reloading");
});

test("automatic recovery preserves the page on explicit non-rotation", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => response(503, "preserved", { success: false, error: "refresh_not_rotated" }),
  });
  await rejects(tab.auth.recover(), "preserved automatic recovery unexpectedly succeeded");
  await rejects(tab.auth.recover(), "a second preserved recovery unexpectedly succeeded");
  assert(tab.calls.length === 1, "preserved automatic recovery retried");
  assert(tab.reloads.length === 0 && tab.redirects.length === 0, "preserved recovery navigated");
});

test("contradictory preserved success never reloads or retries", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => response(200, "preserved", { success: true, authenticated: true }),
  });
  await rejects(tab.auth.recover(), "contradictory preserved response unexpectedly recovered");
  assert(tab.calls.length === 1, "contradictory preserved response retried");
  assert(tab.reloads.length === 0 && tab.redirects.length === 0, "contradictory preserved response navigated");
});

test("missing or invalid refresh state requires confirmed best-effort clearing", async () => {
  for (const unknownState of [null, "unknown", "cleared, cleared"]) {
    let request = 0;
    const unconfirmed = realm({
      locks: lockManager(),
      fetchImpl: async () => {
        request += 1;
        return request === 1
          ? response(200, unknownState, { success: true, authenticated: true })
          : response(502, null, { success: false });
      },
    });
    await rejects(unconfirmed.auth.recover(), `unknown state ${unknownState} unexpectedly recovered`);
    assert(unconfirmed.calls.length === 2, `unknown state ${unknownState} skipped one clear attempt`);
    assert(unconfirmed.redirects.length === 0 && unconfirmed.reloads.length === 0, `unknown state ${unknownState} navigated without confirmed clearing`);

    request = 0;
    const confirmed = realm({
      locks: lockManager(),
      fetchImpl: async () => {
        request += 1;
        return request === 1
          ? response(200, unknownState, { success: true, authenticated: true })
          : response(502, "cleared", { success: false });
      },
    });
    await rejects(confirmed.auth.recover(), `confirmed unknown state ${unknownState} unexpectedly recovered`);
    assert(confirmed.calls.length === 2, `confirmed unknown state ${unknownState} retried refresh`);
    assert(confirmed.redirects.length === 1 && confirmed.reloads.length === 0, `confirmed unknown state ${unknownState} did not navigate exactly once`);
  }
});

test("automatic recovery navigates only after confirmed clearing", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => response(401, "cleared", { success: false, error: "refresh_rejected" }),
  });
  await rejects(tab.auth.recover(), "cleared automatic recovery unexpectedly succeeded");
  assert(tab.calls.length === 1, "cleared automatic recovery retried");
  assert(tab.redirects.length === 1, "confirmed clear did not navigate exactly once");
  assert(tab.reloads.length === 0, "confirmed clear also reloaded");
});

test("automatic recovery transport ambiguity without clear confirmation does not navigate", async () => {
  const tab = realm({
    locks: lockManager(),
    fetchImpl: async () => {
      throw new Error("BFF unavailable");
    },
  });
  await rejects(tab.auth.recover(), "ambiguous automatic recovery unexpectedly succeeded");
  assert(tab.calls.length === 2, "ambiguous recovery did not make one refresh and one clear attempt");
  assert(tab.redirects.length === 0 && tab.reloads.length === 0, "unconfirmed clearing navigated");
});

test("automatic recovery without Web Locks refuses all network I/O", async () => {
  const tab = realm({
    locks: null,
    fetchImpl: async () => response(200, "rotated", { success: true, authenticated: true }),
  });
  await rejects(tab.auth.recover(), "unsupported automatic recovery unexpectedly succeeded");
  assert(tab.calls.length === 0, "unsupported automatic recovery reached the network");
  assert(tab.redirects.length === 0 && tab.reloads.length === 0, "unsupported recovery navigated");
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

test("recovery and SIWE cookie establishment use the same exclusive lock", async () => {
  const locks = lockManager();
  const events = [];
  let releaseRefresh;
  let markRefreshStarted;
  const refreshGate = new Promise((resolveGate) => {
    releaseRefresh = resolveGate;
  });
  const refreshStarted = new Promise((resolveStarted) => {
    markRefreshStarted = resolveStarted;
  });
  const recoveringTab = realm({
    locks,
    fetchImpl: async (path) => {
      assert(path.endsWith("/refresh"), "recovery used the wrong endpoint");
      events.push("refresh-start");
      markRefreshStarted();
      await refreshGate;
      events.push("refresh-end");
      return response(200, "rotated", { success: true, authenticated: true });
    },
  });
  const loginTab = realm({
    locks,
    fetchImpl: async (path) => {
      assert(path.endsWith("/siwe"), "SIWE used the wrong endpoint");
      events.push("siwe-start");
      return response(200, null, { success: true, authenticated: true });
    },
  });

  const recovering = recoveringTab.auth.recover();
  await refreshStarted;
  const loggingIn = loginTab.auth.siweLogin("message", "signature", "0xabc", "nonce", "56");
  await new Promise((resolveDelay) => setTimeout(resolveDelay, 5));
  assert(events.join(",") === "refresh-start", "SIWE overtook recovery inside the mutation lock");
  releaseRefresh();
  await Promise.all([recovering, loggingIn]);
  assert(
    events.join(",") === "refresh-start,refresh-end,siwe-start",
    "recovery and SIWE mutation order drifted",
  );
  assert(locks.maximum === 1, "recovery and SIWE cookie establishment overlapped");
  assert(new Set(locks.names).size === 1, "recovery and SIWE used different mutation locks");
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
