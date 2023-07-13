//! Implementation of metaprogramming tasks (i.e. code generation)

use log::{info, warn};
use std::fs;
use quote::{quote, format_ident};
use prettyplease;
use syn::{
    self,
    visit::Visit,
};

use crate::project::Project;
use crate::board::{pinout, Board};

/// This struct contains all the needed info for adding fields to the System module.
#[derive(Default, Debug)]
struct SystemBoardTokens {
    pub use_statements: Vec<proc_macro2::TokenStream>,
    pub bsp_crate_identifiers: Vec<syn::Ident>,
    pub board_field_identifiers: Vec<syn::Ident>,
    pub board_field_type_identifiers: Vec<syn::Ident>,
    pub field_type_token_streams: Vec<proc_macro2::TokenStream>,
    pub field_constructor_token_streams: Vec<proc_macro2::TokenStream>,
    pub available_types: Vec<String>,
    pub type_substitutions: Vec<(String, Option<String>)>,
}

/// Use the `syn::visit::Visit` trait to explore the syntax tree of the BSP
impl<'ast> Visit<'ast> for SystemBoardTokens {
    fn visit_item_struct(&mut self, item_struct: &'ast syn::ItemStruct) {
        if item_struct.ident.to_string() == "Board" {
            info!("found Board struct");
            let (field, crat, typ) = (
                self.board_field_identifiers.last().unwrap(),
                self.bsp_crate_identifiers.last().unwrap(),
                format_ident!("{}", "Board")
            );
            self.board_field_type_identifiers.push(typ.clone());
            self.field_type_token_streams.push(quote! {
                pub #field: #crat::#typ
            });
            self.field_constructor_token_streams.push(quote! {
                #field: #crat::#typ::new()
            });
            self.visit_generics(&item_struct.generics);
            // self.visit_where_clause(&item_struct.generics.where_clause.clone().unwrap());
        }
    }

    fn visit_item_type(&mut self, item_type: &'ast syn::ItemType) {
        if item_type.ident.to_string() == "I2CBus" {
            info!("found a type with named 'I2CBus'");
            self.available_types.push(item_type.ident.to_string().clone());
        }
    }

    fn visit_generics(&mut self, generics: &'ast syn::Generics) {
        info!("found {} generic parameters", generics.params.len());
        for (i, t) in generics.type_params().enumerate() {
            info!("generic {} Ident is {:?}", i, t.ident.clone());
            let replacement = match t.ident.to_string().as_str() {
                "I2C" => {
                    if self.available_types.contains(&String::from("I2CBus")) {
                        let _ = self.field_type_token_streams.pop();
                        let id = self.board_field_type_identifiers.pop().unwrap();
                        let bsp = self.bsp_crate_identifiers[0].clone();
                        let crat = self.bsp_crate_identifiers.last().unwrap();
                        let typ = format_ident!("{}", "I2CBus");
                        let field = self.board_field_identifiers.last().unwrap();
                        let field_type = quote! {
                            pub #field: #crat::#id<#bsp::#typ>
                        };
                        self.field_type_token_streams.push(field_type);
                        Some(String::from("I2CBus"))
                    } else {
                        None
                    }
                },
                _ => {
                    None
                }
            };
            self.type_substitutions.push((t.ident.to_string().clone(), replacement));
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum BspParseError {
    CrateNameError,
    OtherError,
}

impl Project {

    /// For each board in our system, generate the TokenStream for it's field
    /// in the System struct. Return these in a Vec.
    fn gather_board_fields(&mut self) -> Result<SystemBoardTokens, BspParseError> {

        let mut tokens = SystemBoardTokens::default();
        // TYPE RESOLUTION PASS -- in this pass we go through the BSPs, looking for generic types
        // and trying to substitute them with concrete ones.
        for board in self.system.boards.iter_mut() {

            // Get the BSP crate names to construct the `use` statements and field type identifiers.
            let bsp = board.bsp.clone().unwrap_or_else(|| String::from("todo_get_bsp_name")).replace("-", "_").replace("(", "").replace(")", "");
            let bsp_crate_ident = quote::format_ident!(
                "{}",
                bsp,
            );
            tokens.bsp_crate_identifiers.push(bsp_crate_ident.clone());
            tokens.use_statements.push(quote! {
                use #bsp_crate_ident;
            });

            let board_field_name = board.get_name().replace(" ", "_").to_ascii_lowercase();
            let board_field_ident = quote::format_ident!("{}", board_field_name);
            tokens.board_field_identifiers.push(board_field_ident);

            // Parse the BSP to look at what's in it, determine if we need to resolve 
            // any generic types, etc
            info!("sending board {:?} through the syn visitor!", board.get_name());
            if let Some(syntax) = board.parse_bsp() {
                // iterate throught the AST, looking for a struct called "Board"
                syntax.items.iter().for_each(|item| {
                    match item {
                        syn::Item::Struct(item_struct) => {
                            tokens.visit_item_struct(item_struct);
                        },
                        syn::Item::Type(item_type) => {
                            tokens.visit_item_type(item_type);
                        }
                        _ => (),
                    }
                });
            } else {
                warn!("couldn't parse BSP syntax for board {}!", board.get_name());
                return Err(BspParseError::OtherError);
            }
        }

        info!("after parsing BSPs, the datastructure looks like: \n{:#?}", tokens);

        // Now go through another round to generate the 
        // for (idx, board) in self.system.boards.iter_mut().enumerate() {
        //     // Get the BSP crate names to generate the `use` statements and struct field names.
        //     let bsp = board.bsp.clone().unwrap_or_else(|| String::from("todo_get_bsp_name")).replace("-", "_").replace("(", "").replace(")", "");
        //     let bsp_crate_ident = quote::format_ident!(
        //         "{}",
        //         bsp,
        //     );
            

        //     let use_statement = quote! {
        //         use #bsp_crate_ident;
        //     };

        //     let struct_field = quote! {
        //         pub #board_name: #bsp_crate_ident::Board
        //     };


        //     let struct_constructor = quote! {
        //         #board_name: #bsp_crate_ident::Board::new()
        //     };

        //     sbt.field_constructor_token_streams.push(struct_constructor);

        // }

        return Ok(tokens);

    }

    /// Generate a module based on the system. Lots to improve here. For now, this just saves
    /// the module to the project root (i.e. doesn't account for the existance of a Cargo project).
    pub fn generate_system_module(&mut self) -> Result<(), String> {

        let SystemBoardTokens {
            use_statements,
            field_type_token_streams,
            field_constructor_token_streams,
            ..
        } = match self.gather_board_fields() {
            Ok(sbt) => sbt,
            Err(e) => {
                warn!("error in gather_board_fields method: {:?}", e);
                return Err("error gathering board fields".to_string());
            }
        }
        
        ;

        /************* MODULE CODE HERE *************/
        let output_tokens = quote!
        {
            #(#use_statements)*
            // #(use #crate_identifiers);*

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
        fs::write(self.get_location() + "/src/sys_mod_output_testing.rs", code.as_str()).unwrap();
        
        Ok(())

    }

}