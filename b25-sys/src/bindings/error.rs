#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![deny(dead_code)]

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum AribB25DecoderError {
    ARIB_STD_B25_ERROR_INVALID_PARAM = -1,
    ARIB_STD_B25_ERROR_NO_ENOUGH_MEMORY = -2,
    ARIB_STD_B25_ERROR_NON_TS_INPUT_STREAM = -3,
    ARIB_STD_B25_ERROR_NO_PAT_IN_HEAD_16M = -4,
    ARIB_STD_B25_ERROR_NO_PMT_IN_HEAD_32M = -5,
    ARIB_STD_B25_ERROR_NO_ECM_IN_HEAD_32M = -6,
    ARIB_STD_B25_ERROR_EMPTY_B_CAS_CARD = -7,
    ARIB_STD_B25_ERROR_INVALID_B_CAS_STATUS = -8,
    ARIB_STD_B25_ERROR_ECM_PROC_FAILURE = -9,
    ARIB_STD_B25_ERROR_DECRYPT_FAILURE = -10,
    ARIB_STD_B25_ERROR_PAT_PARSE_FAILURE = -11,
    ARIB_STD_B25_ERROR_PMT_PARSE_FAILURE = -12,
    ARIB_STD_B25_ERROR_ECM_PARSE_FAILURE = -13,
    ARIB_STD_B25_ERROR_CAT_PARSE_FAILURE = -14,
    ARIB_STD_B25_ERROR_EMM_PARSE_FAILURE = -15,
    ARIB_STD_B25_ERROR_EMM_PROC_FAILURE = -16,

    ARIB_STD_B25_WARN_UNPURCHASED_ECM = 1,
    ARIB_STD_B25_WARN_TS_SECTION_ID_MISSMATCH = 2,
    ARIB_STD_B25_WARN_BROKEN_TS_SECTION = 3,
    ARIB_STD_B25_WARN_PAT_NOT_COMPLETE = 4,
    ARIB_STD_B25_WARN_PMT_NOT_COMPLETE = 5,
    ARIB_STD_B25_WARN_ECM_NOT_COMPLETE = 6,
}

impl From<i32> for AribB25DecoderError
{
    fn from(e: i32) -> Self {
        if e== -1 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_INVALID_PARAM
        } else if e== -2 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_NO_ENOUGH_MEMORY
        } else if e== -3 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_NON_TS_INPUT_STREAM
        } else if e== -4 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_NO_PAT_IN_HEAD_16M
        } else if e== -5 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_NO_PMT_IN_HEAD_32M
        } else if e== -6 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_NO_ECM_IN_HEAD_32M
        } else if e== -7 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_EMPTY_B_CAS_CARD
        } else if e== -8 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_INVALID_B_CAS_STATUS
        } else if e== -9 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_ECM_PROC_FAILURE
        } else if e== -10 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_DECRYPT_FAILURE
        } else if e== -11 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_PAT_PARSE_FAILURE
        } else if e== -12 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_PMT_PARSE_FAILURE
        } else if e== -13 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_ECM_PARSE_FAILURE
        } else if e== -14 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_CAT_PARSE_FAILURE
        } else if e== -15 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_EMM_PARSE_FAILURE
        } else if e== -16 {
            AribB25DecoderError::ARIB_STD_B25_ERROR_EMM_PROC_FAILURE
        } else if e== 1 {
            AribB25DecoderError::ARIB_STD_B25_WARN_UNPURCHASED_ECM
        } else if e== 2 {
            AribB25DecoderError::ARIB_STD_B25_WARN_TS_SECTION_ID_MISSMATCH
        } else if e== 3 {
            AribB25DecoderError::ARIB_STD_B25_WARN_BROKEN_TS_SECTION
        } else if e== 4 {
            AribB25DecoderError::ARIB_STD_B25_WARN_PAT_NOT_COMPLETE
        } else if e== 5 {
            AribB25DecoderError::ARIB_STD_B25_WARN_PMT_NOT_COMPLETE
        } else if e== 6 {
            AribB25DecoderError::ARIB_STD_B25_WARN_ECM_NOT_COMPLETE
        } else {
            panic!("unknown error code: {}", e)
        }
    }
}


impl Display for AribB25DecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        type E = AribB25DecoderError;
        match *self {
            E::ARIB_STD_B25_ERROR_INVALID_PARAM => write!(f, "ARIB_STD_B25_ERROR_INVALID_PARAM"),
            E::ARIB_STD_B25_ERROR_NO_ENOUGH_MEMORY => write!(f, "ARIB_STD_B25_ERROR_NO_ENOUGH_MEMORY"),
            E::ARIB_STD_B25_ERROR_NON_TS_INPUT_STREAM => write!(f, "ARIB_STD_B25_ERROR_NON_TS_INPUT_STREAM"),
            E::ARIB_STD_B25_ERROR_NO_PAT_IN_HEAD_16M => write!(f, "ARIB_STD_B25_ERROR_NO_PAT_IN_HEAD_16M"),
            E::ARIB_STD_B25_ERROR_NO_PMT_IN_HEAD_32M => write!(f, "ARIB_STD_B25_ERROR_NO_PMT_IN_HEAD_32M"),
            E::ARIB_STD_B25_ERROR_NO_ECM_IN_HEAD_32M => write!(f, "ARIB_STD_B25_ERROR_NO_ECM_IN_HEAD_32M"),
            E::ARIB_STD_B25_ERROR_EMPTY_B_CAS_CARD => write!(f, "ARIB_STD_B25_ERROR_EMPTY_B_CAS_CARD"),
            E::ARIB_STD_B25_ERROR_INVALID_B_CAS_STATUS => write!(f, "ARIB_STD_B25_ERROR_INVALID_B_CAS_STATUS"),
            E::ARIB_STD_B25_ERROR_ECM_PROC_FAILURE => write!(f, "ARIB_STD_B25_ERROR_ECM_PROC_FAILURE"),
            E::ARIB_STD_B25_ERROR_DECRYPT_FAILURE => write!(f, "ARIB_STD_B25_ERROR_DECRYPT_FAILURE"),
            E::ARIB_STD_B25_ERROR_PAT_PARSE_FAILURE => write!(f, "ARIB_STD_B25_ERROR_PAT_PARSE_FAILURE"),
            E::ARIB_STD_B25_ERROR_PMT_PARSE_FAILURE => write!(f, "ARIB_STD_B25_ERROR_PMT_PARSE_FAILURE"),
            E::ARIB_STD_B25_ERROR_ECM_PARSE_FAILURE => write!(f, "ARIB_STD_B25_ERROR_ECM_PARSE_FAILURE"),
            E::ARIB_STD_B25_ERROR_CAT_PARSE_FAILURE => write!(f, "ARIB_STD_B25_ERROR_CAT_PARSE_FAILURE"),
            E::ARIB_STD_B25_ERROR_EMM_PARSE_FAILURE => write!(f, "ARIB_STD_B25_ERROR_EMM_PARSE_FAILURE"),
            E::ARIB_STD_B25_ERROR_EMM_PROC_FAILURE => write!(f, "ARIB_STD_B25_ERROR_EMM_PROC_FAILURE"),

            E::ARIB_STD_B25_WARN_UNPURCHASED_ECM => write!(f, "ARIB_STD_B25_WARN_UNPURCHASED_ECM"),
            E::ARIB_STD_B25_WARN_TS_SECTION_ID_MISSMATCH => write!(f, "ARIB_STD_B25_WARN_TS_SECTION_ID_MISSMATCH"),
            E::ARIB_STD_B25_WARN_BROKEN_TS_SECTION => write!(f, "ARIB_STD_B25_WARN_BROKEN_TS_SECTION"),
            E::ARIB_STD_B25_WARN_PAT_NOT_COMPLETE => write!(f, "ARIB_STD_B25_WARN_PAT_NOT_COMPLETE"),
            E::ARIB_STD_B25_WARN_PMT_NOT_COMPLETE => write!(f, "ARIB_STD_B25_WARN_PMT_NOT_COMPLETE"),
            E::ARIB_STD_B25_WARN_ECM_NOT_COMPLETE => write!(f, "ARIB_STD_B25_WARN_ECM_NOT_COMPLETE"),
        }
    }
}

impl std::error::Error for AribB25DecoderError {}
