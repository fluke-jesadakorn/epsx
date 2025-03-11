export default {
  trace: () => Promise.resolve({}),
  wrap: (fn: Function) => fn,
  start: () => ({}),
  end: () => {},
};
