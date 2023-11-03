use serde::{Serialize, Deserialize};
use std::collections::{HashMap};
use quote::quote;
use proc_macro2::TokenStream;
use syn::Ident;

use std::process::Command;
use std::fs::File;
use std::io::Write;
use std::fs;
use std::path::PathBuf;

use cargo::ops::{CompileOptions, compile};
use cargo::util::{CargoResult};
use cargo::core::Workspace;
use cargo::core::compiler::{CompileMode};
use cargo::util::config::Config;


#[derive(Serialize, Deserialize)]
struct ConnectionModel {
    input_node: String, 
    output_node: String,
    input: String,
    output: String, 
}

#[derive(Serialize, Deserialize)]
struct InputModel {
    input_type: String 
}

#[derive(Serialize, Deserialize)]
struct OutputModel {
    output_type: String 
}

#[derive(Serialize, Deserialize)]
struct NodeModel {
    node_type: String,
    inputs: HashMap<String, InputModel>,
    outputs: HashMap<String, OutputModel>
}

#[derive(Serialize, Deserialize)]
struct FlowModel {
    nodes: HashMap<String, NodeModel>,
    connections: Vec<ConnectionModel>
}

trait CodeEmitter {
    fn emit_flow_code(&self, flow: &FlowModel) -> String; 
}

struct StandardCodeEmitter {

}

impl StandardCodeEmitter {
    
    fn generate_function(&self, body: &TokenStream) -> TokenStream {        
        quote! {
            #[wasm_bindgen]
            fn run() {
                #body
            }
        }
    }

    fn generate_function_body(&self, flow: &FlowModel) -> TokenStream {
        let mut body = TokenStream::new();

        self.generate_std_locals(&mut body);

        self.generate_nodes(flow, &mut body);

        self.generate_node_connections(flow, &mut body);

        self.generate_flow(flow, &mut body);

        self.generate_exec_call(&mut body);

        body
    }

    fn generate_nodes(&self, flow: &FlowModel, tokens: &mut TokenStream) {
        for (node_name, node) in &flow.nodes {
            let generated_code = self.generate_node_local(node_name, node);
            tokens.extend(generated_code);
        }
    }

    fn generate_node_connections(&self, flow: &FlowModel, tokens: &mut TokenStream) {
        for connection in &flow.connections {
            let generated_code = self.generate_node_connection(connection);
            tokens.extend(generated_code);
        }
    }

    fn generate_node_local(&self, node_name: &str, node: &NodeModel) -> TokenStream {
        let node_ident = Ident::new(&node_name, proc_macro2::Span::call_site());
        let node_type_ident = Ident::new(&node.node_type, proc_macro2::Span::call_site());

        quote! {
            let #node_ident = #node_type_ident::new(Some(&change_observer));
        }
    }

    fn generate_node_connection(&self, connection: &ConnectionModel) -> TokenStream {
        let node_out_ident = Ident::new(&connection.input_node, proc_macro2::Span::call_site());
        let node_inp_ident = Ident::new(&connection.output_node, proc_macro2::Span::call_site());
        let output_ident = Ident::new(&connection.output, proc_macro2::Span::call_site());
        let input_ident = Ident::new(&connection.input, proc_macro2::Span::call_site());

        quote! {
            connect(#node_out_ident.#output_ident.clone(), #node_inp_ident.#input_ident.clone());
        }
    }

    fn generate_std_locals(&self, tokens : &mut TokenStream)  {
        tokens.extend(
        quote! {
            let change_observer = ChangeObserver::new();
            let node_updater = SingleThreadedNodeUpdater::new(None);
            let mut executor = StandardExecutor::new(&change_observer);
            let scheduler = RoundRobinScheduler::new();
        });
    }

    fn generate_flow(&self, flow: &FlowModel, tokens: &mut TokenStream)  {
        tokens.extend(
        quote! {
            let flow = Flow::new_empty("wasm", Version::new(0, 0, 1));
        });

        let mut id:u128 = 0;
        for (node_name, node) in &flow.nodes {
            let node_ident = Ident::new(&node_name, proc_macro2::Span::call_site());
            let node_type = node.node_type.clone();
            tokens.extend(quote! {
                flow.add_node_with_id_and_desc(
                    #node_ident, 
                    #id, 
                    NodeDescription {name: #node_name, description: #node_name /*TODO*/, kind: #node_type});
            });
            id+=1;
        }
    }

    fn generate_exec_call(&self, tokens : &mut TokenStream)  {
        tokens.extend(
        quote! {
            let _ = executor.run(flow, scheduler, node_updater);
        });
    }

}

impl CodeEmitter for StandardCodeEmitter {
    fn emit_flow_code(&self, flow: &FlowModel) -> String {
         
        format!("{}", self.generate_function(
            &self.generate_function_body(flow)))
    }

}

trait WasmPackager {

    fn compile_package(&self, flow: &FlowModel);
}

struct StandardWasmPackager<T> {
    code_emitter : T,
    package_path : PathBuf 
}

impl<T> StandardWasmPackager<T> where T : CodeEmitter {
    fn new(code_emitter : T) -> Self {
        Self {
            code_emitter : code_emitter,
            package_path: PathBuf::from("C:/Users/friedrich/Projekte/flow-rs/tmp")
        }
    } 

     
    fn compile(&self) {

        // Replace "path/to/your/source.rs" with the actual path to your Rust source file
        let source_file = "source.rs";

        // Create a Command to compile the Rust source file
        let output = Command::new("rustc")
            .arg(source_file)
            .output()
            .expect("Failed to execute command");

        // Check if the command was successful
        if output.status.success() {
            println!("Compilation successful!");
        } else {
            // Print the captured standard error
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Compilation failed. stderr: {}", stderr);
        }

        // Get the exit status code
        let exit_code = output.status.code().unwrap_or(-1);
        println!("Exit code: {}", exit_code);
    }

    fn compile_2(&self) {
        let current_dir = self.package_path.clone();

        let config = Config::default().expect("");
        println!("Current: {}", current_dir.to_str().expect(""));

        let workspace = Workspace::new(&current_dir, &config).expect("HERE");
    
        // Specify the target to compile for (can be empty to use default target)
        //let target = None;
    
        // Specify other compilation options
        let compile_options = CompileOptions::new(&config, CompileMode::Build).expect("");
    
        let res = compile(&workspace, &compile_options);
        if let Err(e) = res {
            println!("{:?}", e);
        }
        
    }
}

impl<T> WasmPackager for StandardWasmPackager<T> where T : CodeEmitter {

    fn compile_package(&self, flow: &FlowModel) {
        
        let flow_code = self.code_emitter.emit_flow_code(flow);
        let mut path = self.package_path.clone();
        path.push("src");
        fs::create_dir(path.clone());
        path.push("lib.rs");

        {
            let mut file = File::create(path).expect("error");
            file.write_all(flow_code.as_bytes());
        }

        self.compile_2();

        //fs::remove_file("source.rs").expect("file should exist.");

    }
}

#[test]
fn test(){

    let flow_json = r#"
    {
        "nodes": {
            "node1": {
                "node_type": "NodeType1",
                "inputs": {
                    "input1": {
                        "input_type": "InputType1"
                    }
                },
                "outputs": {
                    "output1": {
                        "output_type": "OutputType1"
                    }
                }
            },
            "node2": {
                "node_type": "NodeType2",
                "inputs": {
                    "input2": {
                        "input_type": "InputType2"
                    }
                },
                "outputs": {
                    "output2": {
                        "output_type": "OutputType2"
                    }
                }
            }
        },
        "connections": [
            {
                "input_node": "node1",
                "output_node": "node2",
                "input": "input1",
                "output": "output2"
            }
        ]
    }
    "#;

    let flow_model: FlowModel = serde_json::from_str(&flow_json).expect("wrong format.");

    let rce = StandardCodeEmitter{};
    //println!("{}", rce.emit_flow_code(&flow_model));

    let pack = StandardWasmPackager::new(rce);
    pack.compile_package(&flow_model);

}
