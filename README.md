# Sirius Quickstart

Welcome to the `sirius-quickstart` repository, a minimal example to get you started with the [**Sirius**](https://github.com/snarkify/sirius/) framework for Incrementally Verifiable Computation (IVC).

## Introduction

This repository provides a simple, runnable example to demonstrate the basic usage of the Sirius framework. The example implements a trivial `StepCircuit` that performs identity mapping (i.e., it outputs the input unchanged) over multiple folding steps. This example is intended to help developers understand the core concepts and API of Sirius.

## Prerequisites

### 1. Install Rust

If you haven't already installed Rust, you can do so by using [rustup](https://rustup.rs/). Rustup will set up your environment with the latest stable Rust compiler and Cargo, Rust's package manager.

To install Rust, run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

After installation, make sure your Rust toolchain is up-to-date:

```bash
rustup update
```

### 2. Clone the Repository
Clone the sirius-quickstart repository to your local machine:

```bash
git clone https://github.com/your-username/sirius-quickstart.git;
cd sirius-quickstart
```

## Project Structure
The project structure is as follows:

- src/: Contains the source code for the example.
- Cargo.toml: The Cargo configuration file, listing dependencies and metadata for the project.

## Running the Example

### 1. First Run

To run the example for the first time, use the following command:

```bash
cargo run --release
```

This will compile the project in release mode, which is optimized for speed. During this initial run, the commitment keys for the BN256 and Grumpkin curves will be generated and cached. This process may take some time, so running in release mode ensures it completes as quickly as possible.

### 2. Subsequent Runs
For subsequent runs, you can use the following command without the --release flag:

```bash
cargo run
```

This will reuse the previously generated commitment keys, so the process will be faster, and there’s no need to recompile in release mode unless you're making significant changes or need the performance optimization again.

### 3. Expected Output
When the example runs successfully, you should see output indicating that the folding steps were executed and verified successfully:

```text
start setup primary commitment key: bn256
start setup secondary commitment key: grumpkin
ivc created
folding step 1 was successful
folding step 2 was successful
folding step 3 was successful
folding step 4 was successful
folding step 5 was successful
verification successful
success
```

## Understanding the Example
This example demonstrates the following key concepts of the Sirius framework:

- StepCircuit: A trait representing the circuit for each step in the IVC. In this example, the circuit performs an identity operation.
- Commitment Keys: Setup for the primary and secondary circuits, using BN256 and Grumpkin elliptic curves.
- Folding Steps: Execution of multiple folding steps, each represented by an invocation of the fold_step function.

For more detailed explanations, please refer to the main [Sirius documentation](https://docs.snarkify.io/sirius-folding/quickstart).

## Next Steps
After understanding this basic example, you can explore more complex examples and customize your circuits:

- Modify the StepCircuit implementation to perform non-trivial operations.
- Experiment with different folding step counts and configurations.
- Explore the Sirius main repository for advanced features like custom gates and high-degree optimizations.

# Getting Involved

We'd love for you to be a part of our community!

If you're as enthusiastic about `Sirius` as we are, we invite you to join our developer community at Telegram. It's a great place to stay updated, get involved, and contribute to the project. Whether you're looking to contribute code, provide feedback, or simply stay in the loop, our Telegram group is the place to be.

:point_right: [Join our developer community](https://t.me/+oQ04SUgs6KMyMzlh)

Thank you for your interest in our project! :sparkles:
