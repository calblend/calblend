# Repository Setup Instructions

## Initial Setup for GitHub Organization

Follow these steps to initialize the Calblend repository in the GitHub organization:

### 1. Clone the repository (if not already done)
```bash
git clone https://github.com/calblend/calblend.git
cd calblend
```

### 2. Initialize Git (if starting fresh)
```bash
# If you haven't initialized git yet
git init

# Add all files
git add .

# Initial commit
git commit -m "feat: initial Calblend project setup

- Rust workspace with calblend-core and calblend-ffi crates
- TypeScript/Node.js SDK package
- N-API FFI bindings using napi-rs
- Unified calendar object design
- Elastic License 2.0 (ELv2)
- Basic example application
- Monorepo structure with npm workspaces"

# Add the remote origin
git remote add origin https://github.com/calblend/calblend.git

# Push to main branch
git push -u origin main
```

### 3. Set up branch protection (after initial push)
In the GitHub repository settings:
1. Go to Settings → Branches
2. Add a branch protection rule for `main`
3. Enable:
   - Require pull request reviews before merging
   - Dismiss stale pull request approvals when new commits are pushed
   - Require status checks to pass before merging
   - Require branches to be up to date before merging
   - Include administrators

### 4. Configure repository settings
In the GitHub repository settings:
1. **General**:
   - Add description: "Unified calendar integration library for Node.js with Rust core"
   - Add topics: `calendar`, `google-calendar`, `outlook`, `rust`, `nodejs`, `typescript`, `ffi`
   - Enable Issues
   - Enable Projects
   - Enable Wiki (optional)

2. **Security**:
   - Enable Dependabot alerts
   - Enable Dependabot security updates

### 5. Create initial GitHub releases
```bash
# Create a pre-release tag
git tag -a v0.1.0-alpha.1 -m "Pre-release v0.1.0-alpha.1"
git push origin v0.1.0-alpha.1
```

### 6. Set up npm organization (if needed)
```bash
# Login to npm
npm login

# Create organization (if not exists)
# This is done through npmjs.com website

# Publish as scoped package when ready
# npm publish --access public
```

### 7. Development workflow
```bash
# Install dependencies
npm install

# Build the project
npm run build

# Run tests
npm test

# Run the example
npm run example:basic
```

## Next Steps

1. Set up CI/CD with GitHub Actions
2. Configure cross-platform builds
3. Add comprehensive test suite
4. Implement calendar provider integrations
5. Create detailed API documentation