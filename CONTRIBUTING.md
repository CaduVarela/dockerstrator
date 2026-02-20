# Contributing to Dockerstrator

Thank you for considering contributing! Here are some guidelines:

## Commit Convention

We use [conventional commits](https://github.com/iuricode/padroes-de-commits) to keep the history clean and organized.

### Structure

```
[type]: [description]
```

### Commit Types

| Type | Description |
|------|-------------|
| feat | Adds a new feature |
| fix | Fixes a bug |
| docs | Changes documentation |
| style | Formats code (no logic changes) |
| refactor | Restructures code |
| perf | Improves performance |
| test | Adds/modifies tests |
| build | Changes dependencies |
| chore | Administrative tasks |
| ci | Changes CI/CD |
| cleanup | Removes commented code |
| remove | Removes obsolete features |

### Examples

```bash
git commit -m "feat: add select all services option"
git commit -m "fix: correct error when stopping services"
git commit -m "docs: update installation instructions"
git commit -m "perf: optimize service discovery"
```

## Reporting Bugs

- Describe the expected behavior vs actual behavior
- Include steps to reproduce
- Mention your Docker/Rust version

## Suggesting Improvements

- Clearly describe the desired functionality
- Explain why it would be useful
- List examples of other tools with similar features

## Pull Requests

1. Fork the repository
2. Create a branch for your feature (`git checkout -b feature/my-feature`)
3. Commit your changes following the convention above
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request describing your changes

## Local Development

```bash
# Clone repository
git clone https://github.com/your-username/dockerstrator.git
cd dockerstrator/orchestrator

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run
./target/release/dockerstrator
```

## Code Standards

- Use `cargo fmt` to format code
- Use `cargo clippy` to check for common issues

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
