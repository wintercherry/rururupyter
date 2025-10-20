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
struct Metadata {
    kernelspec: Option<KernelSpec>,
    language_info: Option<LanguageInfo>,
    #[serde(deserialize_with = "deserialize_orig_nbformat")]
    original_notebook_format: Option<u64>, // minimum is 1
    title: Option<String>,
    authors: Option<Vec<Author>>,

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

        let metadata: Metadata = serde_json::from_str(data).unwrap();

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
}
