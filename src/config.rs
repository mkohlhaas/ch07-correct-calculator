// config.rs - Configuration (alternative to Singleton from Chapter 6)

use crate::token::NumberFormat;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub enum AngleMode {
    Degrees,
    Radians,
}

#[derive(Debug, Clone)]
pub struct CalculatorConfig {
    pub precision: u32,
    pub angle_mode: AngleMode,
    pub notation: NumberFormat,
}

impl Default for CalculatorConfig {
    fn default() -> Self {
        Self {
            precision: 10,
            angle_mode: AngleMode::Radians,
            notation: NumberFormat::Decimal,
        }
    }
}

impl CalculatorConfig {
    // Factory methods for common configurations
    pub fn scientific() -> Self {
        Self {
            precision: 15,
            angle_mode: AngleMode::Radians,
            notation: NumberFormat::Scientific,
        }
    }

    pub fn engineering() -> Self {
        Self {
            notation: NumberFormat::Engineering,
            ..Default::default()
        }
    }
}

// Constants
pub const DEFAULT_PRECISION: u32 = 10;
pub const MAX_PRECISION: u32 = 100;

// If we need a global configuration (alternative to Singleton)
static CONFIG: OnceLock<CalculatorConfig> = OnceLock::new();

pub fn get_global_config() -> &'static CalculatorConfig {
    CONFIG.get_or_init(|| {
        // In a real application, this might load from a file or environment
        CalculatorConfig::default()
    })
}

// Thread-safe calculator with shared config
use std::sync::{Arc, Mutex};

pub struct CalculatorPool {
    shared_config: Arc<CalculatorConfig>,
    // In a real application, this would store calculator instances
    _calculators: Vec<()>,
}

impl CalculatorPool {
    pub fn new(config: CalculatorConfig) -> Self {
        Self {
            shared_config: Arc::new(config),
            _calculators: Vec::new(),
        }
    }

    pub fn get_config(&self) -> Arc<CalculatorConfig> {
        Arc::clone(&self.shared_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = CalculatorConfig::default();
        assert_eq!(config.precision, 10);
        assert!(matches!(config.angle_mode, AngleMode::Radians));
        assert!(matches!(config.notation, NumberFormat::Decimal));
    }

    #[test]
    fn scientific_config() {
        let config = CalculatorConfig::scientific();
        assert_eq!(config.precision, 15);
        assert!(matches!(config.angle_mode, AngleMode::Radians));
        assert!(matches!(config.notation, NumberFormat::Scientific));
    }

    #[test]
    fn engineering_config_keeps_defaults_except_notation() {
        let config = CalculatorConfig::engineering();
        assert_eq!(config.precision, 10);
        assert!(matches!(config.angle_mode, AngleMode::Radians));
        assert!(matches!(config.notation, NumberFormat::Engineering));
    }

    #[test]
    fn precision_constants() {
        assert_eq!(DEFAULT_PRECISION, 10);
        assert_eq!(MAX_PRECISION, 100);
    }

    #[test]
    fn global_config_is_singleton() {
        let first = get_global_config();
        let second = get_global_config();
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn calculator_pool_shares_config() {
        let pool = CalculatorPool::new(CalculatorConfig::scientific());
        let config = pool.get_config();
        assert_eq!(Arc::strong_count(&config), 2);
        assert_eq!(config.precision, 15);
        assert!(matches!(config.notation, NumberFormat::Scientific));
    }
}
