//! Implementation of metaprogramming tasks (i.e. code generation)

use log::warn;
use std::fs;
use quote::quote;
use syn;
use prettyplease;
use crate::project::Project;

/// This struct contains all the needed info for adding fields to the System module.
#[derive(Default)]
struct SystemBoardTokens {
    pub use_statements: Vec<proc_macro2::TokenStream>,
    pub field_type_token_streams: Vec<proc_macro2::TokenStream>,
    pub field_constructor_token_streams: Vec<proc_macro2::TokenStream>,
}

impl Project {

    /// For each board in our system, generate the TokenStream for it's field
    /// in the System struct. Return these in a Vec.
    fn gather_board_fields(&self) -> SystemBoardTokens {

        let SystemBoardTokens {
            mut use_statements,
            mut field_type_token_streams,
            mut field_constructor_token_streams,
        } = SystemBoardTokens::default();

        for board in self.system.boards.iter() {
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

            let struct_constructor = quote! {
                #board_name: #bsp_name::Board::new()
            };

            use_statements.push(use_statement);
            field_type_token_streams.push(struct_field);
            field_constructor_token_streams.push(struct_constructor);

        }
        return SystemBoardTokens {
            use_statements: use_statements,
            field_type_token_streams: field_type_token_streams,
            field_constructor_token_streams: field_constructor_token_streams,
        };
    }

    /// Generate a module based on the system. Lots to improve here. For now, this just saves
    /// the module to the project root (i.e. doesn't account for the existance of a Cargo project).
    pub fn generate_system_module(&self) -> std::io::Result<()> {

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