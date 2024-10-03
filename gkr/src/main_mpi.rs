use circuit::Circuit;
use clap::Parser;
use config::{
    BN254ConfigSha2, Config, FieldType, GF2ExtConfigSha2, GKRConfig, GKRScheme, M31ExtConfigSha2,
    MPIConfig,
};
use gkr::{
    utils::{
        KECCAK_BN254_CIRCUIT, KECCAK_BN254_WITNESS, KECCAK_GF2_CIRCUIT, KECCAK_GF2_WITNESS,
        KECCAK_M31_CIRCUIT, KECCAK_M31_WITNESS, POSEIDON_BN254_CIRCUIT, POSEIDON_M31_CIRCUIT,
    },
    Prover,
};

/// ...
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Field Identifier: fr, m31, m31ext3
    #[arg(short, long,default_value_t = String::from("m31ext3"))]
    field: String,

    // scheme: keccak, poseidon
    #[arg(short, long, default_value_t = String::from("keccak"))]
    scheme: String,

    /// number of repeat
    #[arg(short, long, default_value_t = 1)]
    repeats: usize,
}

fn main() {
    let args = Args::parse();
    print_info(&args);

    let mpi_config = MPIConfig::new();

    match args.field.as_str() {
        "m31ext3" => match args.scheme.as_str() {
            "keccak" => run_benchmark::<M31ExtConfigSha2>(
                &args,
                Config::<M31ExtConfigSha2>::new(GKRScheme::Vanilla, mpi_config.clone()),
            ),
            "poseidon" => run_benchmark::<M31ExtConfigSha2>(
                &args,
                Config::<M31ExtConfigSha2>::new(GKRScheme::GkrSquare, mpi_config.clone()),
            ),
            _ => unreachable!(),
        },
        "fr" => match args.scheme.as_str() {
            "keccak" => run_benchmark::<BN254ConfigSha2>(
                &args,
                Config::<BN254ConfigSha2>::new(GKRScheme::Vanilla, mpi_config.clone()),
            ),
            "poseidon" => run_benchmark::<BN254ConfigSha2>(
                &args,
                Config::<BN254ConfigSha2>::new(GKRScheme::GkrSquare, mpi_config.clone()),
            ),
            _ => unreachable!(),
        },
        "gf2ext128" => match args.scheme.as_str() {
            "keccak" => run_benchmark::<GF2ExtConfigSha2>(
                &args,
                Config::<GF2ExtConfigSha2>::new(GKRScheme::Vanilla, mpi_config.clone()),
            ),
            "poseidon" => run_benchmark::<GF2ExtConfigSha2>(
                &args,
                Config::<GF2ExtConfigSha2>::new(GKRScheme::GkrSquare, mpi_config.clone()),
            ),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };

    MPIConfig::finalize();
}

fn run_benchmark<C: GKRConfig>(args: &Args, config: Config<C>) {
    let pack_size = C::get_field_pack_size();

    // load circuit
    let mut circuit = match args.scheme.as_str() {
        "keccak" => match C::FIELD_TYPE {
            FieldType::GF2 => Circuit::<C>::load_circuit(KECCAK_GF2_CIRCUIT),
            FieldType::M31 => Circuit::<C>::load_circuit(KECCAK_M31_CIRCUIT),
            FieldType::BabyBear => Circuit::<C>::load_circuit(KECCAK_M31_CIRCUIT),
            FieldType::BN254 => Circuit::<C>::load_circuit(KECCAK_BN254_CIRCUIT),
        },
        "poseidon" => match C::FIELD_TYPE {
            FieldType::GF2 => unreachable!(),
            FieldType::M31 => Circuit::<C>::load_circuit(POSEIDON_M31_CIRCUIT),
            FieldType::BabyBear => Circuit::<C>::load_circuit(POSEIDON_M31_CIRCUIT),
            FieldType::BN254 => Circuit::<C>::load_circuit(POSEIDON_BN254_CIRCUIT),
        },
        _ => unreachable!(),
    };

    let witness_path = match C::FIELD_TYPE {
        FieldType::GF2 => KECCAK_GF2_WITNESS,
        FieldType::M31 => KECCAK_M31_WITNESS,
        FieldType::BabyBear => KECCAK_M31_WITNESS,
        FieldType::BN254 => KECCAK_BN254_WITNESS,
    };
    circuit.load_witness_file(witness_path);

    let circuit_copy_size: usize = match (C::FIELD_TYPE, args.scheme.as_str()) {
        (FieldType::GF2, "keccak") => 1,
        (FieldType::M31, "keccak") => 2,
        (FieldType::BN254, "keccak") => 2,
        (FieldType::M31, "poseidon") => 120,
        (FieldType::BN254, "poseidon") => 120,
        _ => unreachable!(),
    };

    const N_PROOF: usize = 1000;

    println!("We are now calculating average throughput, please wait for 1 minutes");
    for i in 0..args.repeats {
        let start_time = std::time::Instant::now();
        for _j in 0..N_PROOF {
            let mut prover = Prover::new(&config);
            prover.prepare_mem(&circuit);
            prover.prove(&mut circuit);
        }
        let stop_time = std::time::Instant::now();
        let duration = stop_time.duration_since(start_time);
        let throughput = (N_PROOF * circuit_copy_size * pack_size * config.mpi_config.world_size())
            as f64
            / duration.as_secs_f64();
        println!("{}-bench: throughput: {} hashes/s", i, throughput.round());
    }
}

fn print_info(args: &Args) {
    let prover = match args.scheme.as_str() {
        "keccak" => "GKR",
        "poseidon" => "GKR^2",
        _ => unreachable!(),
    };

    println!("===============================");
    println!(
        "benchmarking {} with {} over {}",
        args.scheme, prover, args.field
    );
    println!("field:          {}", args.field);
    println!("#bench repeats: {}", args.repeats);
    println!("hash scheme:    {}", args.scheme);
    println!("===============================")
}
