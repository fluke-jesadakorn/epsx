import yahooFinance from 'yahoo-finance2';
const symbol = 'AAPL';

try {
  const result = await yahooFinance.quote(symbol, {
    fields: ['symbol', 'regularMarketPrice'],
  });
  console.log(result);
} catch (error: unknown) {
  if (error instanceof yahooFinance.errors.FailedYahooValidationError) {
    // See the validation docs for examples of how to handle this
    // error.result will be a partially validated / coerced result.
  } else if (error instanceof yahooFinance.errors.HTTPError) {
    // Probably you just want to log and skip these
    console.warn(`Skipping yf.quote("${symbol}"): [${error}]`);
  } else {
    // Same here
    console.warn(`Skipping yf.quote("${symbol}"): [${error}]`);
  }
}
