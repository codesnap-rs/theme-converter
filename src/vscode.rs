use std::fs;

use plist::{Dictionary, Value};
use serde::Deserialize;

use crate::parser::Parser;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TokenColour {
    Single {
        scope: String,
        settings: Settings,
    },
    Multi {
        scope: Vec<String>,
        settings: Settings,
    },
}

#[derive(Debug, Deserialize)]
struct VSCodeTheme {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    colors: serde_json::Value,
    #[serde(rename = "tokenColors", default)]
    token_colors: Vec<TokenColour>,
}

#[derive(Debug, Clone, Deserialize)]
struct Settings {
    #[serde(default)]
    foreground: Option<String>,
    #[serde(default)]
    background: Option<String>,
    #[serde(default, rename = "fontStyle")]
    font_style: Option<String>,
}

pub struct VSCodeThemeParser {
    theme: VSCodeTheme,
}

impl Parser for VSCodeThemeParser {
    fn parse(&self, name: &str) -> Dictionary {
        let get_color = |key: &str| -> Option<String> {
            self.theme
                .colors
                .get(key)
                .and_then(|v| v.as_str())
                .map(|s| s.into())
        };

        let global_settings = Some(self.build_settings_dict(
            "",
            &Settings {
                foreground: get_color("editor.foreground"),
                background: get_color("editor.background"),
                font_style: get_color("editor.fontStyle"),
            },
        ));

        let token_settings = self.theme.token_colors.iter().map(|tok| match tok {
            TokenColour::Single { scope, settings } => self.build_settings_dict(scope, settings),
            TokenColour::Multi { scope, settings } => {
                self.build_settings_dict(&scope.join(", "), settings)
            }
        });

        let settings_array: Vec<Value> =
            global_settings.into_iter().chain(token_settings).collect();

        Dictionary::from_iter([
            ("name", Value::String(name.to_string())),
            ("uuid", Value::String(uuid::Uuid::new_v4().to_string())),
            ("settings", Value::Array(settings_array)),
        ])
    }

    fn from_config(file_name: &str) -> anyhow::Result<Self> {
        let theme: VSCodeTheme = serde_json::from_str(&fs::read_to_string(file_name)?)?;

        Ok(Self { theme })
    }
}

impl VSCodeThemeParser {
    fn build_settings_dict(&self, scope: impl Into<String>, s: &Settings) -> Value {
        let inner = Dictionary::from_iter(
            [
                s.foreground
                    .clone()
                    .map(|v| ("foreground", Value::String(v))),
                s.background
                    .clone()
                    .map(|v| ("background", Value::String(v))),
                s.font_style
                    .clone()
                    .map(|v| ("fontStyle", Value::String(v))),
            ]
            .into_iter()
            .flatten(),
        );

        Value::Dictionary(Dictionary::from_iter([
            ("scope", Value::String(scope.into())),
            ("settings", Value::Dictionary(inner)),
        ]))
    }
}
