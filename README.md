# Base Libraries - Rust

Foundational Rust libraries providing common functionality for S-CORE modules.

---

## 📂 Project Structure

| File/Folder                                          | Description                                |
| ---------------------------------------------------- | ------------------------------------------ |
| `README.md`                                          | Short description and build instructions   |
| `src/`                                               | Source files                               |
| `tests/`                                             | Unit tests (UT) and integration tests (IT) |
| `examples/`                                          | Usage examples                             |
| `docs/`                                              | Documentation using `docs-as-code`         |
| `.github/workflows/`                                 | CI/CD pipelines                            |
| `.vscode`                                            | Recommended VS Code settings               |
| `.bazelrc`, `.bazelversion`, `MODULE.bazel`, `BUILD` | Bazel configuration and settings           |
| `Cargo.toml`, `rust-toolchain.toml`, `rustfmt.toml`  | Cargo configuration and settings           |
| `project_config.bzl`                                 | Project-specific metadata for Bazel macros |
| `LICENSE`, `LICENSE.md`                              | Licensing information                      |
| `CONTRIBUTION.md`                                    | Contribution guidelines                    |
| `NOTICE`                                             | Notices for Eclipse Safe Open Vehicle Core |

---

## 🚀 Getting Started

### 1️⃣ Clone the Repository

```sh
git clone https://github.com/eclipse-score/baselibs_rust.git
cd baselibs_rust
```

### 2️⃣ Build the Examples of module

> DISCLAIMER: Depending what module implements, it's possible that different
> configuration flags needs to be set on command line.

To build all targets of the module the following command can be used:

```sh
bazel build //src/...
```

This command will instruct Bazel to build all targets that are under Bazel
package `src/`. The ideal solution is to provide single target that builds
artifacts, for example:

```sh
bazel build //src/<module_name>:release_artifacts
```

where `:release_artifacts` is filegroup target that collects all release
artifacts of the module.

> NOTE: This is just proposal, the final decision is on module maintainer how
> the module code needs to be built.

### 3️⃣ Run Tests

```sh
bazel test //tests/...
```

---

## 🛠 Tools & Linters

**Tools and linters** from **centralized repositories** are integrated to ensure consistency across projects.

- **Rust:** `clippy`, `rustfmt`, `Rust Unit Tests`
- **CI/CD:** GitHub Actions for automated builds and tests

---

## 📖 Documentation

- A **centralized docs structure** is planned.
