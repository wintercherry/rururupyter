use std::f32::consts::E;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct KernelSpec {
    name: String,
    display_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LanguageInfo {
    name: String,
    codemirror_mode: Option<String>,
    mimetype: Option<String>,
    file_extension: Option<String>,
    pygments_lexer: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Author {
    name: String,
}

fn deserialize_orig_nbformat<'de, D>(deserializer: D) -> std::result::Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<u64> = Option::deserialize(deserializer)?;
    if let Some(v) = value {
        if v < 1 {
            return Err(serde::de::Error::custom(
                "original_notebook_format must be at least 1",
            ));
        }
    }
    Ok(value)
}

#[derive(Serialize, Deserialize, Debug)]
struct NotebookMetadata {
    kernelspec: Option<KernelSpec>,
    language_info: Option<LanguageInfo>,
    #[serde(deserialize_with = "deserialize_orig_nbformat")]
    original_notebook_format: Option<u64>, // minimum is 1
    title: Option<String>,
    authors: Option<Vec<Author>>,
}

fn deserialize_nbformat<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: u64 = u64::deserialize(deserializer)?;
    if value != 4 {
        return Err(serde::de::Error::custom("nbformat must be 4"));
    }
    Ok(value)
}

fn deserialize_nbformat_minor<'de, D>(deserializer: D) -> std::result::Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: u64 = u64::deserialize(deserializer)?;
    if value < 5 {
        return Err(serde::de::Error::custom(
            "nbformat_minor must be 5 or greater",
        ));
    }
    Ok(value)
}

#[derive(Serialize, Deserialize, Debug)]
struct CellMetadata {
    // Placeholder for cell metadata fields
}

fn deserialize_cell_id<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer);
    if let unwrapped_value = value.unwrap() {
        // proceed to validate constraints
        if unwrapped_value.is_empty() {
            return Err(serde::de::Error::custom("Cell id cannot be empty"));
        }
    // must conform to the following constraints:
        if unwrapped_value.len() > 64 {
            return Err(serde::de::Error::custom(
                "Cell id cannot be longer than 64 characters",
            ));
        }
        // check if matches pattern
        let re = regex::Regex::new(r"^[a-zA-Z0-9\-_]+$").unwrap();
        if !re.is_match(&unwrapped_value) {
            return Err(serde::de::Error::custom(
                "Cell id must match pattern ^[a-zA-Z0-9-_]+$",
            ));
        }
        Ok(unwrapped_value)
    } else {
        Err(serde::de::Error::custom("Cell id must be a string"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RawCell {
    #[serde(deserialize_with = "deserialize_cell_id")]
    id: String,
    cell_type: String,
    source: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct MarkdownCell {
    #[serde(deserialize_with = "deserialize_cell_id")]
    id : String,
    cell_type: String,
    source: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CodeCell {
    #[serde(deserialize_with = "deserialize_cell_id")]
    id: String,
    cell_type: String,
    source: String,
    outputs: Vec<serde_json::Value>,
    execution_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Cell {
    RawCell,
    MarkdownCell,
    CodeCell,
}

#[derive(Serialize, Deserialize, Debug)]
struct Notebook {
    metadata: NotebookMetadata,
    #[serde(deserialize_with = "deserialize_nbformat")]
    nbformat: u64,
    #[serde(deserialize_with = "deserialize_nbformat_minor")]
    nbformat_minor: u64,
    cells: Vec<Cell>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // see nbformat-schema.json line 16
    #[test]
    fn test_kernel_spec() {

        // test that if name and display name are present, everything succeeds
        let data = r#"
        {
                "name": "python3",
                "display_name": "Python 3"
        }
        "#;

        let kernel_spec: KernelSpec = serde_json::from_str(data).unwrap();
        assert_eq!(kernel_spec.name, "python3");
        assert_eq!(kernel_spec.display_name, "Python 3");

        // test that if name is missing, deserialization fails
        let data_missing_name = r#"
        {
                "display_name": "Python 3"
        }
        "#;

        let result: Result<KernelSpec> = serde_json::from_str(data_missing_name);
        assert!(result.is_err());

        // test that if display name is missing, deserialization fails
        let data_missing_display_name = r#"
        {
                "name": "python3"
    }
        "#;

        let result: Result<KernelSpec> = serde_json::from_str(data_missing_display_name);
        assert!(result.is_err());
    }

    // see nbformat-schema.json line 31
    #[test]
    fn test_language_info() {
        let data = r#"
        {
                "name": "python",
                "codemirror_mode": "ipython",
                "mimetype": "text/x-python",
                "file_extension": ".py",
                "pygments_lexer": "ipython3"    
    }
        "#;

        let lang_info: LanguageInfo = serde_json::from_str(data).unwrap();
        assert_eq!(lang_info.name, "python");
        assert_eq!(lang_info.codemirror_mode.unwrap(), "ipython");
        assert_eq!(lang_info.mimetype.unwrap(), "text/x-python");
        assert_eq!(lang_info.file_extension.unwrap(), ".py");
        assert_eq!(lang_info.pygments_lexer.unwrap(), "ipython3");

        // test that name is required
        let data_missing_name = r#"
        {
                "codemirror_mode": "ipython"
                "mimetype": "text/x-python"
                "file_extension": ".py"
                "pygments_lexer": "ipython3"
    }
        "#;

        let result: Result<LanguageInfo> = serde_json::from_str(data_missing_name);
        assert!(result.is_err());
    }

    #[test]
    fn test_author() {
        let data = r#"
        {
                "name": "John Doe"
        }
        "#;

        let author: Author = serde_json::from_str(data).unwrap();
        assert_eq!(author.name, "John Doe");
    }

    // see nbformat-schema.json line 58
    #[test]
    fn test_original_notebook_format() {
        let data = r#"
        4
        "#;

        // test using deserialize_orig_nbformat directly
        let value: Option<u64> = serde_json::from_str(data).unwrap();
        assert_eq!(value.unwrap(), 4);
        // test that value less than 1 fails
        let data_invalid = r#"
        0
        "#; 
        let result: std::result::Result<Option<u64>, _> =
            deserialize_orig_nbformat(&mut serde_json::Deserializer::from_str(data_invalid));
        assert!(result.is_err());
    }

    // see nbformat-schema.json line 79
    #[test]
    fn test_nbformat_minor() {
        let data = r#"
        5
        "#;

        // test using deserialize_nbformat_minor directly
        let value: u64 = serde_json::from_str(data).unwrap();
        assert_eq!(value, 5);

        // test that value less than 5 fails
        let data_invalid = r#"
        4
        "#;
        let result: std::result::Result<u64, _> =
            deserialize_nbformat_minor(&mut serde_json::Deserializer::from_str(data_invalid));
        assert!(result.is_err());
    }

    // see nbformat-schema.json line 84
    #[test]
    fn test_nbformat() {
        let data = r#"
        4
        "#;

        // test using deserialize_nbformat directly
        let value: u64 = serde_json::from_str(data).unwrap();
        assert_eq!(value, 4);

        // test that value not equal to 4 fails
        let data_invalid = r#"
        3
        "#;
        let result: std::result::Result<u64, _> =
            deserialize_nbformat(&mut serde_json::Deserializer::from_str(data_invalid));
        assert!(result.is_err());
    }

    // see nbformat-schema.json line 98
    #[test]
    fn test_cell_id() {
        let data = r#"
        "cell-123_A"
        "#;
        let cell_id: String = deserialize_cell_id(&mut serde_json::Deserializer::from_str(data)).unwrap();
        assert_eq!(cell_id, "cell-123_A");
        // test that empty string fails
        let data_empty = r#""#;
        let result: std::result::Result<String, _> =
            serde_json::from_str(data_empty);
        assert!(result.is_err());
        // test that too long string fails
        let data_too_long = r#"
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_extra"
        "#;
        let result = deserialize_cell_id(&mut serde_json::Deserializer::from_str(data_too_long));
        assert!(result.is_err());
        // test that invalid pattern fails
        let data_invalid_pattern = r#"
        "cell@123!"
        "#;
        let result = deserialize_cell_id(&mut serde_json::Deserializer::from_str(data_invalid_pattern));
        assert!(result.is_err());
    }

    // see nbformat-schema.json line 8
    #[test]
    fn test_metadata() {
        let data = r#"
        {
                "kernelspec": {
                        "name": "python3",
                        "display_name": "Python 3"
                },
                "language_info": {
                        "name": "python",
                        "codemirror_mode": "ipython",
                        "mimetype": "text/x-python",
                        "file_extension": ".py",
                        "pygments_lexer": "ipython3"
                },
                "original_notebook_format": 4,
                "title": "Sample Notebook",
                "authors": [
                        {"name": "John Doe"},
                        {"name": "Jane Smith"}
                ]
    }    
     "#;

        let metadata: NotebookMetadata = serde_json::from_str(data).unwrap();

        // Check kernelspec
        let kernelspec = metadata.kernelspec.unwrap();
        assert_eq!(kernelspec.name, "python3");
        assert_eq!(kernelspec.display_name, "Python 3");

        // Check language_info
        let lang_info = metadata.language_info.unwrap();
        assert_eq!(lang_info.name, "python");
        assert_eq!(lang_info.codemirror_mode.unwrap(), "ipython");
        assert_eq!(lang_info.mimetype.unwrap(), "text/x-python");
        assert_eq!(lang_info.file_extension.unwrap(), ".py");
        assert_eq!(lang_info.pygments_lexer.unwrap(), "ipython3");

        // Check original_notebook_format
        assert_eq!(metadata.original_notebook_format.unwrap(), 4);

        // Check title
        assert_eq!(metadata.title.unwrap(), "Sample Notebook");

        // Check authors
        let authors = metadata.authors.unwrap();
        assert_eq!(authors.len(), 2);
        assert_eq!(authors[0].name, "John Doe");
        assert_eq!(authors[1].name, "Jane Smith");
    }

    #[test]
    fn test_notebook() {
        // fail until cells finished
        let data = r#"
        {
                "metadata": {
                        "kernelspec": {
                                "name": "python3",
                                "display_name": "Python 3"
                        },
                        "language_info": {
                                "name": "python",
                                "codemirror_mode": "ipython",
                                "mimetype": "text/x-python",
                                "file_extension": ".py",
                                "pygments_lexer": "ipython3"
                        },
                        "original_notebook_format": 4,
                        "title": "Sample Notebook",
                        "authors": [
                                {"name": "John Doe"},
                                {"name": "Jane Smith"}
                        ]
                },
                "nbformat": 4,
                "nbformat_minor": 5
                "cells": [] // Not implemented yet
        }
        "#;
        // output a note that cells are not implemented
        let notebook_result: Result<Notebook> = serde_json::from_str(data);
        assert!(notebook_result.is_err());
    }
}
