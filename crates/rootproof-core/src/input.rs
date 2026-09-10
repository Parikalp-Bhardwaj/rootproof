use std::{fs, path::Path};

use crate::RootProofError;

pub fn read_incident_input(path: &Path) -> Result<String, RootProofError> {
    fs::read_to_string(path).map_err(|source| RootProofError::ReadIncidentInput {
        path: path.to_path_buf(),
        source,
    })
}
