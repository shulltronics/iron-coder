//! Implementation of metaprogramming tasks (i.e. code generation)

use log::{info, warn};
use std::fs;
use quote::quote;
use prettyplease;
use syn::{
    self,
    visit::Visit,
};

use crate::project::Project;
use crate::board::pinout;

/// This struct contains all the needed info for adding fields to the System module.
#[derive(Default)]
struct SystemBoardTokens {
    pub use_statements: Vec<proc_macro2::TokenStream>,
    pub field_type_token_streams: Vec<proc_macro2::TokenStream>,
    pub field_constructor_token_streams: Vec<proc_macro2::TokenStream>,
}

/// Use the `syn::visit::Visit` trait to explore the syntax tree of the BSP
impl<'ast> Visit<'ast> for SystemBoardTokens {
    fn visit_item_struct(&mut self, item_struct: &'ast syn::ItemStruct) {
        if item_struct.ident.to_string() == "Board" {
            info!("found Board struct");
            self.visit_generics(&item_struct.generics);
        }
    }

    // fn visit_item(&mut self, item: &'ast syn::Item) {
    //     info!("visit_item called");
    // }

    fn visit_generics(&mut self, generics: &'ast syn::Generics) {
        // info!("iterating generics: {:#?}", generics);
        for g in generics.params.iter() {
            info!("found generic param {:?}", g);
        }
    }
}

impl Project {

    /// For each board in our system, generate the TokenStream for it's field
    /// in the System struct. Return these in a Vec.
    fn gather_board_fields(&mut self) -> SystemBoardTokens {

        let mut sbt = SystemBoardTokens::default();

        for board in self.system.boards.iter_mut() {
            // firstly, get the BSP crate names to generate the `use` statements and struct field names.
            let bsp = board.bsp.clone().unwrap_or_else(|| String::from("todo_get_bsp_name")).replace("-", "_").replace("(", "").replace(")", "");
            let bsp_name = quote::format_ident!(
                "{}",
                bsp,
            );
            let name = board.get_name().replace(" ", "_").to_ascii_lowercase();
            let board_name = quote::format_ident!("{}", name);

            let use_statement = quote! {
                use #bsp_name;
            };

            let struct_field = quote! {
                pub #board_name: #bsp_name::Board
            };
            // push them onto the vecs
            sbt.use_statements.push(use_statement);
            sbt.field_type_token_streams.push(struct_field);

            let struct_constructor = quote! {
                #board_name: #bsp_name::Board::new()
            };
            sbt.field_constructor_token_streams.push(struct_constructor);

            // Parse the BSP to look at what's in it, determine if we need to resolve 
            // any generic types, etc
            info!("sending board {:?} through the syn visitor!", board.get_name());
            if let Some(syntax) = board.parse_bsp() {
                // iterate throught the AST, looking for a struct called "Board"
                syntax.items.iter().for_each(|item| {
                    match item {
                        syn::Item::Struct(item_struct) => {
                            sbt.visit_item_struct(item_struct);
                        },
                        _ => (),
                    }
                });
            } else {
                warn!("couldn't parse BSP syntax!");
            }

        }

        return sbt;

    }

    /// Generate a module based on the system. Lots to improve here. For now, this just saves
    /// the module to the project root (i.e. doesn't account for the existance of a Cargo project).
    pub fn generate_system_module(&mut self) -> std::io::Result<()> {

        let SystemBoardTokens {
            use_statements,
            field_type_token_streams,
            field_constructor_token_streams,
        } = self.gather_board_fields();

        /************* MODULE CODE HERE *************/
        let output_tokens = quote!
        {
            #(#use_statements)*

            // todo - include needed imports

            pub struct System {
                #(#field_type_token_streams),*
            }

            impl System {
                pub fn new() -> Self {
                    Self {
                        #(#field_constructor_token_streams),*
                    }
                }
            }

            // #(#connection_impls)*

        };
        /************* End Module Code *************/

        // now output the module code to a file, passing through the prettyplease formatter
        let syn_code: syn::File = match syn::parse2(output_tokens) {
            Ok(syn_code) => syn_code,
            Err(e) => {
                warn!("couldn't parse output_tokens! {:?}", e);
                syn::parse_str("// error generating module").unwrap()
            }
        };
        let code = prettyplease::unparse(&syn_code);
        fs::write(self.get_location() + "/src/sys_mod_output_testing.rs", code.as_str())?;
        
        Ok(())

    }

}