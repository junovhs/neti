use crate::config::Config;

/// Configuration items that can be edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigItem {
    MaxTokens,
    MaxComplexity,
    MaxNesting,
    MaxArgs,
    MaxWords,
    MaxLcom4,
    MinAhf,
    MaxCbo,
    MaxSfout,
    AutoCopy,
    WriteFixPacket,
    RequirePlan,
    AutoPromote,
    LocalityMode,
    LocalityMaxDistance,
}

impl ConfigItem {
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![
            Self::MaxTokens,
            Self::MaxComplexity,
            Self::MaxNesting,
            Self::MaxArgs,
            Self::MaxWords,
            Self::MaxLcom4,
            Self::MinAhf,
            Self::MaxCbo,
            Self::MaxSfout,
            Self::AutoCopy,
            Self::WriteFixPacket,
            Self::RequirePlan,
            Self::AutoPromote,
            Self::LocalityMode,
            Self::LocalityMaxDistance,
        ]
    }
    
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::MaxTokens => "Max file tokens",
            // UPDATED LABEL
            Self::MaxComplexity => "Max cognitive complexity",
            Self::MaxNesting => "Max nesting depth",
            Self::MaxArgs => "Max function args",
            Self::MaxWords => "Max function words",
            Self::MaxLcom4 => "Max LCOM4",
            Self::MinAhf => "Min AHF (%)",
            Self::MaxCbo => "Max CBO",
            Self::MaxSfout => "Max SFOUT",
            _ => self.label_toggles(),
        }
    }

    fn label_toggles(self) -> &'static str {
        match self {
            Self::AutoCopy => "Auto-copy to clipboard",
            Self::WriteFixPacket => "Write fix packet to file",
            Self::RequirePlan => "Require PLAN block",
            Self::AutoPromote => "Auto-promote on green",
            Self::LocalityMode => "Locality mode",
            Self::LocalityMaxDistance => "Locality max distance",
            _ => "Unknown",
        }
    }

    #[must_use]
    pub const fn is_boolean(self) -> bool {
        matches!(
            self,
            Self::AutoCopy | Self::WriteFixPacket | Self::RequirePlan | Self::AutoPromote
        )
    }

    #[must_use]
    pub const fn is_enum(self) -> bool {
        matches!(self, Self::LocalityMode)
    }

    #[must_use]
    pub fn get_value(self, config: &Config) -> String {
        if self.is_boolean() {
            return self.get_boolean_value(config);
        }
        
        if self.is_enum() {
             return format!("[{}]", config.rules.locality.mode);
        }

        self.get_numeric_value(config)
    }

    fn get_boolean_value(self, config: &Config) -> String {
         let checked = match self {
            Self::AutoCopy => config.preferences.auto_copy,
            Self::WriteFixPacket => config.preferences.write_fix_packet,
            Self::RequirePlan => config.preferences.require_plan,
            Self::AutoPromote => config.preferences.auto_promote,
            _ => false,
        };
        checkbox(checked)
    }

    fn get_numeric_value(self, config: &Config) -> String {
        let val = self.get_number(config);
        format!("[{val}]")
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_number(self, config: &Config) -> usize {
        match self {
            Self::MaxTokens => config.rules.max_file_tokens,
            // UPDATED FIELD NAME
            Self::MaxComplexity => config.rules.max_cognitive_complexity,
            Self::MaxNesting => config.rules.max_nesting_depth,
            Self::MaxArgs => config.rules.max_function_args,
            Self::MaxWords => config.rules.max_function_words,
            Self::MaxLcom4 => config.rules.max_lcom4,
            Self::MinAhf => config.rules.min_ahf as usize,
            Self::MaxCbo => config.rules.max_cbo,
            Self::MaxSfout => config.rules.max_sfout,
            Self::LocalityMaxDistance => config.rules.locality.max_distance,
            _ => 0,
        }
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn set_number(self, config: &mut Config, value: usize) {
        match self {
            Self::MaxTokens => config.rules.max_file_tokens = value,
            // UPDATED FIELD NAME
            Self::MaxComplexity => config.rules.max_cognitive_complexity = value,
            Self::MaxNesting => config.rules.max_nesting_depth = value,
            Self::MaxArgs => config.rules.max_function_args = value,
            Self::MaxWords => config.rules.max_function_words = value,
            Self::MaxLcom4 => config.rules.max_lcom4 = value,
            Self::MinAhf => config.rules.min_ahf = value as f64,
            Self::MaxCbo => config.rules.max_cbo = value,
            Self::MaxSfout => config.rules.max_sfout = value,
            Self::LocalityMaxDistance => config.rules.locality.max_distance = value,
            _ => {}
        }
    }

    pub fn toggle_boolean(self, config: &mut Config) {
        match self {
            Self::AutoCopy => config.preferences.auto_copy = !config.preferences.auto_copy,
            Self::WriteFixPacket => config.preferences.write_fix_packet = !config.preferences.write_fix_packet,
            Self::RequirePlan => config.preferences.require_plan = !config.preferences.require_plan,
            Self::AutoPromote => config.preferences.auto_promote = !config.preferences.auto_promote,
            _ => {}
        }
    }

    pub fn cycle_enum(self, config: &mut Config) {
        if self == Self::LocalityMode {
            config.rules.locality.mode = match config.rules.locality.mode.as_str() {
                "warn" => "error".to_string(),
                "error" => "off".to_string(),
                _ => "warn".to_string(),
            };
        }
    }
}

fn checkbox(checked: bool) -> String {
    if checked { "[x]".to_string() } else { "[ ]".to_string() }
}