extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, LitStr};

#[proc_macro_attribute]
pub fn topic_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 解析结构体定义
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let generics = &input.generics;

    // 处理属性参数
    let topic_lit = if attr.is_empty() {
        // 无参数时使用结构体名
        let name = struct_name.to_string();
        LitStr::new(&name, struct_name.span())
    } else {
        // 解析字符串参数
        match syn::parse::<LitStr>(attr) {
            Ok(lit) => lit,
            Err(e) => return e.to_compile_error().into(),
        }
    };

    // 生成实现代码
    let expanded = quote! {
        #input

        impl #generics pubsub::Topic for #struct_name #generics {
            const TOPIC: &'static str = #topic_lit;
        }
    };

    TokenStream::from(expanded)
}
