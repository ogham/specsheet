use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use horrorshow::html;
use serde::Serialize;
use spec_exec::RanCommand;

use crate::input::InputSource;
use crate::results::{ResultsSection, Stats};


#[derive(PartialEq, Debug)]
pub struct DocumentPaths {
    pub html_path: Option<PathBuf>,
    pub json_path: Option<PathBuf>,
    pub toml_path: Option<PathBuf>,
}

impl DocumentPaths {
    pub fn write(&self, run: CompletedRun<'_>) -> io::Result<()> {

        if let Some(path) = &self.html_path {
            HtmlPage.write(&path, &run)?;
        }

        if let Some(path) = &self.json_path {
            JsonDoc.write(&path, &run)?;
        }

        if let Some(path) = &self.toml_path {
            TomlDoc.write(&path, &run)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct CompletedRun<'a> {
    pub sections: Vec<CompletedSection>,

    #[serde(skip)]
    pub commands: Vec<&'a RanCommand>,

    pub totals: Stats,
}

#[derive(Debug, Serialize)]
pub struct CompletedSection {
    pub input: InputSource,
    pub results: ResultsSection,
}


#[derive(Debug, PartialEq)]
pub struct JsonDoc;

impl JsonDoc {
    pub fn write(&self, path: &Path, run: &CompletedRun<'_>) -> io::Result<()> {
        let mut file = File::create(path)?;

        write!(file, "{}", serde_json::json!(run))?;

        Ok(())
    }
}


#[derive(Debug, PartialEq)]
pub struct TomlDoc;

impl TomlDoc {
    pub fn write(&self, path: &Path, run: &CompletedRun<'_>) -> io::Result<()> {
        let mut file = File::create(path)?;

        write!(file, "{}", toml::to_string(&run).unwrap())?;

        Ok(())
    }
}


#[derive(Debug, PartialEq)]
pub struct HtmlPage;

impl HtmlPage {
    pub fn write(&self, path: &Path, run: &CompletedRun<'_>) -> io::Result<()> {
        let mut file = File::create(path)?;

        let html = html! {
            html {
                head {
                    title : "Specsheet results";
                    meta(charset="utf-8");
                }
                body {
                    h1 {
                        : "Specsheet results"
                    }

                    @ for section in &run.sections {
                        section {
                            h2 {
                                : "Section"
                            }

                            ul {
                                @for output in &section.results.check_outputs {
                                    li {
                                        span {
                                            : &output.message
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };

        write!(file, "{}", html)?;

        Ok(())
    }
}
