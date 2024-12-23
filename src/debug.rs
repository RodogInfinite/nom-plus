use crate::prelude::*;
use std::{
    env,
    fmt::{Debug, Formatter},
};

use annotate_snippets::{Level, Renderer, Snippet};
use std::path::PathBuf;

impl<'a> Debug for ContextError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(debug_assertions)]
        {
            self.fmt_annotation(f)
        }
        #[cfg(not(debug_assertions))]
        {
            Ok(())
        }
    }
}
impl<'a> ContextError<'a> {
    #[allow(clippy::unnecessary_wraps, clippy::range_plus_one)]
    #[cfg(debug_assertions)]
    pub fn fmt_annotation(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("unknown"));

        let context = self.context();
        dbg!("CONTEXT!", &context);
        let signature = &context.signature;
        let signature_text = &signature.source_text;
        let signature_line_number = signature.line_number;
        let parser_contexts = context.parser_contexts();

        let message = match self.combine_parser_sources() {
            Ok(source_capture) => {
                let parser_context_span = source_capture.span_length.unwrap();
                let parser_context_text = source_capture.source_text.into_boxed_str();
                let parser_context_line_number = source_capture.line_number;
                Level::Error
                    .title("ContextError")
                    .snippet(
                        Snippet::source(signature_text.as_str())
                            .origin(self.file.unwrap_or_default())
                            .line_start(signature_line_number),
                    )
                    .snippet(
                        Snippet::source(Box::leak(parser_context_text))
                            .line_start(parser_context_line_number)
                            .annotation(
                                Level::Error
                                    .span(
                                        source_capture.start_column.unwrap()
                                            ..parser_context_span + 1, // This is a hack. when trying to capture bindings such as (remaining,captured) = some_parser(input), we have to manually add the space after the eq_token and that leads to the span being 1 short. this may cause future problems with being outside of the SourceAnnotation range buffer
                                    )
                                    .label("error occurred here"),
                            ), // .annotation(
                               // Level::Info.span(0..4).label(self.input.unwrap_or_default()),
                               // ),
                    )
                    .footer(
                        Level::Info.title(self.input.unwrap_or_default()), // .span(0..4)
                    )
            }

            _e => {
                Level::Error.title("HANDLE ALTERNATIVE CASE ERROR FOR PARSERS ON DIFFERENT LINES")
            }
        };
        // dbg!("SOURCE CAPTURE", &source_capture);

        // dbg!("SOURCE?", &source_capture);
        // dbg!("MESSAGE", &message);

        let style = anstyle::Style::new().fg_color(Some(anstyle::AnsiColor::BrightRed.into()));
        let renderer = Renderer::styled().line_no(style);
        let rendered_output = renderer.render(message);
        let _ = writeln!(f, "{rendered_output}").is_ok();
        Ok(())
    }
}
