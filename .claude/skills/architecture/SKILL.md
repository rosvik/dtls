---
name: architecture
description: Architecture principles for dtls. Use when making structural changes — adding features, reorganizing modules, introducing abstractions, or adding dependencies. Not needed for small localized fixes.
---

# dtls architecture principles

dtls is a small, focused CLI: it inspects a file and prints what it finds. Every architectural decision should keep it easy to understand end-to-end in one sitting. These principles guide changes to the structure — they describe where the code should head, not necessarily where it is today.

## 1. Optimize for deletion

The measure of good structure here is: **how hard is it to remove a feature?** Removing a piece of functionality (say, EXIF support) should mean deleting one module and the handful of lines that call it — not untangling shared state, trait hierarchies, or config plumbing.

- Keep each feature's logic self-contained, with a narrow entry point called from the main flow.
- Don't let features reach into each other. If two features need the same helper, that helper should be trivial and dependency-free — otherwise duplicate it.
- When adding a feature, ask: "if this is removed next year, what does the diff look like?" If the answer touches many files, restructure before merging.
- Prefer code that is easy to delete over code that is easy to extend. Speculative extension points are liabilities.

## 2. Simplicity

- Straight-line code over clever code. A function that reads top-to-bottom beats a small graph of indirections.
- Don't build for hypothetical future requirements. Solve the problem in front of you; the codebase is small enough to restructure when a real need arrives.
- Prefer plain functions and plain data (structs, enums) over traits, generics, and callbacks unless there are at least two real implementations today.
- If a change makes the code harder to explain, it's probably the wrong change even if it's "more correct" by some abstract standard.

## 3. Few, well-thought-out layers of abstraction

Abstractions are allowed — but each one must earn its place, and there should be few of them.

- Aim for a shallow structure: the main flow orchestrates, feature modules do the work, and a thin formatting/output layer presents it. Avoid layers whose only job is to call the next layer.
- When an abstraction is warranted (e.g. a common shape for "a section of output"), design it deliberately: name it well, keep its surface minimal, and make sure it pays for itself across several call sites.
- Never introduce an abstraction to hide one thing. Introduce it when it removes real duplication or isolates real variation.
- Rework or delete abstractions that no longer fit rather than working around them.

## 4. Dependencies

Each dependency is architecture you don't control.

- Look to established industry standards first (e.g. `clap`, `chrono`, `sha2`). A crate that is the de-facto standard for its job is usually the right call.
- If there is no established standard, build the solution yourself or pick a small, focused crate — something you could vendor or replace in an afternoon.
- Avoid frameworks. Anything that wants to own the program's structure or control flow is off the table; dtls's structure belongs to dtls.
- Before adding a dependency, check whether std or an existing dependency already covers it.

## 5. Isolate platform-specific code

- OS-specific logic (macOS xattrs, plist metadata, etc.) lives in dedicated modules behind `#[cfg(target_os = ...)]` gates.
- The rest of the code calls it through a narrow seam so other platforms compile cleanly and the platform module remains deletable (see principle 1).
- Don't scatter `cfg` blocks through shared code — if a function needs more than one, the platform-specific part should move into its own module.
