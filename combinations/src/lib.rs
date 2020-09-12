use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Result};
use syn::parse::{Parse, ParseStream};

struct Combinations {
    name: syn::Ident,
    n: syn::LitInt,
}

impl Parse for Combinations {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let n = input.parse()?;
        Ok(Combinations { name, n })
    }
}

#[proc_macro]
pub fn minimal(input: TokenStream) -> TokenStream {
    let Combinations { name, n } = parse_macro_input!(input as Combinations);
    (quote! {
        fn #name() -> i32 {
            #n
        }
    }).into()
}

#[proc_macro]
pub fn punctuated(input: TokenStream) -> TokenStream {
    let Combinations { name,  n } = parse_macro_input!(input as _);
    let n_value = n.base10_parse::<i32>().unwrap();
    let indices: syn::punctuated::Punctuated<_, syn::Token![,]> = (0..n_value)
        .map(|i| quote! { #i })
        .collect();
    (quote! {
        fn #name() -> Vec<i32> {
            vec![#indices]
        }
    }).into()
}

#[proc_macro]
pub fn combinations(input: TokenStream) -> TokenStream {
    let Combinations { name, n } = parse_macro_input!(input as _);
    let n_value: usize = n.base10_parse().unwrap();
    let pool_indices: syn::punctuated::Punctuated<_, syn::Token![,]> = (0..n_value)
        .map(|i| quote! { &pool[indices[#i]] })
        .collect();
    let comb = quote! {
        fn #name<T>(pool: &[T]) -> Vec<[&T; #n]> {
            let r = #n;
            let mut res = vec![];
            let n = pool.len();
            if r > n {
                return res;
            }
            let mut indices: Vec<_> = (0..r).collect();
            res.push([#pool_indices]);
            loop {
                match (0..r).rev().find(|&i| indices[i] != (i + n - r)) {
                    Some(i) => {
                        indices[i] += 1;
                        for j in (i + 1)..r {
                            indices[j] = indices[j - 1] + 1;
                        }
                        res.push([#pool_indices]);
                    },
                    None => break,
                }
            }
            res
        }
    };
    comb.into()
}

#[proc_macro]
pub fn iter_combinations(input: TokenStream) -> TokenStream {
    let Combinations { name, n } = parse_macro_input!(input as Combinations);
    let n_value: usize = n.base10_parse().unwrap();
    let pool_indices: syn::punctuated::Punctuated<_, syn::Token![,]> = (0..n_value as usize)
        .map(|i| quote! { &self.pool[self.indices[#i]] })
        .collect();

    let initial_indices: syn::punctuated::Punctuated<_, syn::Token![,]> =
        (0..n_value as usize).collect();

    let iter_name = proc_macro2::Ident::new(
        &format!("CombIndicesIter{}", n_value),
        proc_macro2::Span::call_site(),
    );

    let comb = quote! {
        pub struct #iter_name<'a, T> {
            pool: &'a [T],
            indices: [usize; #n],
            started: bool,
        }

        impl<'a, T> Iterator for #iter_name<'a, T> {
            type Item = [&'a T; #n];

            fn next(&mut self) -> Option<Self::Item> {
                if !self.started {
                    self.started = true;
                    Some([#pool_indices])
                } else {
                    let n = self.pool.len();
                    (0..#n).rev().find(|&i| self.indices[i] != i + n - #n)
                        .map(|i| {
                                self.indices[i] += 1;
                                for j in (i + 1)..#n {
                                    self.indices[j] = self.indices[j - 1] + 1;
                                }
                                [#pool_indices]
                        })
                }
            }
        }

        fn #name<T>(pool: &[T]) -> impl Iterator<Item = [&T; #n]> {
            #iter_name {
                pool,
                indices: [#initial_indices],
                started: false,
            }
        }
    };
    comb.into()
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
