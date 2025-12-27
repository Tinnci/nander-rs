# crates.io Release Checklist

## ✅ Pre-Release Checklist

### 1. **Code Quality**
- [x] All tests pass (`cargo test`)
- [x] No clippy warnings (`cargo clippy -- -D warnings`)
- [x] Code formatted (`cargo fmt -- --check`)
- [x] No unused dependencies

### 2. **Documentation**
- [x] README.md is complete and up-to-date
- [x] CHANGELOG.md exists (create if missing)
- [ ] All public APIs have doc comments
- [ ] Examples compile and run
- [x] User guides are complete

### 3. **Cargo.toml Metadata**
```toml
[package]
name = "nander-rs"
version = "0.5.0"  # ← Update this
edition = "2021"
authors = ["driezy <shisoratsu@icloud.com>"]
description = "A modern SPI NAND/NOR Flash programmer for CH341A with cross-platform GUI"
license = "MIT OR Apache-2.0"
repository = "https://github.com/tinnci/nander-rs"
documentation = "https://docs.rs/nander-rs"  # ← Add this
homepage = "https://github.com/tinnci/nander-rs"  # ← Add this
readme = "README.md"  # ← Add this
keywords = ["flash", "spi", "nand", "programmer", "ch341a"]
categories = ["embedded", "hardware-support", "command-line-utilities", "gui"]
```

**Required Fields:**
- ✅ name
- ✅ version
- ✅ authors or license
- ✅ edition

**Recommended Fields:**
- ✅ description (< 200 chars)
- ✅ license
- ✅ repository
- [ ] documentation
- [ ] homepage
- [ ] readme
- ✅ keywords (max 5)
- ✅ categories (max 5)

### 4. **License Files**
Ensure both license files exist in the repository root:
- [ ] LICENSE-MIT
- [ ] LICENSE-APACHE

Template content in section below.

### 5. **Version Management**
- [ ] Update version in `Cargo.toml` to `0.5.0`
- [ ] Create CHANGELOG.md with release notes
- [ ] Update version references in README.md

### 6. **Dependencies**
- [x] All dependencies use compatible versions (no git dependencies)
- [x] Optional dependencies are marked as `optional = true`
- [x] Features are properly documented

Check with:
```bash
cargo tree
```

### 7. **Build Testing**
Test on all target platforms:
```bash
# Windows
cargo build --release

# Test without default features
cargo build --no-default-features

# Test with all features
cargo build --all-features
```

### 8. **Package Verification**
```bash
# Dry-run packaging
cargo package --allow-dirty

# Check package contents
cargo package --list

# Verify the package builds
cargo publish --dry-run
```

### 9. **Git Repository**
- [x] All changes committed
- [x] Working directory is clean
- [ ] Create version tag: `git tag v0.5.0`
- [ ] Push tag: `git push origin v0.5.0`

### 10. **Documentation Build**
```bash
# Test documentation builds
cargo doc --no-deps --open

# Check for broken links
cargo deadlinks
```

---

## 📝 Required Actions

### Action 1: Create CHANGELOG.md

Create `CHANGELOG.md` in the project root:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-12-27

### Added
- **Cross-platform GUI** using `egui` with hex viewer and drag-and-drop support
- **WCH Device Database** with intelligent device recognition and error messages
- **Diagnostic Tools** for testing programmer without flash chip connected
- **Windows WinUSB Support** with automatic driver issue detection
- **Comprehensive Documentation** including troubleshooting guides
- Interactive SPI command tester for advanced users
- Real-time device enumeration and status reporting

### Improved
- Enhanced error messages with actionable solutions
- Optimized CH341A driver with bulk transfer support
- Better progress reporting in CLI and GUI
- Modernized architecture with Domain-Driven Design

### Fixed
- Windows driver compatibility issues
- Device mode detection (UART vs SPI)
- Memory management in large flash operations

## [0.4.0] - 2025-XX-XX
(Previous release notes...)
```

### Action 2: Create LICENSE-MIT

```
MIT License

Copyright (c) 2025 driezy

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Action 3: Create LICENSE-APACHE

```
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

   TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION
   
   (Full Apache 2.0 license text - see https://www.apache.org/licenses/LICENSE-2.0.txt)
```

### Action 4: Update Cargo.toml

Add these fields:
```toml
documentation = "https://docs.rs/nander-rs"
homepage = "https://github.com/tinnci/nander-rs"
readme = "README.md"
exclude = [
    "docs/*",
    "*.txt",
    ".github/*",
]
```

### Action 5: Add API Documentation

Ensure all public items have doc comments:
```rust
//! Module-level documentation

/// Brief description of the function
///
/// # Examples
///
/// ```
/// use nander_rs::something;
/// // example code
/// ```
pub fn something() { }
```

---

## 🚀 Publishing Commands

### Step 1: Login to crates.io
```bash
cargo login <YOUR_API_TOKEN>
```

Get token from: https://crates.io/settings/tokens

### Step 2: Dry Run
```bash
cargo publish --dry-run
```

### Step 3: Publish!
```bash
cargo publish
```

**Note**: Publishing is **permanent** and cannot be undone. You can only yank versions, not delete them.

---

## 📦 Post-Release

### 1. GitHub Release
- Create GitHub release for tag `v0.5.0`
- Attach binary builds (Windows/Linux/macOS)
- Copy CHANGELOG.md content to release notes

### 2. Update Repository
- Update README badge to show crates.io version
- Announce on relevant forums/channels

### 3. Monitor
- Watch for build failures on docs.rs
- Check for user issues/questions

---

## 🔍 Common Issues & Solutions

### Issue: "no license file found"
**Solution**: Add `LICENSE-MIT` and `LICENSE-APACHE` files

### Issue: "description too long"
**Solution**: Keep description under 200 characters in Cargo.toml

### Issue: "failed to verify package"
**Solution**: Run `cargo package --allow-dirty` to see what's being included

### Issue: "git dependencies are not allowed"
**Solution**: All dependencies must use version numbers, not git URLs

### Issue: "docs.rs build failed"
**Solution**: Test with `cargo doc --no-deps` and fix any issues

---

## 📊 Estimated Timeline

- **Documentation Review**: 1-2 hours
- **License Setup**: 15 minutes
- **Cargo.toml Updates**: 15 minutes
- **Testing & Verification**: 30 minutes
- **Publishing**: 10 minutes

**Total**: ~3 hours of careful preparation

---

## ✨ Final Checks Before Publishing

- [ ] Version bumped to 0.5.0
- [ ] CHANGELOG.md created
- [ ] LICENSE files added
- [ ] Cargo.toml metadata complete
- [ ] `cargo publish --dry-run` succeeds
- [ ] Git tag created and pushed
- [ ] Documentation builds without errors
- [ ] All tests pass
- [ ] Ready to commit!

---

Last Updated: 2025-12-27
