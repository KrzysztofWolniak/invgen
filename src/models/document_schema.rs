use diesel::sql_types::Array;
use printpdf::*;
use serde::{Serialize, Deserialize};
use crate::utils::pdf_utils::Position;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemsSchema {
    pub(crate) position: Position,
    pub(crate) header_text: String,
    pub(crate) max_width: f32,
    pub(crate) header_font_size: f32,
    pub(crate) child_font_size: f32,
    pub(crate) is_header_bold: bool,
    pub(crate) is_text_bold: bool,
    pub(crate) header_line_height: f32,
    pub(crate) child_line_height: f32,
    pub(crate) alignment: TextAlignment,
    pub(crate) childrens: Vec<Vec<ItemsChildSchema>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemsChildSchema {
    alignment: TextAlignment,
    text: String,
    is_text_bold: bool,
    line_height: f32,
    font_size: f32,

}

