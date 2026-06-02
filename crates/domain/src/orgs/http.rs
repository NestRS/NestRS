use poem::error::ResponseError;
use poem::http::StatusCode;

use crate::orgs::error::OrgError;

impl ResponseError for OrgError {
    fn status(&self) -> StatusCode {
        match self {
            OrgError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
