// Copyright (C) 2017 Élisabeth HENRY.
//
// This file is part of Crowbook.
//
// Crowbook is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published
// by the Free Software Foundation, either version 2.1 of the License, or
// (at your option) any later version.
//
// Caribon is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received ba copy of the GNU Lesser General Public License
// along with Crowbook.  If not, see <http://www.gnu.org/licenses/>.

use error::Result;
use book::Book;

use crowbook_text_processing::escape;


#[cfg(feature="syntect")]
use syntect;

/// Wrapper around syntect, so it can be more easily optionally compiled.
#[cfg(feature="syntect")]
pub struct Syntax {
    syntax_set: syntect::parsing::SyntaxSet,
    theme: syntect::highlighting::Theme,
}

#[cfg(not(feature="syntect"))]
pub struct Syntax {}

#[cfg(feature="syntect")]
impl Syntax {
    /// Creates a new Syntax wrapper
    pub fn new(book: &Book, theme_name: &str) -> Syntax {
        let mut theme_set = syntect::highlighting::ThemeSet::load_defaults();
        let theme = match theme_set.themes.remove(theme_name) {
            Some(theme) => theme,
            None => {
                book.logger.error(lformat!("could not set syntect theme to {theme}, defaulting to \"InspiredGithub\"",
                                           theme = theme_name));
                book.logger.info(lformat!("valid theme names are: {themes}",
                                          themes = theme_set.themes
                                          .keys()
                                          .map(|s| s.to_owned())
                                          .collect::<Vec<_>>()
                                          .join(", ")));
                theme_set.themes.remove("InspiredGitHub").unwrap()
            }
        };
        Syntax {
            syntax_set: syntect::parsing::SyntaxSet::load_defaults_nonewlines(),
            theme: theme,
        }
    }
    
    /// Convert a string containing code to HTML
    pub fn to_html(&self, code: &str, language: &str) -> Result<String> {
        let language = strip_language(language);
        let syntax = self.syntax_set.find_syntax_by_token(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut h = syntect::easy::HighlightLines::new(syntax, &self.theme);
        let regions = h.highlight(code);
        Ok(format!("<pre>{}</pre>",
                   syntect::html::styles_to_coloured_html(&regions[..],
                                                          syntect::html::IncludeBackground::No)))
    }

    pub fn to_tex(&self, code: &str, language: &str) -> Result<String> {
        let language = strip_language(language);
        use latex::insert_breaks;
        use syntect::highlighting::{BLACK, FONT_STYLE_BOLD, FONT_STYLE_ITALIC, FONT_STYLE_UNDERLINE};
        let syntax = self.syntax_set.find_syntax_by_token(language)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut h = syntect::easy::HighlightLines::new(syntax, &self.theme);
        let regions = h.highlight(code);
        
        let mut result = String::with_capacity(code.len());
        for (style, text) in regions {
            let mut content = escape::tex(text).into_owned();
            content = insert_breaks(&content);
            content = content.replace('\n', "\\\\{}\n")
                .replace(' ', "\\hphantom{ }\\allowbreak{}");
            content = format!("\\texttt{{{}}}", content);
            if style.foreground != BLACK {
                let r = style.foreground.r as f32 / 255.0;
                let g = style.foreground.g as f32 / 255.0;
                let b = style.foreground.b as f32 / 255.0;
                content = format!("\\textcolor[rgb]{{{r}, {g}, {b}}}{{{text}}}",
                                  r = r,
                                  g = g,
                                  b = b,
                                  text = content);
            }
            if style.font_style.contains(FONT_STYLE_BOLD) {
                content = format!("\\textbf{{{}}}", content);
            }
            if style.font_style.contains(FONT_STYLE_ITALIC) {
                content = format!("\\emph{{{}}}", content);
            }
            if style.font_style.contains(FONT_STYLE_UNDERLINE) {
                content = format!("\\underline{{{}}}", content);
            }
            result.push_str(&content);
        }
        Ok(format!("{{\\sloppy {}}}", result))
    }
}

/// Strip language name of possible other infos, e.g. "rust,ignore" -> "rust"
/// Currently only ',' is done
fn strip_language(language: &str) -> &str {
    let splits: Vec<_> = language
        .split(|c: char| match c {
            ',' => true,
            _ => false
        })
        .collect();
    splits[0].trim()
}


#[cfg(not(feature="syntect"))]
impl Syntax {
    pub fn new(book: &Book, _: &str) -> Syntax {
        book.logger.error(lformat!("crowbook was compiled without syntect support, syntax highlighting will be disabled"));
        Syntax {}
    }

    pub fn to_html(&self, code: &str, language: &str) -> Result<String> {
        Ok(format!("<pre><code class = \"language-{lang}\">{code}</code></pre>",
                code = escape::html(code),
                lang = language))
    }

    pub fn to_tex(&self, code: &str, _: &str) -> Result<String> {
        Ok(format!("\\begin{{spverbatim}}{}\\end{{spverbatim}}\n",
                code))
    }
}
