use proc_macro::{TokenStream, TokenTree};

#[proc_macro]
pub fn flip_bit(stream: TokenStream) -> TokenStream {
    let mut iter = stream.into_iter();
    if let TokenTree::Ident(bitfield) = iter.next().expect("Missing bitfield") {
        let mut bitfield = bitfield.to_string();
        if let Some(TokenTree::Punct(punct)) = iter.next() {
            if punct.as_char() == '.' {
                if let Some(TokenTree::Literal(lit)) = iter.next() {
                    iter.next();
                    bitfield.push('.');
                    bitfield.push_str(&lit.to_string());
                }
            }
        }
        if let TokenTree::Literal(bit) = iter.next().expect("Missing bit") {
            let bit = bit.to_string().parse::<u8>().expect("Failed to parse bit");
            format!("state.temporary_variant_progress.{bitfield} ^= 1 << {bit}").parse().unwrap()
        } else {
            panic!("Invalid bit")
        }
    } else {
        panic!("Invalid bitfield")
    }
}

#[proc_macro]
pub fn flip_bool(stream: TokenStream) -> TokenStream {
    let mut iter = stream.into_iter();
    if let TokenTree::Ident(bool) = iter.next().expect("Missing bool") {
        let mut bool = bool.to_string();
        if let Some(TokenTree::Punct(punct)) = iter.next() {
            if punct.as_char() == '.' {
                if let Some(TokenTree::Literal(lit)) = iter.next() {
                    iter.next();
                    bool.push('.');
                    bool.push_str(&lit.to_string());
                }
            }
        }
        format!("state.temporary_variant_progress.{bool} = !state.temporary_variant_progress.{bool}").parse().unwrap()
    } else {
        panic!("Invalid bool")
    }
}

#[proc_macro]
pub fn incr(stream: TokenStream) -> TokenStream {
    let mut iter = stream.into_iter();
    if let TokenTree::Ident(bitfield) = iter.next().expect("Missing bitfield") {
        let mut bitfield = bitfield.to_string();
        if let Some(TokenTree::Punct(punct)) = iter.next() {
            if punct.as_char() == '.' {
                if let Some(TokenTree::Literal(lit)) = iter.next() {
                    iter.next();
                    bitfield.push('.');
                    bitfield.push_str(&lit.to_string());
                }
            }
        }
        if let TokenTree::Literal(offset) = iter.next().expect("Missing offset") {
            iter.next();
            let offset = offset.to_string().parse::<u8>().expect("Failed to parse bit");
            if let TokenTree::Literal(size) = iter.next().expect("Missing size") {
                iter.next();
                let size = size.to_string().parse::<u8>().expect("Failed to parse bit");
                if let TokenTree::Literal(max) = iter.next().expect("Missing max") {
                    let max = max.to_string().parse::<u8>().expect("Failed to parse max");
                    format!("{{let mask = (1 << {size}) - 1; let current = state.temporary_variant_progress.{bitfield} >> {offset} & mask; if current < {max} {{state.temporary_variant_progress.{bitfield} = state.temporary_variant_progress.{bitfield} & !(mask << {offset}) | (current + 1) << {offset};}}}}")
                        .parse()
                        .unwrap()
                } else {
                    panic!("Invalid max")
                }
            } else {
                panic!("Invalid size")
            }
        } else {
            panic!("Invalid offset")
        }
    } else {
        panic!("Invalid bitfield")
    }
}

#[proc_macro]
pub fn reset(stream: TokenStream) -> TokenStream {
    let mut iter = stream.into_iter();
    if let TokenTree::Ident(bitfield) = iter.next().expect("Missing bitfield") {
        let mut bitfield = bitfield.to_string();
        if let Some(TokenTree::Punct(punct)) = iter.next() {
            if punct.as_char() == '.' {
                if let Some(TokenTree::Literal(lit)) = iter.next() {
                    iter.next();
                    bitfield.push('.');
                    bitfield.push_str(&lit.to_string());
                }
            }
        }
        if let TokenTree::Literal(offset) = iter.next().expect("Missing offset") {
            iter.next();
            let offset = offset.to_string().parse::<u8>().expect("Failed to parse bit");
            if let TokenTree::Literal(size) = iter.next().expect("Missing size") {
                let size = size.to_string().parse::<u8>().expect("Failed to parse bit");
                format!("state.temporary_variant_progress.{bitfield} = state.temporary_variant_progress.{bitfield} & !(((1 << {size}) - 1) << {offset})")
                    .parse()
                    .unwrap()
            } else {
                panic!("Invalid size")
            }
        } else {
            panic!("Invalid offset")
        }
    } else {
        panic!("Invalid bitfield")
    }
}

#[proc_macro]
pub fn greatest(stream: TokenStream) -> TokenStream {
    let mut iter = stream.into_iter();
    if let TokenTree::Ident(ident) = iter.next().expect("Missing ident") {
        let ident = ident.to_string();
        format!("if other.{ident} > self.{ident} {{self.{ident} = other.{ident};}}").parse().unwrap()
    } else {
        panic!("Invalid ident")
    }
}
