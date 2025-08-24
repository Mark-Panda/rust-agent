#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::*;
    use crate::prompt_template::PromptRenderer;

    #[test]
    fn test_prompt_renderer() {
        let renderer = PromptRenderer::new();
        let result = renderer.render(
            "test tool list",
            "test OS",
            "test files"
        );
        
        assert!(result.contains("test tool list"));
        assert!(result.contains("test OS"));
        assert!(result.contains("test files"));
    }

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        registry.register(ReadFileTool);
        registry.register(WriteFileTool);
        
        assert_eq!(registry.tools.len(), 2);
        assert!(registry.get_tool("read_file").is_some());
        assert!(registry.get_tool("write_to_file").is_some());
        assert!(registry.get_tool("nonexistent").is_none());
    }

    #[test]
    fn test_tool_list_generation() {
        let mut registry = ToolRegistry::new();
        registry.register(ReadFileTool);
        registry.register(WriteFileTool);
        
        let tool_list = registry.get_tool_list();
        assert!(tool_list.contains("read_file"));
        assert!(tool_list.contains("write_to_file"));
    }
}
