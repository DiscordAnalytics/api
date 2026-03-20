use anyhow::Result;
use mrml::{mjml::Mjml, prelude::render::RenderOptions};
use tracing::warn;

use crate::utils::logger::LogCode;

pub fn compile_mjml(mjml_content: &str) -> Result<String> {
    let mjml = match Mjml::parse(mjml_content) {
        Ok(m) => m,
        Err(e) => {
            warn!(
                code = %LogCode::Mail,
                error = ?e,
                "Failed to parse MJML content"
            );
            return Err(anyhow::anyhow!("Failed to parse MJML content: {}", e));
        }
    };

    let render_options = RenderOptions::default();
    let html = mjml.element.render(&render_options)?;

    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_mjml() {
        let mjml = r#"
            <mjml>
                <mj-body>
                    <mj-section>
                        <mj-column>
                            <mj-text>Hello World</mj-text>
                        </mj-column>
                    </mj-section>
                </mj-body>
            </mjml>
        "#;

        let html = compile_mjml(mjml).unwrap();
        assert!(html.contains("Hello World"));
    }
}
