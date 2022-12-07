#![allow(unknown_lints)]
#![allow(clippy::uninlined_format_args)] // Added in 1.65.0
//! [Ref](https://github.com/Alamofire/Alamofire/blob/5.6.4/Source/HTTPHeaders.swift#L370)

use std::{
    error, fmt,
    io::{self, BufRead as _},
    str::{self, FromStr},
};

use semver::Version;

const UNKNOWN: &str = "Unknown";

const OS_VERSION_DEFAULT: &(u64, u64, u64) = &(0, 0, 0);
const ALAMOFIRE_VERSION_DEFAULT: &(u64, u64, u64) = &(5, 6, 4);

//
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultUserAgent {
    pub executable: Option<String>,
    pub app_version: Option<Version>,
    pub bundle: Option<String>,
    pub app_build: Option<DefaultUserAgentAppBuild>,
    pub os_name: Option<DefaultUserAgentOsName>,
    pub os_version: Version,
    pub alamofire_version: Version,
}
impl Default for DefaultUserAgent {
    fn default() -> Self {
        Self {
            executable: None,
            app_version: None,
            bundle: None,
            app_build: None,
            os_name: None,
            os_version: Version {
                major: OS_VERSION_DEFAULT.0,
                minor: OS_VERSION_DEFAULT.1,
                patch: OS_VERSION_DEFAULT.2,
                pre: Default::default(),
                build: Default::default(),
            },
            alamofire_version: Version {
                major: ALAMOFIRE_VERSION_DEFAULT.0,
                minor: ALAMOFIRE_VERSION_DEFAULT.1,
                patch: ALAMOFIRE_VERSION_DEFAULT.2,
                pre: Default::default(),
                build: Default::default(),
            },
        }
    }
}

impl DefaultUserAgent {
    pub fn parse(bytes: impl AsRef<[u8]>) -> Result<Self, DefaultUserAgentParseError> {
        let bytes = bytes.as_ref();

        let mut cursor = io::Cursor::new(bytes);
        let mut buf = vec![];

        //
        //
        //
        let n = cursor
            .read_until(b'/', &mut buf)
            .map_err(|err| DefaultUserAgentParseError::ExecutableReadFailed(err.to_string()))?;
        if !buf.ends_with(&[b'/']) {
            return Err(DefaultUserAgentParseError::ExecutableReadFailed(
                "Invalid".to_owned(),
            ));
        }
        let mut executable =
            Some(String::from_utf8(buf[..n - 1].to_vec()).map_err(|err| {
                DefaultUserAgentParseError::ExecutableParseFailed(err.to_string())
            })?);
        if executable.as_deref() == Some(UNKNOWN) {
            executable = None;
        }
        buf.clear();

        //
        //
        //
        let n = cursor
            .read_until(b' ', &mut buf)
            .map_err(|err| DefaultUserAgentParseError::AppVersionReadFailed(err.to_string()))?;
        if !buf.ends_with(&[b' ']) {
            return Err(DefaultUserAgentParseError::AppVersionReadFailed(
                "Invalid".to_owned(),
            ));
        }
        let mut app_version =
            Some(String::from_utf8(buf[..n - 1].to_vec()).map_err(|err| {
                DefaultUserAgentParseError::AppVersionParseFailed(err.to_string())
            })?);
        if app_version.as_deref() == Some(UNKNOWN) {
            app_version = None;
        }

        let app_version = if let Some(app_version) = app_version {
            Some(Version::parse(&app_version).map_err(|err| {
                DefaultUserAgentParseError::AppVersionParseFailed(err.to_string())
            })?)
        } else {
            None
        };

        buf.clear();

        //
        //
        //
        if !cursor.get_ref()[cursor.position() as usize..].starts_with(&[b'(']) {
            return Err(DefaultUserAgentParseError::Other("Mismatch AppVersion end"));
        }
        cursor.set_position(cursor.position() + 1);

        //
        //
        //
        let n = cursor
            .read_until(b';', &mut buf)
            .map_err(|err| DefaultUserAgentParseError::BundleReadFailed(err.to_string()))?;
        if !buf.ends_with(&[b';']) {
            return Err(DefaultUserAgentParseError::BundleReadFailed(
                "Invalid".to_owned(),
            ));
        }
        let mut bundle = Some(
            String::from_utf8(buf[..n - 1].to_vec())
                .map_err(|err| DefaultUserAgentParseError::BundleParseFailed(err.to_string()))?,
        );
        if bundle.as_deref() == Some(UNKNOWN) {
            bundle = None;
        }

        buf.clear();

        //
        //
        //
        if !cursor.get_ref()[cursor.position() as usize..].starts_with(b" build:") {
            return Err(DefaultUserAgentParseError::Other("Mismatch Bundle end"));
        }
        cursor.set_position(cursor.position() + 7);

        //
        //
        //
        let n = cursor
            .read_until(b';', &mut buf)
            .map_err(|err| DefaultUserAgentParseError::AppBuildReadFailed(err.to_string()))?;
        if !buf.ends_with(&[b';']) {
            return Err(DefaultUserAgentParseError::AppBuildReadFailed(
                "Invalid".to_owned(),
            ));
        }
        let app_build = String::from_utf8(buf[..n - 1].to_vec())
            .map_err(|err| DefaultUserAgentParseError::AppBuildParseFailed(err.to_string()))?;

        let app_build = DefaultUserAgentAppBuild::parse(app_build)
            .map_err(DefaultUserAgentParseError::AppBuildParseFailed)?;

        buf.clear();

        //
        //
        //
        if !cursor.get_ref()[cursor.position() as usize..].starts_with(&[b' ']) {
            return Err(DefaultUserAgentParseError::Other("Mismatch AppBuild end"));
        }
        cursor.set_position(cursor.position() + 1);

        //
        //
        //
        let n = cursor
            .read_until(b' ', &mut buf)
            .map_err(|err| DefaultUserAgentParseError::OsNameReadFailed(err.to_string()))?;
        if !buf.ends_with(&[b' ']) {
            return Err(DefaultUserAgentParseError::OsNameReadFailed(
                "Invalid".to_owned(),
            ));
        }
        let os_name = String::from_utf8(buf[..n - 1].to_vec())
            .map_err(|err| DefaultUserAgentParseError::OsNameParseFailed(err.to_string()))?;

        let os_name = DefaultUserAgentOsName::parse(os_name)
            .map_err(|err| DefaultUserAgentParseError::OsNameParseFailed(err.to_string()))?;

        buf.clear();

        //
        //
        //
        let n = cursor
            .read_until(b')', &mut buf)
            .map_err(|err| DefaultUserAgentParseError::OsVersionReadFailed(err.to_string()))?;
        if !buf.ends_with(&[b')']) {
            return Err(DefaultUserAgentParseError::OsVersionReadFailed(
                "Invalid".to_owned(),
            ));
        }
        let os_version = String::from_utf8(buf[..n - 1].to_vec())
            .map_err(|err| DefaultUserAgentParseError::OsVersionParseFailed(err.to_string()))?;

        let os_version = Version::parse(&os_version)
            .map_err(|err| DefaultUserAgentParseError::OsVersionParseFailed(err.to_string()))?;

        buf.clear();

        //
        //
        //
        if !cursor.get_ref()[cursor.position() as usize..].starts_with(b" Alamofire/") {
            return Err(DefaultUserAgentParseError::Other("Mismatch OsVersion end"));
        }
        cursor.set_position(cursor.position() + 11);

        //
        //
        //
        let n = cursor.read_until(b'\n', &mut buf).map_err(|err| {
            DefaultUserAgentParseError::AlamofireVersionReadFailed(err.to_string())
        })?;
        if !cursor.get_ref()[cursor.position() as usize..].is_empty() {
            return Err(DefaultUserAgentParseError::Other(
                "Mismatch AlamofireVersion end",
            ));
        }

        let alamofire_version =
            String::from_utf8(buf[..if buf.ends_with(&[b'\n']) { n - 1 } else { n }].to_vec())
                .map_err(|err| {
                    DefaultUserAgentParseError::AlamofireVersionParseFailed(err.to_string())
                })?;

        let alamofire_version = Version::parse(&alamofire_version).map_err(|err| {
            DefaultUserAgentParseError::AlamofireVersionParseFailed(err.to_string())
        })?;

        //
        //
        //
        Ok(DefaultUserAgent {
            executable,
            app_version,
            bundle,
            app_build,
            os_name,
            os_version,
            alamofire_version,
        })
    }
}

//
#[derive(Debug, Clone)]
pub enum DefaultUserAgentParseError {
    //
    ExecutableReadFailed(String),
    ExecutableParseFailed(String),
    //
    AppVersionReadFailed(String),
    AppVersionParseFailed(String),
    //
    BundleReadFailed(String),
    BundleParseFailed(String),
    //
    AppBuildReadFailed(String),
    AppBuildParseFailed(String),
    //
    OsNameReadFailed(String),
    OsNameParseFailed(String),
    //
    OsVersionReadFailed(String),
    OsVersionParseFailed(String),
    //
    AlamofireVersionReadFailed(String),
    AlamofireVersionParseFailed(String),
    //
    Other(&'static str),
}
impl fmt::Display for DefaultUserAgentParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl error::Error for DefaultUserAgentParseError {}

//
impl FromStr for DefaultUserAgent {
    type Err = DefaultUserAgentParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s.to_string().as_str())
    }
}

impl fmt::Display for DefaultUserAgent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{} ({}; build:{}; {} {}) Alamofire/{}",
            self.executable.as_ref().unwrap_or(&UNKNOWN.to_owned()),
            self.app_version
                .as_ref()
                .map(|x| x.to_string())
                .unwrap_or_else(|| UNKNOWN.to_owned()),
            self.bundle.as_ref().unwrap_or(&UNKNOWN.to_owned()),
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    fn test_default() {
        assert_eq!(
            DefaultUserAgent::default(),
            DefaultUserAgent {
                executable: None,
                app_version: None,
                bundle: None,
                app_build: None,
                os_name: None,
                os_version: "0.0.0".parse().unwrap(),
                alamofire_version: "5.6.4".parse().unwrap()
            }
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            DefaultUserAgent::parse(
                "iOS Example/1.0.0 (org.alamofire.iOS-Example; build:1; iOS 13.0.0) Alamofire/5.0.0"
            )
            .unwrap(),
            DefaultUserAgent {
                executable: Some("iOS Example".to_owned()),
                app_version: Some("1.0.0".parse().unwrap()),
                bundle: Some("org.alamofire.iOS-Example".to_owned()),
                app_build: Some(DefaultUserAgentAppBuild("1.0.0".parse().unwrap())),
                os_name: Some(DefaultUserAgentOsName::iOS),
                os_version: "13.0.0".parse().unwrap(),
                alamofire_version: "5.0.0".parse().unwrap()
            }
        );

        assert_eq!(
            DefaultUserAgent::parse(
                "Unknown/Unknown (Unknown; build:Unknown; Unknown 13.0.0) Alamofire/5.0.0"
            )
            .unwrap(),
            DefaultUserAgent {
                executable: None,
                app_version: None,
                bundle: None,
                app_build: None,
                os_name: None,
                os_version: "13.0.0".parse().unwrap(),
                alamofire_version: "5.0.0".parse().unwrap()
            }
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(
            DefaultUserAgent {
                executable: Some("iOS Example".to_owned()),
                app_version: Some("1.0.0".parse().unwrap()),
                bundle: Some("org.alamofire.iOS-Example".to_owned()),
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
