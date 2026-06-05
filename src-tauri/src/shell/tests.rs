// Integration tests for shell module

#[cfg(test)]
mod integration_tests {
    use crate::shell::*;

    #[test]
    fn test_module_exports() {
        // Verify all public types are accessible
        let _classifier = CommandClassifier::new();

        // This test just ensures compilation succeeds and exports are correct
    }

    #[test]
    fn test_command_tier_enum() {
        // Verify enum variants exist and can be compared
        assert_eq!(CommandTier::Tier1, CommandTier::Tier1);
        assert_ne!(CommandTier::Tier1, CommandTier::Tier2);
        assert_ne!(CommandTier::Tier2, CommandTier::Tier3);
    }
}
