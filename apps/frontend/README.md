## Version Control

This project follows [Semantic Versioning](https://semver.org/) (MAJOR.MINOR.PATCH):

- **MAJOR** version for incompatible API changes
- **MINOR** version for added functionality in a backward-compatible manner
- **PATCH** version for backward-compatible bug fixes

### Changelog Management
All notable changes are documented in [CHANGELOG.md](./CHANGELOG.md) following the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

When making changes:
1. Add entries under the [Unreleased] section in CHANGELOG.md
2. When releasing a new version:
   - Move [Unreleased] entries to a new version section
   - Update the version in package.json
   - Create a git tag for the version

Current version: 0.2.0
