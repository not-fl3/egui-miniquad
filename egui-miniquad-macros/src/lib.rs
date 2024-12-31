use std::collections::HashMap;

use quote::ToTokens;
use syn::{parse_macro_input, ItemImpl};

#[proc_macro_attribute]
pub fn egui_miniquad(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut ast: ItemImpl = parse_macro_input!(item as ItemImpl);

    let mut functions = HashMap::from([
    ("mouse_motion_event", quote::quote! {fn mouse_motion_event(&mut self, x: f32, y: f32) {
        self.egui_mq.mouse_motion_event(x, y);
    }}),

    ("mouse_wheel_event", quote::quote! {fn mouse_wheel_event(&mut self, dx: f32, dy: f32) {
        self.egui_mq.mouse_wheel_event(dx, dy);
    }}),

    ("mouse_button_down_event", quote::quote! {fn mouse_button_down_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_down_event(mb, x, y);
    }}),

    ("mouse_button_up_event", quote::quote! {fn mouse_button_up_event(&mut self, mb: mq::MouseButton, x: f32, y: f32) {
        self.egui_mq.mouse_button_up_event(mb, x, y);
    }}),

    ("char_event", quote::quote! {fn char_event(&mut self, character: char, _keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.char_event(character);
    }}),

    ("key_down_event", quote::quote! {fn key_down_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods, _repeat: bool) {
        self.egui_mq.key_down_event(keycode, keymods);
    }}),

    ("key_up_event", quote::quote! {fn key_up_event(&mut self, keycode: mq::KeyCode, keymods: mq::KeyMods) {
        self.egui_mq.key_up_event(keycode, keymods);
    }}),
    ]);

    for item in &ast.items {
        if let syn::ImplItem::Fn(m) = item {
            functions.remove(m.sig.ident.to_string().as_str());
        }
    }

    for (_, v) in functions {
        ast.items.push(syn::ImplItem::Fn(syn::parse2(v).unwrap()));
    }

    ast.to_token_stream().into()
}
