//! [Ref](https://github.com/Alamofire/Alamofire/blob/5.4.4/Source/HTTPHeaders.swift#L372)

use std::{error, fmt, str::FromStr};

use semver::Version;

const UNKNOWN: &str = "Unknown";

//
#[derive(Clone)]
pub struct DefaultUserAgent<'a> {
    pub executable: Option<&'a str>,
    pub app_version: Option<Version>,
    pub bundle: Option<&'a str>,
    pub app_build: Option<DefaultUserAgentAppBuild>,
    pub os_name: Option<DefaultUserAgentOsName>,
    pub os_version: Version,
    pub alamofire_version: Version,
}

impl<'a> DefaultUserAgent<'a> {
    pub fn parse(_s: impl AsRef<str>) -> Result<Self, DefaultUserAgentParseError> {
        todo!()
    }
}

//
#[derive(Debug, Clone)]
pub enum DefaultUserAgentParseError {}
impl fmt::Display for DefaultUserAgentParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl error::Error for DefaultUserAgentParseError {}

//
impl<'a> FromStr for DefaultUserAgent<'a> {
    type Err = DefaultUserAgentParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for DefaultUserAgent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{} ({}; build:{}; {} {}) Alamofire/{}",
            self.executable.unwrap_or(UNKNOWN),
            self.app_version
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or_else(|| UNKNOWN.to_owned()),
            self.bundle.unwrap_or(UNKNOWN),
            self.app_build
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or_else(|| UNKNOWN.to_owned()),
            self.os_name
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or_else(|| UNKNOWN.to_owned()),
            self.os_version,
            self.alamofire_version,
        )
    }
}

//
//
//
#[derive(Debug, Clone)]
pub struct DefaultUserAgentAppBuild(pub Version);
impl DefaultUserAgentAppBuild {
    pub fn parse(s: impl AsRef<str>) -> Result<Option<Self>, String> {
        OptionDefaultUserAgentAppBuild::from_str(s.as_ref()).map(|x| x.0)
    }
}

struct OptionDefaultUserAgentAppBuild(Option<DefaultUserAgentAppBuild>);
impl FromStr for OptionDefaultUserAgentAppBuild {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            UNKNOWN => Ok(Self(None)),
            _ => {
                let version = match s.split('.').count() {
                    1 => Version::parse(format!("{}.0.0", s).as_str())
                        .map_err(|err| err.to_string())?,
                    2 => Version::parse(format!("{}.0", s).as_str())
                        .map_err(|err| err.to_string())?,
                    3 => Version::parse(s).map_err(|err| err.to_string())?,
                    _ => {
                        return Err("Invalid".to_owned());
                    }
                };

                Ok(Self(Some(DefaultUserAgentAppBuild(version))))
            }
        }
    }
}

impl fmt::Display for DefaultUserAgentAppBuild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.patch == 0 && self.0.minor == 0 {
            write!(f, "{}", self.0.major)
        } else if self.0.minor == 0 {
            write!(f, "{}.{}", self.0.major, self.0.minor)
        } else {
            write!(f, "{}.{}.{}", self.0.major, self.0.minor, self.0.patch)
        }
    }
}

//
//
//
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DefaultUserAgentOsName {
    macOSCatalyst,
    iOS,
    watchOS,
    tvOS,
    macOS,
    Linux,
    Windows,
}
impl DefaultUserAgentOsName {
    pub fn parse(s: impl AsRef<str>) -> Result<Option<Self>, &'static str> {
        OptionDefaultUserAgentOsName::from_str(s.as_ref()).map(|x| x.0)
    }
}

struct OptionDefaultUserAgentOsName(Option<DefaultUserAgentOsName>);
impl FromStr for OptionDefaultUserAgentOsName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "macOS(Catalyst)" => Ok(Self(Some(DefaultUserAgentOsName::macOSCatalyst))),
            "iOS" => Ok(Self(Some(DefaultUserAgentOsName::iOS))),
            "watchOS" => Ok(Self(Some(DefaultUserAgentOsName::watchOS))),
            "tvOS" => Ok(Self(Some(DefaultUserAgentOsName::tvOS))),
            "macOS" => Ok(Self(Some(DefaultUserAgentOsName::macOS))),
            "Linux" => Ok(Self(Some(DefaultUserAgentOsName::Linux))),
            "Windows" => Ok(Self(Some(DefaultUserAgentOsName::Windows))),
            UNKNOWN => Ok(Self(None)),
            _ => Err("Mismatch"),
        }
    }
}

impl fmt::Display for DefaultUserAgentOsName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::macOSCatalyst => write!(f, "macOS(Catalyst)"),
            Self::iOS => write!(f, "iOS"),
            Self::watchOS => write!(f, "watchOS"),
            Self::tvOS => write!(f, "tvOS"),
            Self::macOS => write!(f, "macOS"),
            Self::Linux => write!(f, "Linux"),
            Self::Windows => write!(f, "Windows"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        assert_eq!(
            DefaultUserAgent {
                executable: Some("iOS Example"),
                app_version: Some("1.0.0".parse().unwrap()),
                bundle: Some("org.alamofire.iOS-Example"),
                app_build: Some(DefaultUserAgentAppBuild("1.0.0".parse().unwrap())),
                os_name: Some(DefaultUserAgentOsName::iOS),
                os_version: "13.0.0".parse().unwrap(),
                alamofire_version: "5.0.0".parse().unwrap()
            }
            .to_string(),
            "iOS Example/1.0.0 (org.alamofire.iOS-Example; build:1; iOS 13.0.0) Alamofire/5.0.0"
        );

        assert_eq!(
            DefaultUserAgent {
                executable: None,
                app_version: None,
                bundle: None,
                app_build: None,
                os_name: None,
                os_version: "13.0.0".parse().unwrap(),
                alamofire_version: "5.0.0".parse().unwrap()
            }
            .to_string(),
            "Unknown/Unknown (Unknown; build:Unknown; Unknown 13.0.0) Alamofire/5.0.0"
        );
    }
}
