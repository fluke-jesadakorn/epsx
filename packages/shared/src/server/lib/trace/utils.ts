export const detectTracer = () => null;
export const getTracer = () => ({
  startSpan: () => ({
    end: () => {},
  }),
});
