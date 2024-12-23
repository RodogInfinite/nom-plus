pub mod debug;
pub mod to_tokens;
use nom::{IResult, Parser};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use std::fmt::{Debug, Display, Formatter};
use syn::{spanned::Spanned, ItemFn};

#[allow(unused_imports)]
pub mod prelude {
    pub use super::ContextError;
    #[cfg(debug_assertions)]
    pub use super::{
        debug::*, map_parser_err, FunctionContext, ParserSourceCapture, SourceCapture,
    };
}

#[derive(Clone, Default)]
pub struct ContextError<'a> {
    pub message: Option<&'a str>,
    #[cfg(debug_assertions)]
    pub context: FunctionContext,
    #[cfg(debug_assertions)]
    pub file: Option<&'static str>,
    pub input: Option<&'a str>,
}

impl<'a> ContextError<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            message: None,
            #[cfg(debug_assertions)]
            context: FunctionContext::default(),
            #[cfg(debug_assertions)]
            file: None,
            input: None,
        }
    }

    #[must_use]
    pub fn message(&self) -> &Option<&'a str> {
        &self.message
    }

    #[must_use]
    #[cfg(debug_assertions)]
    pub fn context(&self) -> &FunctionContext {
        &self.context
    }

    #[must_use]
    pub fn input(&self) -> &Option<&'a str> {
        &self.input
    }

    pub fn set_message(&mut self, message: &'a str) {
        self.message = Some(message);
    }

    #[cfg(debug_assertions)]
    pub fn set_context(&mut self, context: FunctionContext) {
        self.context = context;
    }

    pub fn set_input(&mut self, input: &'a str) {
        self.input = Some(input);
    }
    #[cfg(debug_assertions)]
    pub fn combine_parser_sources(&self) -> Result<SourceCapture, Box<dyn std::error::Error>> {
        self.context
            .parser_contexts
            .as_ref()
            .ok_or_else(|| "No parser contexts available".to_string())?
            .iter()
            .map(|parser_source_capture| {
                dbg!(&parser_source_capture.binding_pattern);
                let line_number = parser_source_capture.ident.line_number;
                let mut source_capture_state = SourceCapture::default();
                if let Some(binding_pattern) = &parser_source_capture.binding_pattern {
                    source_capture_state
                        .merge_source(binding_pattern, line_number)
                        .merge_source(&parser_source_capture.ident, line_number)
                        .merge_source(&parser_source_capture.pattern, line_number)
                        .merge_source(&parser_source_capture.input, line_number);
                } else {
                    source_capture_state
                        .merge_source(&parser_source_capture.ident, line_number)
                        .merge_source(&parser_source_capture.pattern, line_number)
                        .merge_source(&parser_source_capture.input, line_number);
                }
                dbg!("SOURCE CAPTURE", &source_capture_state);
                Ok(source_capture_state.clone())
            })
            .next()
            .ok_or_else(|| "No parser contexts available".to_string())?
    }
}

impl<'a> AsRef<ContextError<'a>> for ContextError<'a> {
    fn as_ref(&self) -> &ContextError<'a> {
        self
    }
}
impl<'a> Display for ContextError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(debug_assertions)]
        {
            // Ok(())
            self.fmt_annotation(f)
        }
        #[cfg(not(debug_assertions))]
        {
            Ok(())
        }
    }
}
impl<'a> nom::error::ParseError<&'a str> for ContextError<'a> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        ContextError::new()
    }

    fn append(_input: &'a str, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: &'a str, _c: char) -> Self {
        ContextError::new()
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug, Default)]
pub struct FunctionContext {
    pub signature: SourceCapture,
    pub parser_contexts: Option<Vec<ParserSourceCapture>>,
    pub parser_context_failure_index: Option<usize>,
    pub nested_parser_contexts: Option<Vec<ParserSourceCapture>>,
    pub closing_tokens: Option<SourceCapture>,
}

#[cfg(debug_assertions)]
impl FunctionContext {
    #[must_use]
    pub fn signature(&self) -> &SourceCapture {
        &self.signature
    }

    #[must_use]
    pub fn parser_contexts(&self) -> &Option<Vec<ParserSourceCapture>> {
        &self.parser_contexts
    }

    #[must_use]
    pub fn parser_context_failure_index(&self) -> &Option<usize> {
        &self.parser_context_failure_index
    }

    #[must_use]
    pub fn nested_parser_contexts(&self) -> &Option<Vec<ParserSourceCapture>> {
        &self.nested_parser_contexts
    }

    #[must_use]
    pub fn closing_tokens(&self) -> &Option<SourceCapture> {
        &self.closing_tokens
    }

    #[must_use]
    pub fn has_parser_contexts(&self) -> bool {
        self.parser_contexts.is_some()
    }

    #[must_use]
    pub fn has_nested_parser_contexts(&self) -> bool {
        self.nested_parser_contexts.is_some()
    }

    #[must_use]
    pub fn set_signature(
        mut self,
        item_fn: &ItemFn,
        signature_source_capture: &mut SourceCapture,
    ) -> Self {
        let line_number = item_fn.sig.ident.span().start().line;
        let sig_text = item_fn.format();
        let _ = signature_source_capture
            .set_source_text(sig_text)
            .set_line_number(line_number);
        self.signature = std::mem::take(signature_source_capture);
        self
    }

    pub fn set_parser_context(&mut self, parser_context_source_capture: ParserSourceCapture) {
        self.parser_contexts
            .get_or_insert_with(Vec::new)
            .push(parser_context_source_capture);
    }

    #[must_use]
    pub fn set_parser_context_failure_index(mut self, index: usize) -> Self {
        self.parser_context_failure_index = Some(index);
        self
    }

    pub fn set_nested_parser_context(&mut self, nested_context: ParserSourceCapture) {
        self.nested_parser_contexts
            .get_or_insert_with(Vec::new)
            .push(nested_context);
    }

    #[must_use]
    pub fn set_closing_tokens(mut self, closing_tokens_source_capture: SourceCapture) -> Self {
        self.closing_tokens = Some(closing_tokens_source_capture);
        self
    }
}

#[cfg(debug_assertions)]
impl AsRef<FunctionContext> for FunctionContext {
    fn as_ref(&self) -> &FunctionContext {
        self
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SourceCapture {
    pub source_text: String,
    pub line_number: usize,
    pub start_column: Option<usize>,
    pub end_column: Option<usize>,
    pub span_length: Option<usize>,
}

#[cfg(debug_assertions)]
impl SourceCapture {
    // #[must_use]
    // pub fn source_text(&self) -> &String {
    //     &self.source_text
    // }

    // #[must_use]
    // pub fn line_number(&self) -> &usize {
    //     &self.line_number
    // }

    // #[must_use]
    // pub fn start_column(&self) -> &Option<usize> {
    //     &self.start_column
    // }

    // #[must_use]
    // pub fn end_column(&self) -> &Option<usize> {
    //     &self.end_column
    // }

    // #[must_use]
    // pub fn span_length(&self) -> &Option<usize> {
    //     &self.span_length
    // }

    #[must_use]
    pub fn set_source_text(&mut self, source_text: String) -> &mut Self {
        self.source_text = source_text;
        self
    }

    #[must_use]
    pub fn set_line_number(&mut self, line_number: usize) -> &mut Self {
        self.line_number = line_number;
        self
    }

    #[must_use]
    pub fn set_start_column(&mut self, start_column: usize) -> &mut Self {
        self.start_column = Some(start_column);
        self
    }

    // #[must_use]
    pub fn set_end_column(&mut self, end_column: usize) -> &mut Self {
        self.end_column = Some(end_column);
        self
    }

    #[must_use]
    pub fn set_span_length(&mut self, span_length: usize) -> &mut Self {
        self.span_length = Some(span_length);
        self
    }

    #[must_use]
    pub fn get_span_length(&self) -> Result<usize, Box<dyn std::error::Error>> {
        //TODO: rethink this. There may be some issues when the span is on a different line causing issues with the span_length calulation which assumes the parser is all on a single line
        Ok(self.end_column.expect("end_column not found")
            - self.start_column.expect("start_column not found"))
    }

    #[must_use]
    pub fn build(mut self, source_text: String, span: Span) -> Self {
        if self.source_text.is_empty() {
            self.set_source_text(source_text)
                .set_line_number(span.start().line)
                .set_start_column(span.start().column)
                .set_end_column(span.end().column);
        }
        self
    }

    fn merge_source(
        &mut self,
        source_capture_input: &SourceCapture,
        line_number: usize,
    ) -> &mut Self {
        if source_capture_input.line_number == line_number {
            if self.start_column.is_none() {
                self.line_number = line_number;
                if let Some(span_length) = self.span_length {
                    self.span_length =
                        Some(span_length + source_capture_input.start_column.unwrap());
                } else {
                    self.span_length = Some(source_capture_input.start_column.unwrap());
                }
                self.source_text.push_str(
                    " ".repeat(source_capture_input.start_column.unwrap())
                        .as_str(),
                );
                self.source_text.push_str(&source_capture_input.source_text);
                self.start_column = source_capture_input.start_column;
            } else {
                self.source_text.push_str(&source_capture_input.source_text);
            }
            if let Some(span_length) = self.span_length {
                self.span_length = Some(
                    span_length
                        + source_capture_input
                            .get_span_length()
                            .expect("Could not calculate source_capture's span_length"),
                );
            }
            self.end_column = source_capture_input.end_column;
        }

        self
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug, Default)]
pub struct ParserSourceCapture {
    pub binding_pattern: Option<SourceCapture>,
    pub ident: SourceCapture,
    pub pattern: SourceCapture,
    pub nested_parsers: Option<Vec<SourceCapture>>,
    pub input: SourceCapture,
}

#[cfg(debug_assertions)]
impl ParserSourceCapture {
    #[must_use]
    pub fn binding_pattern(&self) -> &Option<SourceCapture> {
        &self.binding_pattern
    }

    #[must_use]
    pub fn ident(&self) -> &SourceCapture {
        &self.ident
    }

    #[must_use]
    pub fn pattern(&self) -> &SourceCapture {
        &self.pattern
    }

    #[must_use]
    pub fn nested_parsers(&self) -> &Option<Vec<SourceCapture>> {
        &self.nested_parsers
    }

    #[must_use]
    pub fn input(&self) -> &SourceCapture {
        &self.input
    }

    pub fn set_binding_pattern(&mut self, binding_pattern: SourceCapture) {
        self.binding_pattern = Some(binding_pattern);
    }
    pub fn set_ident(&mut self, ident_source_capture: &SourceCapture) {
        self.ident = ident_source_capture.clone();
    }
    pub fn set_pattern(&mut self, pattern_source_capture: &SourceCapture) {
        self.pattern = pattern_source_capture.clone();
    }
    pub fn set_input(&mut self, input_source_capture: &SourceCapture) {
        self.input = input_source_capture.clone();
    }
    pub fn push_nested_parser_source(&mut self, parser_source_capture: &SourceCapture) {
        self.nested_parsers
            .get_or_insert_with(Vec::new)
            .push(parser_source_capture.clone());
    }
}

#[cfg(debug_assertions)]
impl AsRef<ParserSourceCapture> for ParserSourceCapture {
    fn as_ref(&self) -> &ParserSourceCapture {
        self
    }
}
trait RemoveWhitespace {
    fn remove_whitespace(&mut self) -> String
    where
        Self: ToString,
    {
        self.to_string().split_whitespace().collect::<String>()
    }
}
impl RemoveWhitespace for TokenStream {}

trait FnSignatureFormat {
    fn format(&self) -> String;
}

impl FnSignatureFormat for ItemFn {
    /*
      Unfortunately, when we try to use the `syn::Signature` directly,
      regardless of whether in token form using the `quote!{...}` macro or converted to a String using the `format!(...)` macro, the `syn::Signature` output field introduces a `/n` in between a second lifetime definition and its ident.

      For example:
      if the original source code's output is:
          `-> IResult<&'a str, &'a str, ContextError<'a>>`
      it becomes:
          `-> IResult<&'a str, &'a/nstr, ContextError<'a>>`

      but when we use the `syn::Signature.output` directly with `format!(...)` it no longer appears; however, when we use `quote!{...}`, the newline is still introduced even with the manually extracted fields. Although, the newline is rendered as an actual newline in the terminal output.

      Another unfortunate side effect, is that the conversion from TokenStream to String on the `syn::Signature` or its fields directly, introduces some formatting artifacts, most notably, extra whitespace in the conversion from `syn::Lifetimes` so we must explicitly handle those also.

      One last unfortunate occurance is that when using `quote!{...}`, there is no way to include the opening brace for the function's code block without it being surrounded by quotes. This is because it must be rendered as a `proc_macro2::Literal`. And `quote!` doesn't support bracket escaping such as:
          `quote!{ #some_output {{ <- not supported}`.
      This is a very niche case so I'm sure the value of allowing that capability is not worth the implementation effort since the majority of users wouldn't benefit from it. Would have been useful though..

      Note: Since our errors are being rendered with annotate_snippets at the final step in the process and since the annotation of the specific line the error occurs on happens after the function signature, we only need the opening brace in the snippet created from this output which will be followed by another snippet with the actual error occurance.

      Something like this would have also been applicable to this case:
          ```
            pub struct Brace {
                 span: Span
            }

            pub struct Braces {
                  open: Brace,
                  close: Brace,
            }

            pub struct Block {
                  pub braces: Braces,
                  pub stmts: Vec<Stmt>,
            }

            // Use:
            let vis = item_fn.vis;
            let sig = item_fn.sig;
            let brace = item_fn.block.braces.open;
            let fn_signature_text = quote!(#vis #sig #brace);
          ```
     Lamentably, without a significant rewrite to syn and proc_macro, this won't be feasable.
    */

    fn format(&self) -> String {
        let vis = self.vis.to_token_stream();
        let constness = if self.sig.constness.is_some() {
            format!("{} ", self.sig.constness.to_token_stream())
        } else {
            String::new()
        };
        let asyncness = if self.sig.asyncness.is_some() {
            format!("{} ", self.sig.asyncness.to_token_stream())
        } else {
            String::new()
        };
        let unsafety = if self.sig.unsafety.is_some() {
            format!("{} ", self.sig.unsafety.to_token_stream())
        } else {
            String::new()
        };
        let abi = if self.sig.abi.is_some() {
            format!("{} ", self.sig.abi.to_token_stream())
        } else {
            String::new()
        };
        let fn_token = self.sig.fn_token.to_token_stream();
        let ident = self.sig.ident.to_token_stream();
        let generics = self.sig.generics.to_token_stream().remove_whitespace();

        let sig_end_line = &self.sig.span().end().line;
        let output_end_line = &self.sig.output.span().end().line;
        let block_brace_start_line = &self.block.brace_token.span.open().start().line;
        let output_brace_difference = block_brace_start_line - output_end_line;
        let output_newline_brace = if output_brace_difference > 0 {
            "\n".repeat(output_brace_difference)
        } else {
            String::new()
        };
        // This is brittle, but no other quick solutions present themselves at this time
        let inputs = self
            .sig
            .inputs
            .to_token_stream()
            .to_string()
            .replace("& '", "&'")
            .replace(" :", ":");
        let output = self
            .sig
            .output
            .to_token_stream()
            .to_string()
            .replace(" < & ", "<&")
            .replace("& '", "&'")
            .replace(" >", ">")
            .replace("< '", "<'");

        format!(
            "{vis} {constness}{asyncness}{unsafety}{abi}{fn_token} {ident}{generics}({inputs}) {output}{output_newline_brace} {{"
        )
    }
}

pub fn map_parser_err<'a, Input, Output, ParserType, MapFn>(
    mut parser: ParserType,
    mut f: MapFn,
) -> impl FnMut(Input) -> IResult<Input, Output, ContextError<'a>>
where
    Input: Clone,
    ParserType: Parser<Input, Output, ContextError<'a>>,
    MapFn: FnMut(nom::Err<ContextError<'a>>) -> nom::Err<ContextError<'a>>,
{
    move |input: Input| parser.parse(input).map_err(&mut f)
}
