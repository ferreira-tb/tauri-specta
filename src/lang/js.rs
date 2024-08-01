use heck::ToLowerCamelCase;
use specta::datatype::FunctionResultVariant;
use specta_typescript::js_doc;

use crate::{ExportContext, LanguageExt};

use super::js_ts;

const GLOBALS: &str = include_str!("./globals.js");

impl LanguageExt for specta_jsdoc::JSDoc {
    fn render_commands(&self, cfg: &ExportContext) -> Result<String, Self::Error> {
        let commands = cfg
            .commands
            .iter()
            .map(|function| {
                let jsdoc = {
                    let ret_type = js_ts::handle_result(function, &cfg.type_map, &self.0)?;

                    let mut builder = js_doc::Builder::default();

                    if let Some(d) = function.deprecated() {
                        builder.push_deprecated(d);
                    }

                    if !function.docs().is_empty() {
                        builder.extend(function.docs().split("\n"));
                    }

                    builder.extend(function.args().flat_map(|(name, typ)| {
                        specta_typescript::datatype(
                            &self.0,
                            &FunctionResultVariant::Value(typ.clone()),
                            &cfg.type_map,
                        )
                        .map(|typ| {
                            let name = name.to_lower_camel_case();

                            format!("@param {{ {typ} }} {name}")
                        })
                    }));
                    builder.push(&format!("@returns {{ Promise<{ret_type}> }}"));

                    builder.build()
                };

                Ok(js_ts::function(
                    &jsdoc,
                    &function.name().to_lower_camel_case(),
                    // TODO: Don't `collect` the whole thing
                    &js_ts::arg_names(&function.args().cloned().collect::<Vec<_>>()),
                    None,
                    &js_ts::command_body(&cfg.plugin_name, function, false),
                ))
            })
            .collect::<Result<Vec<_>, Self::Error>>()?
            .join(",\n");

        Ok(format!(
            r#"export const commands = {{
    {commands}
}}"#
        ))
    }

    fn render_events(&self, cfg: &ExportContext) -> Result<String, Self::Error> {
        if cfg.events.is_empty() {
            return Ok(Default::default());
        }

        let (events_types, events_map) =
            js_ts::events_data(&cfg.events, &self.0, &cfg.plugin_name, &cfg.type_map)?;

        let events = {
            let mut builder = js_doc::Builder::default();

            builder.push("@type {typeof __makeEvents__<{");
            builder.extend(events_types);
            builder.push("}>}");

            builder.build()
        };

        Ok(format! {
            r#"
{events}
const __typedMakeEvents__ = __makeEvents__;

export const events = __typedMakeEvents__({{
{events_map}
}})"#
        })
    }

    fn render(&self, cfg: &ExportContext) -> Result<String, Self::Error> {
        let dependant_types = cfg
            .type_map
            .iter()
            .map(|(_sid, ndt)| js_doc::typedef_named_datatype(&self.0, ndt, &cfg.type_map))
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.join("\n"))?;

        js_ts::render_all_parts::<Self>(self, cfg, &dependant_types, GLOBALS, &self.0.header)
    }
}
