mod core;

#[cfg(feature = "subnet-catalog-host")]
use super::*;
#[cfg(feature = "subnet-catalog-host")]
use super::{ClassificationSource, GeographicScope, SubnetSpecialization};

#[cfg(feature = "subnet-catalog-host")]
use crate::tests::temp_dir;
#[cfg(feature = "subnet-catalog-host")]
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(feature = "subnet-catalog-host")]
mod authority;
#[cfg(feature = "subnet-catalog-host")]
mod cache;
#[cfg(feature = "subnet-catalog-host")]
mod fixtures;
#[cfg(feature = "subnet-catalog-host")]
mod info;
#[cfg(feature = "subnet-catalog-host")]
mod list;
#[cfg(feature = "subnet-catalog-host")]
mod refresh;
#[cfg(feature = "subnet-catalog-host")]
mod stale_time;
