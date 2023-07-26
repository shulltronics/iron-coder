//! This module contains data and operations pertaining to the parsing of a
//! Board Support Package (BSP).

use log::{info, warn};
use std::vec::Vec;
use std::string::String;
use std::fs;
use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
};
use syn::{
    Ident,
    visit::Visit,
};

use crate::board::Board;

#[derive(Debug)]
#[non_exhaustive]
pub enum BspParseError {
    BspMissingError,
    OtherError,
}

pub type Result = core::result::Result<(), BspParseError>;

/// This struct contains all the needed info for adding fields to the System module.
#[derive(Default, Debug, Clone)]
pub struct BspParseInfo {
    /// The identifier representing the name of the BSP crate
    pub bsp_crate_identifier: Option<Ident>,
    /// The import statement to include this BSP in another module
    pub use_statement: TokenStream,
    /// A list of identifiers representing the field names for this board. For a main board, this will be
    /// "main_board", and for perihperal boards, TODO.
    pub board_field_identifiers: Vec<Ident>,
    /// A list of identifiers representing the types of the fields of the System struct
    pub board_field_type_identifiers: Vec<Ident>,
    /// A list of TokenStreams of the form <field>: <type> for use in the System struct
    pub field_type_token_streams: Vec<TokenStream>,
    /// A list of TokenStreams of the form <field>: <constructor> for use in the System struct impl
    pub field_constructor_token_streams: Vec<TokenStream>,
    /// A list of public type aliases offered by the main board BSP
    pub available_types: Vec<String>,
    /// A mapping of generic types to concrete types for use in `self.board_field_type_identifiers`
    pub type_substitutions: Vec<(String, Option<String>)>,
}


/// Use the `syn::visit::Visit` trait to explore the syntax tree of the BSP.
impl<'ast> Visit<'ast> for BspParseInfo {

    /// Actions when we visit a top-level struct in the BSP.
    fn visit_item_struct(&mut self, item_struct: &'ast syn::ItemStruct) {
        if item_struct.ident.to_string() == "Board" {
            info!("found Board struct");
            let (field, crat, typ) = (
                self.board_field_identifiers.last().unwrap(),
                self.bsp_crate_identifier.clone().unwrap(),
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
    /// Actions when we visit a top-level type in the BSP.
    fn visit_item_type(&mut self, item_type: &'ast syn::ItemType) {
        if item_type.ident.to_string() == "I2CBus" {
            info!("found a type with named 'I2CBus'");
            self.available_types.push(item_type.ident.to_string().clone());
        }
    }
    /// Actions when we visit some generic type parameters in the BSP.
    fn visit_generics(&mut self, generics: &'ast syn::Generics) {
        info!("found {} generic parameters", generics.params.len());
        for (i, t) in generics.type_params().enumerate() {
            info!("generic {} Ident is {:?}", i, t.ident.clone());
            let replacement = match t.ident.to_string().as_str() {
                "I2C" => {
                    if self.available_types.contains(&String::from("I2CBus")) {
                        let _ = self.field_type_token_streams.pop();
                        let id = self.board_field_type_identifiers.pop().unwrap();
                        let bsp = self.bsp_crate_identifier.clone();
                        let crat = self.bsp_crate_identifier.clone().unwrap();
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

/// Board impls regarding parsing of BSP syntax
impl Board {

    // Attempt to parse the BSP lib file.
    fn parse_bsp(&self) -> Option<syn::File> {
        let mut syntax = None;
        if let Some(bsp_dir) = self.bsp_path.clone() {
            let src = bsp_dir.join("src/lib.rs");
            let src = fs::read_to_string(src.as_path()).unwrap();
            syntax = match syn::parse_file(src.as_str()) {
                Ok(syntax) => {
                    Some(syntax)
                },
                Err(e) => {
                    warn!("Couldn't parse BSP for board {:?} with syn: {:?}", self.get_name(), e);
                    None
                },
            };
        }
        // self.bsp_syntax = syntax.clone();
        return syntax;
    }

    /// Try to add the BSP info to self, returning Ok on success or otherwise indicating the type of error.
    pub fn load_bsp_info(&mut self) -> Result {

        let mut bsp_parse_info = BspParseInfo::default();

        // Get the BSP crate names to construct the `use` statements and field type identifiers.
        let bsp = match self.bsp.clone() {
            Some(bsp) => bsp.replace("-", "_").replace("(", "").replace(")", "").replace(".", ""),
            None => return Err(BspParseError::BspMissingError),
        };
        let bsp_crate_ident = quote::format_ident!(
            "{}",
            bsp,
        );
        bsp_parse_info.bsp_crate_identifier = Some(bsp_crate_ident.clone());
        bsp_parse_info.use_statement = quote! {
            use #bsp_crate_ident;
        };

        let board_field_name = self.get_name()
            .replace(" ", "_")
            .replace("-", "_")
            .replace("(", "")
            .replace(")", "")
            .replace(".", "")
            .to_ascii_lowercase();
        let board_field_ident = quote::format_ident!("{}", board_field_name);
        bsp_parse_info.board_field_identifiers.push(board_field_ident);

        // Parse the BSP to look at what's in it, determine if we need to resolve 
        // any generic types, etc
        info!("sending board {:?} through the syn visitor!", self.get_name());
        if let Some(syntax) = self.parse_bsp() {
            // iterate throught the AST, looking for a struct called "Board"
            syntax.items.iter().for_each(|item| {
                match item {
                    syn::Item::Struct(item_struct) => {
                        bsp_parse_info.visit_item_struct(item_struct);
                    },
                    syn::Item::Type(item_type) => {
                        bsp_parse_info.visit_item_type(item_type);
                    }
                    _ => (),
                }
            });
        } else {
            warn!("couldn't parse BSP syntax for board {}!", self.get_name());
            return Err(BspParseError::OtherError);
        }

        // debug!("after parsing BSPs, the datastructure looks like: \n{:#?}", bsp_parse_info);
        self.bsp_parse_info = Some(bsp_parse_info);
        Ok(())
    }

}