use quote::quote;
use proc_macro::TokenStream;
use syn::*;
use syn::parse::Parse;
use syn::punctuated::Punctuated;

struct Slot{
    name: Option<Ident>,
    ty: Type,
    vis: Option<Expr>,
}

impl Slot{
    fn quote_bind_group(&self, device: &proc_macro2::TokenStream) -> proc_macro2::TokenStream{
        let name = match &self.name{
            Some(name) => quote!(Some(std::stringify!(#name))),
            None => quote!(None),
        };

        let ty = &self.ty;
        let ty = quote!(#ty);

        let vis = match &self.vis{
            Some(vis) => quote!(#vis),
            None => quote!(wgpu::ShaderStages::all()),
        };
        quote!{&<#ty>::create_bind_group_layout_vis(#device, #name, #vis).layout,}
    }

    fn quote_push_const(&self) -> proc_macro2::TokenStream{
        let _name = match &self.name{
            Some(name) => quote!(Some(std::stringify!(#name))),
            None => quote!(None),
        };

        let ty = &self.ty;
        let ty = quote!(#ty);

        let vis = match &self.vis{
            Some(vis) => quote!(#vis),
            None => quote!(wgpu::ShaderStages::all()),
        };
        quote!{<#ty>::push_const_layout(#vis),}
    }
}

impl Parse for Slot{
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let name;
        let ty;
        let mut vis = None;

        if input.peek2(Token!(:)){
            name = input.parse()?;
            let _dd: Token!(:) = input.parse()?;
            ty = input.parse()?;
            if input.peek(Token!(=>)){
                let _arrow_tok: Token!(=>) = input.parse()?;
                vis = Some(input.parse()?);
            }
        }
        else if input.peek2(Token!(=>)){
            name = None;
            ty = input.parse()?;
            if input.peek(Token!(=>)){
                let _arrow_tok: Token!(=>) = input.parse()?;
                vis = Some(input.parse()?);
            }
        }
        else{
            name = None;
            vis = None;
            ty = input.parse()?;
        }

        Ok(Self{
            name,
            ty,
            vis,
        })
    }
}

struct Group{
    slots: Punctuated<Slot, Token!(,)>,
}

impl Group{
    pub fn quote_bind_group(&self, device: &proc_macro2::TokenStream) -> proc_macro2::TokenStream{
        let slots: Vec<proc_macro2::TokenStream> = self.slots.iter()
            .map(|x|{x.quote_bind_group(device)}).collect();

        quote!{&[#(#slots)*]}
    }
    pub fn quote_push_const(&self) -> proc_macro2::TokenStream{
        let slots: Vec<proc_macro2::TokenStream> = self.slots.iter()
            .map(|x|{x.quote_push_const()}).collect();

        quote!{&[#(#slots)*]}
    }

}

impl Parse for Group{
    fn parse(input: parse::ParseStream) -> Result<Self>{

        let content;

        let _brace: token::Brace = braced!(content in input);

        let bind_groups = content.parse_terminated(Slot::parse)?;

        Ok(Self{
            slots: bind_groups,
        })
    }
}

mod kw{
    syn::custom_keyword!(bind_groups);
    syn::custom_keyword!(push_constants);
}

struct PipelineLayout{
    bind_groups: Option<Group>,
    push_constants: Option<Group>,
}

impl Parse for PipelineLayout{
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let mut bind_groups: Option<Group> = None;
        let mut push_constants: Option<Group> = None;

        // Parse the first group,
        if input.peek(kw::bind_groups){
            if bind_groups.is_none(){
                let _kw: kw::bind_groups = input.parse()?;
                let _dd: Token!(:) = input.parse()?;

                bind_groups = Some(input.parse()?);
            }
        }
        else if input.peek(kw::push_constants){
            if push_constants.is_none(){
                let _kw: kw::push_constants = input.parse()?;
                let _dd: Token!(:) = input.parse()?;

                push_constants = Some(input.parse()?);
            }
        }
        else{
            if bind_groups.is_none(){
                bind_groups = Some(input.parse()?);
            }
        }

        if input.peek(Token!(,)){
            let _c: Token!(,) = input.parse()?;
        }
        // Parse the second group
        if input.peek(kw::bind_groups){
            if bind_groups.is_none(){
                let _kw: kw::bind_groups = input.parse()?;
                let _dd: Token!(:) = input.parse()?;

                bind_groups = Some(input.parse()?);
            }
        }
        else if input.peek(kw::push_constants){
            if push_constants.is_none(){
                let _kw: kw::push_constants = input.parse()?;
                let _dd: Token!(:) = input.parse()?;

                push_constants = Some(input.parse()?);
            }
        }
        else{
            if push_constants.is_none(){
                push_constants = Some(input.parse()?);
            }
        }
        if input.peek(Token!(,)){
            let _c: Token!(,) = input.parse()?;
        }

        Ok(Self{
            bind_groups,
            push_constants,
        })
    }
}

struct MacroInput{
    device: Expr,
    pipeline_layout: PipelineLayout,
}

impl Parse for MacroInput{
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let device = input.parse()?;

        let _c: Token!(,) = input.parse()?;

        let pipeline_layout = input.parse()?;

        Ok(Self{
            device,
            pipeline_layout,
        })
    }
}

impl MacroInput{
    fn quote(&self) -> proc_macro2::TokenStream{
        let device = &self.device;
        let device_ts = quote!{#device};

        let bind_groups = match &self.pipeline_layout.bind_groups{
            Some(bind_groups) => bind_groups.quote_bind_group(&device_ts),
            None => quote!{&[]},
        };

        let push_constants = match &self.pipeline_layout.push_constants{
            Some(push_constants) => push_constants.quote_push_const(),
            None => quote!{&[]},
        };

        quote!{
            PipelineLayout::new(#device, #bind_groups, #push_constants, None)
        }
    }
}

pub fn generate_pipeline_layout(input: TokenStream) -> TokenStream{
    let macro_input = syn::parse_macro_input!(input as MacroInput);

    macro_input.quote().into()
}
