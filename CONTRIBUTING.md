# Contributing to Calblend

Thank you for your interest in contributing to Calblend! We welcome contributions from the community and are grateful for any help you can provide.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How to Contribute

### Reporting Issues

- Check if the issue already exists in our [issue tracker](https://github.com/calblend/calblend/issues)
- Include a clear description of the problem
- Provide steps to reproduce the issue
- Include relevant error messages and logs
- Mention your environment (OS, Node.js version, Rust version)

### Suggesting Features

- Open a [discussion](https://github.com/calblend/calblend/discussions) first to gauge interest
- Clearly describe the use case and benefits
- Consider how it fits with the project's goals
- Be open to feedback and alternative approaches

### Contributing Code

1. **Fork the repository** and create your branch from `main`
2. **Set up your development environment**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/calblend.git
   cd calblend
   npm install
   cargo build
   ```

3. **Make your changes**:
   - Follow the existing code style
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass

4. **Test your changes**:
   ```bash
   # Run Rust tests
   cargo test --workspace
   
   # Run TypeScript tests
   npm test
   
   # Check formatting
   cargo fmt --check
   npm run lint
   ```

5. **Commit your changes**:
   - Use conventional commit format: `type(scope): message`
   - Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
   - Example: `feat(core): add iOS calendar provider`

6. **Push to your fork** and submit a pull request

### Development Guidelines

#### Rust Code
- Follow Rust naming conventions and idioms
- Use `cargo fmt` for formatting
- Run `cargo clippy` and address warnings
- Document public APIs with doc comments
- Prefer safe code; justify any `unsafe` blocks

#### TypeScript Code
- Use TypeScript strict mode
- Follow the existing code style (2-space indentation)
- Avoid `any` types
- Document exported functions and types
- Use ESLint and Prettier

#### FFI Boundary
- Keep the FFI interface simple and safe
- Use `#[napi(object)]` for structured data
- Handle errors gracefully across the boundary
- Test both Rust and TypeScript sides

### Pull Request Guidelines

- **PR title** should follow conventional commit format
- **Description** should explain what and why (not how)
- **Link to related issues** using keywords (fixes, closes)
- **Keep PRs focused** - one feature/fix per PR
- **Add tests** for new functionality
- **Update documentation** as needed
- **Ensure CI passes** before requesting review

### Review Process

1. A maintainer will review your PR
2. Address any feedback or requested changes
3. Once approved, a maintainer will merge your PR
4. Your contribution will be included in the next release

## Development Setup

### Prerequisites
- Rust 1.88+ (install via [rustup](https://rustup.rs/))
- Node.js 20.11.0+ (install via [nvm](https://github.com/nvm-sh/nvm))
- npm 10.0.0+

### Building
```bash
# Build everything
npm run build

# Build in debug mode
npm run build:debug

# Build only Rust
cargo build --workspace

# Build only TypeScript
npm run build -w @calblend/calendar
```

### Testing
```bash
# Run all tests
npm test

# Run Rust tests only
cargo test --workspace

# Run TypeScript tests only
npm test -w @calblend/calendar

# Run with coverage
cargo tarpaulin
npm test -- --coverage
```

## Release Process

Releases are managed by maintainers following semantic versioning:
- **Major**: Breaking changes
- **Minor**: New features (backward compatible)
- **Patch**: Bug fixes (backward compatible)

## License

By contributing to Calblend, you agree that your contributions will be licensed under the [Elastic License 2.0](LICENSE).

## Questions?

Feel free to open a [discussion](https://github.com/calblend/calblend/discussions) or reach out to the maintainers.

Thank you for contributing to Calblend! 🎉