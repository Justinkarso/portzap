use crate::errors::{KillportError, Result};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub port: u16,
    pub protocol: Protocol,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl fmt::Display for ProcessInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PID {} ({}) on port {}/{}",
            self.pid, self.name, self.port, self.protocol
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillSignal {
    Term,
    Kill,
    Int,
    Hup,
}

impl fmt::Display for KillSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KillSignal::Term => write!(f, "SIGTERM"),
            KillSignal::Kill => write!(f, "SIGKILL"),
            KillSignal::Int => write!(f, "SIGINT"),
            KillSignal::Hup => write!(f, "SIGHUP"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct KillResult {
    pub process: ProcessInfo,
    pub success: bool,
    pub signal_sent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PortSpec {
    Single(u16),
    Range(u16, u16),
}

impl PortSpec {
    pub fn parse(s: &str) -> Result<Self> {
        if let Some((start_str, end_str)) = s.split_once('-') {
            let start: u16 = start_str
                .parse()
                .map_err(|_| KillportError::InvalidPortRange(s.to_string()))?;
            let end: u16 = end_str
                .parse()
                .map_err(|_| KillportError::InvalidPortRange(s.to_string()))?;
            if start > end {
                return Err(KillportError::InvalidPortRange(format!(
                    "start ({start}) > end ({end})"
                )));
            }
            if start == 0 {
                return Err(KillportError::InvalidPortRange(
                    "port 0 is not valid".to_string(),
                ));
            }
            Ok(PortSpec::Range(start, end))
        } else {
            let n: u32 = s.parse().map_err(|_| KillportError::InvalidPort(0))?;
            if n == 0 || n > 65535 {
                return Err(KillportError::InvalidPort(n));
            }
            Ok(PortSpec::Single(n as u16))
        }
    }

    pub fn expand(&self) -> Vec<u16> {
        match self {
            PortSpec::Single(p) => vec![*p],
            PortSpec::Range(start, end) => (*start..=*end).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_port() {
        let spec = PortSpec::parse("3000").unwrap();
        assert!(matches!(spec, PortSpec::Single(3000)));
        assert_eq!(spec.expand(), vec![3000]);
    }

    #[test]
    fn parse_port_range() {
        let spec = PortSpec::parse("3000-3005").unwrap();
        assert!(matches!(spec, PortSpec::Range(3000, 3005)));
        assert_eq!(spec.expand(), vec![3000, 3001, 3002, 3003, 3004, 3005]);
    }

    #[test]
    fn parse_invalid_port_zero() {
        assert!(PortSpec::parse("0").is_err());
    }

    #[test]
    fn parse_invalid_port_too_large() {
        assert!(PortSpec::parse("99999").is_err());
    }

    #[test]
    fn parse_invalid_port_text() {
        assert!(PortSpec::parse("abc").is_err());
    }

    #[test]
    fn parse_reversed_range() {
        assert!(PortSpec::parse("3010-3000").is_err());
    }

    #[test]
    fn parse_range_starting_at_zero() {
        assert!(PortSpec::parse("0-100").is_err());
    }
}
