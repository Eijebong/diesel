use syn;
use quote;

use database_url::extract_database_url;
use diesel_infer_schema::*;

use util::{get_options_from_input, get_option, get_optional_option};

pub fn derive_infer_schema(input: syn::MacroInput) -> quote::Tokens {
    fn bug() -> ! {
        panic!("This is a bug. Please open a Github issue \
               with your invocation of `infer_schema`!");
    }

    let options = get_options_from_input("infer_schema_options", &input.attrs, bug)
        .unwrap_or_else(|| bug());
    let database_url = extract_database_url(get_option(&options, "database_url", bug)).unwrap();
    let schema_name = get_optional_option(&options, "schema_name");
    let schema_name = schema_name.as_ref().map(|s| &**s);

    let table_names = load_table_names(&database_url, schema_name)
        .expect(&error_message("table names", &database_url, schema_name));
    let foreign_keys = load_foreign_key_constraints(&database_url, schema_name)
        .expect(&error_message("foreign keys", &database_url, schema_name));
    let foreign_keys = remove_unsafe_foreign_keys_for_codegen(
        &database_url,
        &foreign_keys,
        &table_names,
    );

    let tables = table_names.iter()
        .map(|table| {
            let mod_ident = syn::Ident::new(format!("infer_{}", table.name));
            let table_name = table.to_string();
            quote! {
                mod #mod_ident {
                    infer_table_from_schema!(#database_url, #table_name);
                }
                pub use self::#mod_ident::*;
            }
        });
    let joinables = foreign_keys.into_iter()
        .map(|fk| {
            let child_table = syn::Ident::new(fk.child_table.name);
            let parent_table = syn::Ident::new(fk.parent_table.name);
            let foreign_key = syn::Ident::new(fk.foreign_key);
            quote!(joinable!(#child_table -> #parent_table (#foreign_key));)
        });

    let tokens = quote!(#(#tables)* #(#joinables)*);
    if let Some(schema_name) = schema_name {
        let schema_ident = syn::Ident::new(schema_name);
        quote!(pub mod #schema_ident { #tokens })
    } else {
        tokens
    }
}

pub fn derive_infer_table_from_schema(input: syn::MacroInput) -> quote::Tokens {
    fn bug() -> ! {
        panic!("This is a bug. Please open a Github issue \
               with your invocation of `infer_table_from_schema`!");
    }

    let options = get_options_from_input("infer_table_from_schema_options", &input.attrs, bug)
        .unwrap_or_else(|| bug());
    let database_url = extract_database_url(get_option(&options, "database_url", bug)).unwrap();
    let table_name = get_option(&options, "table_name", bug);

    expand_infer_table_from_schema(&database_url, &table_name.parse().unwrap())
        .expect(&format!("Could not infer table {}", table_name))
}

fn error_message(attempted_to_load: &str, database_url: &str, schema_name: Option<&str>) -> String {
    let mut message = format!("Could not load {} from database `{}`", attempted_to_load, database_url);
    if let Some(name) = schema_name {
        message += &format!(" with schema `{}`", name);
    }
    message
}
