//! Serial Data Quality Analysis
//!
//! Provides real-time analysis of serial data to help users detect
//! baud rate mismatches and data quality issues.

/// Quality level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    /// Excellent: >85% printable ASCII
    Excellent,
    /// Good: 60-85% printable ASCII
    Good,
    /// Fair: 30-60% printable ASCII
    Fair,
    /// Poor: <30% printable ASCII, likely wrong baud rate
    Poor,
    /// No data received yet
    NoData,
}

impl QualityLevel {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Poor => "Poor - Check Baud Rate",
            Self::NoData => "No Data",
        }
    }

    /// Get the color for UI display (as RGB)
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Self::Excellent => (0, 200, 0),  // Green
            Self::Good => (100, 200, 0),     // Light green
            Self::Fair => (255, 200, 0),     // Yellow
            Self::Poor => (255, 80, 80),     // Red
            Self::NoData => (128, 128, 128), // Gray
        }
    }
}

/// Common protocol finger prints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    /// U-Boot Bootloader
    UBoot,
    /// Linux Kernel
    Linux,
    /// AT Command Set (Modems, ESP8266, etc)
    AtCommand,
    /// GRUB Bootloader
    Grub,
    /// Unknown/Generic
    Unknown,
}

impl ProtocolType {
    pub fn description(&self) -> &'static str {
        match self {
            Self::UBoot => "U-Boot Bootloader",
            Self::Linux => "Linux Kernel",
            Self::AtCommand => "AT Modem/Device",
            Self::Grub => "GRUB Bootloader",
            Self::Unknown => "Unknown Protocol",
        }
    }

    pub fn identify(data: &[u8]) -> Self {
        let s = String::from_utf8_lossy(data);
        if s.contains("U-Boot") || s.contains("Hit any key to stop autoboot") {
            Self::UBoot
        } else if s.contains("Linux version") || s.contains("Booting Linux") {
            Self::Linux
        } else if s.contains("AT+") || s.contains("OK\r\n") || s.contains("ERROR\r\n") {
            Self::AtCommand
        } else if s.contains("GRUB ") {
            Self::Grub
        } else {
            Self::Unknown
        }
    }
}

/// Data quality metrics for serial communication
#[derive(Debug, Clone, Default)]
pub struct DataQualityMetrics {
    /// Total bytes analyzed
    pub total_bytes: usize,
    /// Printable ASCII characters (0x20-0x7E, plus common control chars)
    pub printable_count: usize,
    /// Common control characters (newline, tab, carriage return)
    pub control_count: usize,
    /// Error-like bytes (0x00, 0xFF - often indicate framing errors)
    pub error_bytes: usize,
    /// High-bit bytes (0x80-0xFE - could be binary or wrong baud)
    pub high_bit_bytes: usize,
}

impl DataQualityMetrics {
    /// Analyze a chunk of data and return metrics
    pub fn analyze(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::default();
        }

        let mut metrics = Self {
            total_bytes: data.len(),
            ..Default::default()
        };

        for &byte in data {
            match byte {
                // Printable ASCII
                0x20..=0x7E => metrics.printable_count += 1,
                // Common control characters (newline, tab, CR)
                0x09 | 0x0A | 0x0D => metrics.control_count += 1,
                // Error-like bytes
                0x00 | 0xFF => metrics.error_bytes += 1,
                // High-bit bytes
                0x80..=0xFE => metrics.high_bit_bytes += 1,
                // Other control characters
                _ => {}
            }
        }

        metrics
    }

    /// Calculate overall quality score (0.0 - 1.0)
    pub fn quality_score(&self) -> f32 {
        if self.total_bytes == 0 {
            return 0.0;
        }

        // Readable characters = printable + common control
        let readable = self.printable_count + self.control_count;
        let base_score = readable as f32 / self.total_bytes as f32;

        // Penalty for error bytes (strong indicator of wrong baud rate)
        let error_penalty = (self.error_bytes as f32 / self.total_bytes as f32) * 0.5;

        // Penalty for high-bit bytes
        let high_bit_penalty = (self.high_bit_bytes as f32 / self.total_bytes as f32) * 0.3;

        (base_score - error_penalty - high_bit_penalty).clamp(0.0, 1.0)
    }

    /// Get the quality level classification
    pub fn quality_level(&self) -> QualityLevel {
        if self.total_bytes == 0 {
            return QualityLevel::NoData;
        }

        let score = self.quality_score();
        if score >= 0.85 {
            QualityLevel::Excellent
        } else if score >= 0.60 {
            QualityLevel::Good
        } else if score >= 0.30 {
            QualityLevel::Fair
        } else {
            QualityLevel::Poor
        }
    }

    /// Merge with another metrics (for rolling window analysis)
    pub fn merge(&mut self, other: &Self) {
        self.total_bytes += other.total_bytes;
        self.printable_count += other.printable_count;
        self.control_count += other.control_count;
        self.error_bytes += other.error_bytes;
        self.high_bit_bytes += other.high_bit_bytes;
    }

    /// Reset metrics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Rolling window analyzer for real-time quality tracking
#[derive(Debug, Clone)]
pub struct RollingQualityAnalyzer {
    /// Window size in bytes
    window_size: usize,
    /// Current metrics
    metrics: DataQualityMetrics,
    /// Bytes analyzed since last reset
    bytes_since_reset: usize,
    /// Last identified protocol
    last_protocol: ProtocolType,
}

impl RollingQualityAnalyzer {
    /// Create a new analyzer with the specified window size
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            metrics: DataQualityMetrics::default(),
            bytes_since_reset: 0,
            last_protocol: ProtocolType::Unknown,
        }
    }

    /// Process new incoming data
    pub fn process(&mut self, data: &[u8]) {
        let chunk_metrics = DataQualityMetrics::analyze(data);
        self.metrics.merge(&chunk_metrics);
        self.bytes_since_reset += data.len();

        // Protocol identification (on every chunk for now)
        let proto = ProtocolType::identify(data);
        if proto != ProtocolType::Unknown {
            self.last_protocol = proto;
        }

        // Reset if we've exceeded the window size
        if self.bytes_since_reset > self.window_size {
            // Keep only recent data by resetting to current chunk
            self.metrics = chunk_metrics;
            self.bytes_since_reset = data.len();
        }
    }

    /// Get current quality level
    pub fn quality_level(&self) -> QualityLevel {
        self.metrics.quality_level()
    }

    /// Get current quality score (0.0 - 1.0)
    pub fn quality_score(&self) -> f32 {
        self.metrics.quality_score()
    }

    /// Get the underlying metrics
    pub fn metrics(&self) -> &DataQualityMetrics {
        &self.metrics
    }

    /// Get the last identified protocol
    pub fn protocol(&self) -> ProtocolType {
        self.last_protocol
    }

    /// Reset the analyzer
    pub fn reset(&mut self) {
        self.metrics.reset();
        self.bytes_since_reset = 0;
        self.last_protocol = ProtocolType::Unknown;
    }
}

impl Default for RollingQualityAnalyzer {
    fn default() -> Self {
        Self::new(4096) // 4KB window by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_ascii() {
        let data = b"Hello, World!\r\n";
        let metrics = DataQualityMetrics::analyze(data);
        assert!(metrics.quality_score() > 0.9);
        assert_eq!(metrics.quality_level(), QualityLevel::Excellent);
    }

    #[test]
    fn test_garbage_data() {
        let data: Vec<u8> = vec![0x00, 0xFF, 0x00, 0xFF, 0x80, 0x81, 0x82];
        let metrics = DataQualityMetrics::analyze(&data);
        assert!(metrics.quality_score() < 0.3);
        assert_eq!(metrics.quality_level(), QualityLevel::Poor);
    }

    #[test]
    fn test_mixed_data() {
        let data = b"Hello\x00\xFF World";
        let metrics = DataQualityMetrics::analyze(data);
        let level = metrics.quality_level();
        assert!(level == QualityLevel::Fair || level == QualityLevel::Good);
    }

    #[test]
    fn test_empty_data() {
        let data: &[u8] = &[];
        let metrics = DataQualityMetrics::analyze(data);
        assert_eq!(metrics.quality_level(), QualityLevel::NoData);
    }

    #[test]
    fn test_rolling_analyzer() {
        let mut analyzer = RollingQualityAnalyzer::new(100);

        // Process good data
        analyzer.process(b"Hello World\r\n");
        assert!(analyzer.quality_score() > 0.8);

        // Process more good data
        analyzer.process(b"How are you?\r\n");
        assert!(analyzer.quality_score() > 0.8);
    }
}
