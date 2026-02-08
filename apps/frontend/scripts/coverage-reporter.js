class CoverageReporter {
    onBegin(_config, _suite) { }
    onTestBegin(_test) { }
    onTestEnd(_test, _result) { }
    onEnd(_result) { }
}
module.exports = CoverageReporter;
