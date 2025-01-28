use std::process;
use clap::Command;
use serve::CommandRunner;
use std::error::Error;
use crate::install::RealCommandRunner;
use crate::serve::RealCommand;
use std::path::Path;
use std::thread;
use eth_rpc::EthRpcInstaller;

pub mod serve;
pub mod template;
pub mod install;
pub mod chain_specs;
pub mod os_check;
pub mod eth_rpc;


fn main() {
    let matches = Command::new("polkadot-cli")
        .version("0.1.0")
        .author("Author Name <author@example.com>")
        .about("CLI tool for Polkadot")
        .subcommand(
            Command::new("install")
                .about("Installs the polkadot-sdk, generate chain spec and will get omni-node binary (Default)")
                .arg(
                    clap::Arg::new("template")
                        .help("The template to use for installation")
                        .long("template")
                        .global(true)
                        .action(clap::ArgAction::Set), // Use Set to capture the value
                )
                .arg(
                    clap::Arg::new("chain_spec")
                        .help("Specify the chain to install")
                        .long("chain-spec")
                        .global(true)
                        .action(clap::ArgAction::Set), // Use Set to capture the value
                )
        )
        .subcommand(
            Command::new("serve")
                .about("Serve omni-node using westend asset hub runtime (Default)")
                .arg(
                    clap::Arg::new("chain_spec")
                        .help("The fullpath to the chain spec file")
                        .long("chain-spec")
                        .required(false)
                        .value_name("CHAIN_SPEC")
                        .index(1),
                )
        )
    .get_matches();


    match matches.subcommand() {
        Some(("install", sub_matches)) => handle_install(sub_matches),
        Some(("serve", sub_matches)) => handle_serve(sub_matches),
        _ => {
            eprintln!("No valid subcommand provided. Use --help for more information.");
            process::exit(1);
        }
    }
}

fn handle_install(matches: &clap::ArgMatches) {
    let mut sub_commands: Vec<(String, String)> = Vec::new();

    if let Some(template) = matches.get_one::<String>("template") {
        handle_template_options(&template, matches);
        sub_commands.push(("--template".to_string(), template.clone()));
    }

    else if let Some(chain) = matches.get_one::<String>("chain_spec") {
        handle_chain_spec_options(&chain, matches);
        sub_commands.push(("--chain-spec".to_string(), chain.clone()));
    } else {
        println!("Installing default configuration.");
        install("default");
    }
}

pub fn install(_template: &str){
    let mut results: Vec<(Result<(), Box<dyn Error>>, &str)> = Vec::new();
    let binaries_dir = "./binaries";
    
    let wasm_source_path =  Path::new("./nodes/westend.wasm");
    let chain_spec_builder_path = Path::new("./binaries/chain-spec-builder");
    let destination = Path::new("./nodes/westend.wasm");

    let installer = EthRpcInstaller::new(binaries_dir, "eth-rpc");
    let eth_installer_result = installer.install();

    let real_runner = RealCommandRunner;
    results.push((install::install_polkadot(&real_runner), "$ Polkadot installation"));
    results.push((chain_specs::install_chain_spec_builder(), "$ Chain spec builder installation"));
    results.push((install::install_omni_node(), "$ Omni-node installation"));
    results.push((eth_installer_result, "$ Eth-rpc installation"));
    results.push((install::run_download_script(&real_runner, &destination ), "$ Wasm file download script"));
    results.push((chain_specs::gen_chain_spec(Some(&wasm_source_path), Some(&chain_spec_builder_path)), "$ Chain spec script"));

    println!(" ");
    println!("===========================================================================");
    println!(" ");
    for (result, message) in results {
        match result {
            Ok(_) => println!("{} success ✓", message),
            Err(_e) => println!("{} failed ✗", message),
        }
    }
    println!(" ");
    println!("===========================================================================");
    println!(" ");
}

fn handle_template_options(template_name: &str, matches: &clap::ArgMatches) {
    let args: Vec<&str> = matches.get_many::<String>("args")
        .map(|values| values.map(|s| s.as_str()).collect())
        .unwrap_or_else(|| Vec::new());

    println!("Called template installation");
    let _ = template::run_template(&args, &template_name);
}

fn handle_chain_spec_options(chain_spec: &str, matches: &clap::ArgMatches) {
    let _args: Vec<&str> = matches.get_many::<String>("args")
        .map(|values| values.map(|s| s.as_str()).collect())
        .unwrap_or_else(|| Vec::new());

    println!("Called chain_spec generation");

    match chain_spec {
        "westend" | "paseo" | "rococo" => {
            println!("No available functionality for chain spec generation yet.");
        }
        _ => {
            eprintln!("Invalid chain specification provided: {}", chain_spec);
            process::exit(1);
        }
    }
}

fn handle_serve(matches: &clap::ArgMatches) {
    let mut args: Vec<String> = matches.get_one::<String>("ARGS")
        .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
        .unwrap_or_else(|| vec![]);  

    if args.is_empty() {
        args = vec!["--chain".to_string(), "./chain-specs/chain_spec.json".to_string()];
    }
    args.push("--base-path".to_string());
    args.push("/tmp/node".to_string());
    args.push("--dev".to_string());


    println!("Starting omni-node and eth-rpc concurrently...");

    let handle_run = thread::spawn(move || {
        serve::run(&args.iter().map(|s| s.as_str()).collect::<Vec<&str>>()); // Convert to &str slice
    });

    let mut real_runner_eth = RealCommand::new("./binaries/eth-rpc");
    let handle_run_eth = thread::spawn(move || {
        let eth_args: Vec<&str> = vec![];
        if let Err(e) = serve::run_eth(&mut real_runner_eth, &eth_args) {
            eprintln!("Error running eth-rpc: {}", e);
        }
    });

    handle_run.join().expect("Failed to join run thread");
    handle_run_eth.join().expect("Failed to join run_eth thread");

    process::exit(0);
}