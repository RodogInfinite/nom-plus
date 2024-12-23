use crate::ContextError;
#[cfg(debug_assertions)]
use crate::{FunctionContext, ParserSourceCapture, SourceCapture};
use quote::{quote, ToTokens};
impl<'a> ToTokens for ContextError<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let message = self
            .message
            .map_or_else(|| quote! {None}, |message| quote! {Some(#message)});

        #[cfg(debug_assertions)]
        let context = self.as_ref().context();

        let input = self
            .input
            .map_or_else(|| quote! {None}, |input| quote! {Some(#input)});

        #[cfg(debug_assertions)]
        tokens.extend(quote! {
            ContextError {
                message: #message,
                context: #context,
                file: Some(file!()),
                input: #input,
            }
        });

        #[cfg(not(debug_assertions))]
        tokens.extend(quote! {
            ContextError {
                message: #message,
                file: Some(file!()),
                input: #input,
            }
        });
    }
}
#[cfg(debug_assertions)]
impl ToTokens for FunctionContext {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let signature = self.signature();
        let parser_contexts = self.parser_contexts.as_ref().map_or_else(
            || quote! { None },
            |parser_contexts| quote! {Some(vec![#(#parser_contexts,)*])},
        );

        let parser_context_failure_index = self
            .parser_context_failure_index
            .map_or_else(|| quote! { None }, |index| quote! { Some(#index)});

        let nested_parser_contexts = self.nested_parser_contexts.as_ref().map_or_else(
            || quote! { None },
            |nested_parser_contexts| quote! {Some(vec![#(#nested_parser_contexts,)*])},
        );

        let closing_tokens = self.closing_tokens.as_ref().map_or_else(
            || quote! { None },
            |closing_tokens| quote! { Some(#closing_tokens)},
        );

        tokens.extend(quote! {
            FunctionContext{
                signature: #signature,
                parser_contexts: #parser_contexts,
                parser_context_failure_index: #parser_context_failure_index,
                nested_parser_contexts: #nested_parser_contexts,
                closing_tokens: #closing_tokens,
            }
        });
    }
}

#[cfg(debug_assertions)]
impl ToTokens for SourceCapture {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let source_text = &self.source_text;
        let line_number = self.line_number;

        let start_column = self.start_column.map_or_else(
            || quote! { None },
            |start_column| quote! {Some(#start_column)},
        );

        let end_column = self
            .end_column
            .map_or_else(|| quote! { None }, |end_column| quote! { Some(#end_column)});

        let span_length = self.end_column.map_or_else(
            || quote! { None },
            |span_length| quote! { Some(#span_length)},
        );

        tokens.extend(quote! {
            SourceCapture {
                source_text: #source_text.to_string(),
                line_number: #line_number,
                start_column: #start_column,
                end_column: #end_column,
                span_length: #span_length,
            }
        });
    }
}

#[cfg(debug_assertions)]
impl ToTokens for ParserSourceCapture {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let binding_pattern = self.binding_pattern().as_ref().map_or_else(
            || quote! {None},
            |binding_pattern| quote! {Some(#binding_pattern)},
        );
        // dbg!("WHAT HERE", &let_binding_pattern);
        let ident = self.ident();
        let pattern = self.pattern();
        let nested_parsers = self.nested_parsers().as_ref().map_or_else(
            || quote! { None },
            |nested_parsers| quote! {Some(vec![#(#nested_parsers,)*])},
        );

        let input = self.input();

        tokens.extend(quote! {
        ParserSourceCapture {
            binding_pattern: #binding_pattern,
            ident: #ident,
            pattern: #pattern,
            nested_parsers: #nested_parsers,
            input: #input,
        }
        });
    }
}
