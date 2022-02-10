use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

#[proc_macro]
#[cfg(feature = "region")]
pub fn country_code_str_to_region(stream: TokenStream) -> TokenStream {
    let stream: proc_macro2::TokenStream = stream.into();

    //println!("{:?}", locale_codes::country::all_codes().iter().filter_map(|cc| locale_codes::country::lookup(cc).map(|ci| (cc, ci))).map(|(cc, ci)| (cc, &ci.short_code)).collect::<Vec<_>>());

    let mut regions: Vec<RegionFromCountryCodeStr> = locale_codes::country::all_codes()
        .iter()
        .filter_map(|cc| locale_codes::country::lookup(cc).map(|ci| (&ci.short_code, ci)))
        .filter_map(|(c, ci)| ci.region_code.map(|rc| (c, rc)))
        .filter_map(|(c, rc)| locale_codes::region::lookup(rc).map(|ri| (c, ri)))
        .map(|(c, ri)| RegionFromCountryCodeStr::new(c, &ri.name))
        .collect();

    regions.sort_by_key(|r| r.0.clone());

    let result = quote! {
        match #stream {
             #(#regions),*,
             _ => None
        }
    };
    result.into()
}

#[cfg(feature = "region")]
struct RegionFromCountryCodeStr(String, String);

impl RegionFromCountryCodeStr {
    pub fn new(mut country_code: &str, mut region_name: &str) -> Self {
        // Workaround: https://github.com/johnstonskj/locale-codes/issues/3
        if country_code == "nan" {
            country_code = "NA";
        }
        if region_name == "Americas" {
            // We can be more specific!
            // http://www.geocountries.com/country/codes/north-america
            if matches!(
                country_code,
                "AI" | "AG"
                    | "AW"
                    | "BB"
                    | "BZ"
                    | "BM"
                    | "BQ"
                    | "VG"
                    | "CA"
                    | "KY"
                    | "CR"
                    | "CU"
                    | "CW"
                    | "DM"
                    | "DO"
                    | "SV"
                    | "GL"
                    | "GD"
                    | "GP"
                    | "GT"
                    | "HT"
                    | "HN"
                    | "JM"
                    | "MQ"
                    | "MX"
                    | "MS"
                    | "AN"
                    | "NI"
                    | "PA"
                    | "PR"
                    | "BL"
                    | "KN"
                    | "LC"
                    | "MF"
                    | "PM"
                    | "VC"
                    | "SX"
                    | "BS"
                    | "TT"
                    | "TC"
                    | "US"
                    | "VI"
            ) {
                region_name = "NorthAmerica";
            } else {
                region_name = "SouthAmerica";
            }
        }
        Self(country_code.to_owned(), region_name.to_owned())
    }
}

#[cfg(feature = "region")]
impl quote::ToTokens for RegionFromCountryCodeStr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.0.to_owned();
        let ident = str_to_ident(&self.1);

        let ts: proc_macro2::TokenStream = {
            quote! {
               #name => Some(Self::#ident)
            }
        }
        .into();

        tokens.extend(ts);
    }
}

#[allow(unused)]
fn str_to_ident(s: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(s, Span::call_site())
}
