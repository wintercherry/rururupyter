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
struct Metadata {
    kernelspec: Option<KernelSpec>,
    language_info: Option<LanguageInfo>,
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
}
