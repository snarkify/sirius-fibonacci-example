use std::{ops::Add, path::Path};

use sirius::{
    ff::Field,
    halo2_proofs::{
        plonk::{Advice, Column, Selector},
        poly::Rotation,
    },
    ivc::{
        step_circuit::{trivial, AssignedCell, ConstraintSystem, Layouter},
        SynthesisError,
    },
    prelude::{
        bn256::{new_default_pp, C1Affine, C1Scalar, C2Affine, C2Scalar},
        CommitmentKey, PrimeField, StepCircuit, IVC,
    },
};

/// Number of folding steps
const FOLD_STEP_COUNT: usize = 5;

// === PRIMARY ===

/// Arity : Input/output size per fold-step for primary step-circuit
const A1: usize = 2;

/// Key size for Primary Circuit
///
/// This is the minimum value, for your circuit you may get the output that the key size is
/// insufficient, then increase this constant
const PRIMARY_COMMITMENT_KEY_SIZE: usize = 20;

/// Table size for Primary Circuit
///
/// Requires at least 17, for service purposes, but if the primary requires more, increase the
/// constant
const PRIMARY_CIRCUIT_TABLE_SIZE: usize = 17;

// === SECONDARY ===

/// Arity : Input/output size per fold-step for secondary step-circuit
/// For tivial case it can be any number
const A2: usize = 1;

/// Input to be passed on the zero step to the secondary circuit
const SECONDARY_Z_0: [C2Scalar; A2] = [C2Scalar::ZERO];

/// Table size for Primary Circuit
///
/// Requires at least 17, for service purposes, but if the primary requires more, increase the
/// constant
const SECONDARY_CIRCUIT_TABLE_SIZE: usize = 17;

/// Key size for Secondary Circuit
///
/// This is the minimum value, for your circuit you may get the output that the key size is
/// insufficient, then increase this constant
const SECONDARY_COMMITMENT_KEY_SIZE: usize = 20;

/// Iterator for generating Fibonacci sequence values.
///
/// Given two initial values, it produces subsequent Fibonacci numbers by
/// summing the two preceding numbers.
///
/// # Example
/// ```
/// const EXPECTED: [u64; 20] = [
///     0, 1, 1, 2, 3, 5, 8, 13, 21, 34,
///     55, 89, 144, 233, 377, 610, 987,
///     1597, 2584, 4181,
/// ];
/// let actual = FibonacciIter(0, 1).take(20).collect::<Vec<_>>();
/// assert_eq!(&actual, &EXPECTED);
/// ```
struct FibonacciIter<F>(F, F);

impl<F: Add<Output = F> + Copy> Iterator for FibonacciIter<F> {
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.0;

        self.0 = self.1;
        self.1 = cur + self.1;

        Some(cur)
    }
}

/// Configuration for the Fibonacci circuit, which includes a selector and an advice column to hold
/// intermediate values of the Fibonacci sequence.
#[derive(Debug, Clone)]
struct FibonacciConfig {
    /// Selector used to activate the gate that enforces the Fibonacci relation.
    s: Selector,
    /// Advice column to store the current and previous Fibonacci numbers.
    e: Column<Advice>,
}

/// Circuit that generates Fibonacci numbers over multiple folding steps.
///
/// The circuit is configured to prove a specified number of Fibonacci sequence elements in each
/// step, defined by `ELEMENTS_NUM`.
struct FibonacciCircuit<const ELEMENTS_NUM: usize> {}

impl<F: PrimeField, const N: usize> StepCircuit<A1, F> for FibonacciCircuit<N> {
    /// This is a configuration object that stores things like columns.
    type Config = FibonacciConfig;

    /// Configure the step circuit. This method initializes necessary
    /// fixed columns and advice columns
    fn configure(cs: &mut ConstraintSystem<F>) -> Self::Config {
        let config = Self::Config {
            s: cs.selector(),
            e: cs.advice_column(),
        };

        cs.enable_equality(config.e);

        cs.create_gate("fibo-block", |meta| {
            let s = meta.query_selector(config.s);

            let e1 = meta.query_advice(config.e, Rotation(-2));
            let e2 = meta.query_advice(config.e, Rotation(-1));
            let e3 = meta.query_advice(config.e, Rotation(0));

            vec![s * (e1 + e2 - e3)]
        });

        config
    }

    /// Sythesize the circuit for a computation step and return variable
    /// that corresponds to the output of the step z_{i+1}
    /// this method will be called when we synthesize the IVC_Circuit
    ///
    /// Return `z_out` result
    fn synthesize_step(
        &self,
        config: Self::Config,
        layouter: &mut impl Layouter<F>,
        z_i: &[AssignedCell<F, F>; 2],
    ) -> Result<[AssignedCell<F, F>; 2], SynthesisError> {
        let z_out = layouter.assign_region(
            || "main",
            |mut region| {
                let [a, b] = z_i;

                FibonacciIter(a.value().copied(), b.value().copied())
                    .enumerate()
                    .map(|(offset, value)| {
                        let assigned = region.assign_advice(
                            || "element of sequence",
                            config.e,
                            offset,
                            || value,
                        )?;

                        // Enforce equality constraints on the first two elements.
                        //
                        // For all other - enable gate with check. Note that the gate starts work
                        // at index 2, because the gate references the -2 cell internally
                        match offset {
                            0 => {
                                region.constrain_equal(a.cell(), assigned.cell())?;
                            }
                            1 => {
                                region.constrain_equal(b.cell(), assigned.cell())?;
                            }
                            _ => {
                                config.s.enable(&mut region, offset)?;
                            }
                        }

                        Ok(assigned)
                    })
                    .take(N + A1)
                    .skip(N) // We only need the last two elements (A1 := 2)
                    .collect::<Result<Vec<_>, _>>()
            },
        )?;

        Ok(z_out.try_into().unwrap())
    }
}

fn main() {
    let sc1 = FibonacciCircuit::<10> {};
    let sc2 = trivial::Circuit::<A2, C2Scalar>::default();

    // This folder will store the commitment key so that we don't have to generate it every time.
    //
    // NOTE: since the key files are not serialized, but reflected directly from memory, the
    // functions to load them is `unsafe`
    let key_cache = Path::new(".cache");

    println!("start setup primary commitment key: bn256");

    // Safety: because the cache file is correct
    let primary_commitment_key = unsafe {
        CommitmentKey::<C1Affine>::load_or_setup_cache(
            key_cache,
            "bn256",
            PRIMARY_COMMITMENT_KEY_SIZE,
        )
        .unwrap()
    };

    println!("start setup secondary commitment key: grumpkin");

    // Safety: because the cache file is correct
    let secondary_commitment_key = unsafe {
        CommitmentKey::<C2Affine>::load_or_setup_cache(
            key_cache,
            "grumpkin",
            SECONDARY_COMMITMENT_KEY_SIZE,
        )
        .unwrap()
    };

    let pp = new_default_pp::<A1, _, A2, _>(
        SECONDARY_CIRCUIT_TABLE_SIZE as u32,
        &primary_commitment_key,
        &sc1,
        PRIMARY_CIRCUIT_TABLE_SIZE as u32,
        &secondary_commitment_key,
        &sc2,
    );

    // Input to be passed on the zero step to the primary circuit
    let primary_z_0: [C1Scalar; A1] = [C1Scalar::ZERO, C1Scalar::ONE];

    let mut ivc = IVC::new(&pp, &sc1, primary_z_0, &sc2, SECONDARY_Z_0, true)
        .expect("failed to create `IVC`");
    println!("ivc created");

    for step in 1..FOLD_STEP_COUNT {
        // you can modify circuit data here
        ivc.fold_step(&pp, &sc1, &sc2)
            .expect("failed to run fold step");

        println!("folding step {step} was successful");
    }

    ivc.verify(&pp).expect("failed to verify ivc");
    println!("verification successful");

    println!("success");
}
