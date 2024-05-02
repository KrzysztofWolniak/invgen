
// Plik: pdf_utils.rs

use printpdf::*;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io;
use rusttype::{Font, Scale};
use std::io::{BufWriter, Cursor, Read, Write};
use std::process::{Command, Stdio};
use std::time::Instant;
use crate::models::document_schema::ItemsSchema;

#[derive(Serialize, Deserialize, Debug)]
pub struct TextPositions {
    x: f32,
    y: f32,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TextInfo {
    position:TextPositions,
    text: String,
    #[serde(rename = "fontSize")]
    font_size: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextArray {
    arr: Vec<TextInfo>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Dimensions {
    height: f32,
    width: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Position {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Rectangle {
    #[serde(rename = "borderWidth")]
    border_width: f32,
    dimensions: Dimensions,
    position: Position,
    #[serde(rename = "borderRadius")]
    border_radius: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RectangleArray {
    arr: Vec<Rectangle>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct LineDim {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    thickness: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LineArray {
    arr: Vec<LineDim>,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone)]

pub struct WrappedTextParams {
    pub(crate) text: String,
    pub(crate) max_width: Mm,
    pub(crate) font_size: f32,
    pub(crate) line_height: Mm,
    pub(crate) alignment: TextAlignment,
}


pub struct PdfDocumentWrapper {
    doc: PdfDocumentReference,
    page1: PdfPageIndex,
    layer1: PdfLayerIndex,
    font: IndirectFontRef,
    text_font:rusttype::Font<'static>,
}

impl PdfDocumentWrapper {
    pub fn new(title: &str, font_path: &str) -> Self {
        let (doc, page1, layer1) = PdfDocument::new(title, Mm(210.0), Mm(297.0), "Layer 1");
        let font = doc.add_external_font(File::open(font_path).unwrap()).unwrap();
        let text_font_data = std::fs::read(font_path).expect("Error reading font file");
        let text_font = rusttype::Font::try_from_vec(text_font_data)
            .unwrap();
        PdfDocumentWrapper { doc, page1, layer1, font ,text_font}
    }
    //Deprecated
    // pub fn load_font_from_file(path: &str) -> Result<Font<'static>, Box<dyn std::error::Error>> {
    //     let mut font_data = Vec::new();
    //     File::open(path)?.read_to_end(&mut font_data)?;
    //     Ok(Font::try_from_vec(font_data).ok_or("Error converting to Font type")?)
    // }

    // Funkcja do obliczania szerokości tekstu
    fn calculate_text_width(&self, text: &str, font_size: f32) -> f32 {
        let scale = rusttype::Scale::uniform(font_size);
        self.text_font.layout(text, scale, rusttype::point(0.0, 0.0))
            .map(|glyph| glyph.unpositioned().h_metrics().advance_width)
            .sum()
    }
    pub fn build_items_list_from_schema(&self, schema:ItemsSchema )->u32 {
        // Todo: Implementacja tej funkcji
        return 1
    }
    pub fn draw_wrapped_text(&self, params: &WrappedTextParams, start_x: Mm, start_y: Mm) -> Vec<String> {
        // Assume that max_width is already provided in mm
        let max_width_pt = params.max_width.into_pt().0;
        let font_size_pt = params.font_size;
        let mut current_height_mm = start_y;

        let current_layer = self.doc.get_page(self.page1).get_layer(self.layer1);
        let mut lines = vec![];
        let mut line = String::new();
        let space_width_mm = self.calculate_text_width(" ", font_size_pt);

        for word in params.text.split_whitespace() {
            let word_width_mm = self.calculate_text_width(word, font_size_pt) + space_width_mm;

            if (self.calculate_text_width(&line, font_size_pt) + word_width_mm) < max_width_pt {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            } else {
                // Adjust text based on alignment
                let line_width_pt = self.calculate_text_width(&line, font_size_pt);
                let x = match params.alignment {
                    TextAlignment::Left => start_x,
                    TextAlignment::Center => Mm(start_x.0 + (max_width_pt - line_width_pt) / 2.0),
                    TextAlignment::Right => Mm(start_x.0 + (max_width_pt - line_width_pt)),
                };

                // Draw the line and move to the next
                current_layer.use_text(&line, font_size_pt, x, current_height_mm, &self.font);
                current_height_mm = Mm(current_height_mm.0 - params.line_height.0); // Line height assumed to be in mm
                lines.push(line);
                line = String::from(word);
            }
        }

        if !line.is_empty() {
            // Draw the last line
            let line_width_mm = self.calculate_text_width(&line, font_size_pt);
            let x = match params.alignment {
                TextAlignment::Left => start_x,
                TextAlignment::Center => Mm(start_x.0 + (max_width_pt - line_width_mm) / 2.0),
                TextAlignment::Right => Mm(start_x.0 + (max_width_pt - line_width_mm)),
            };
            current_layer.use_text(&line, font_size_pt, x, current_height_mm, &self.font);
            lines.push(line);
        }

        lines
    }


    // This function assumes you have a way to get the item's layout properties.
    pub fn draw_items(&self, items: Vec<Vec<WrappedTextParams>>, space_between_items: Mm, base_start_x: Mm, mut start_y: Mm) {
        for item in items {
            let mut current_x = base_start_x;
            let mut max_item_height:f32 = 0.0;

            // Przechodzimy przez wszystkie params dla pojedynczego item
            for params in &item {
                let lines = self.draw_wrapped_text(params, current_x, start_y);
                let lines_height = lines.len() as f32 * params.line_height.0;
                max_item_height = max_item_height.max(lines_height);

                // Przesuwamy current_x w prawo o max_width obecnego params + space_between_items
                current_x = Mm(current_x.0 + params.max_width.0 + space_between_items.0);
            }

            // Po obsłużeniu wszystkich params dla item, przesuwamy start_y w dół o max_item_height + space_between_items
            // Uwzględniamy odwrócenie osi Y w bibliotece printpdf
            start_y = Mm(start_y.0 - max_item_height - space_between_items.0);
        }

    }


    pub fn draw_texts(&self, texts: &Vec<TextInfo>) {
        let current_layer = self.doc.get_page(self.page1).get_layer(self.layer1);
        for text in texts {
            current_layer.use_text(&text.text, Mm(text.font_size).into_pt().0, Mm(text.position.x), Mm(text.position.y), &self.font);
        }
    }
    pub fn draw_rectangles(&self, rectangles: &Vec<Rectangle>) {
        let current_page = &self.doc.get_page(self.page1);
        let _layer = &current_page.get_layer(self.layer1);

        for rect in rectangles {
            self.draw_rounded_rectangle(
                Mm(rect.position.x),
                Mm(rect.position.y),
                Mm(rect.dimensions.width),
                Mm(rect.dimensions.height),
                Mm(rect.border_radius),
                rect.border_width,
            );
        }
    }
    pub fn draw_rounded_rectangle(
        &self,
        x: Mm,
        y: Mm,
        width: Mm,
        height: Mm,
        radius: Mm,
        thickness: f32,
    ) {
        let current_page = &self.doc.get_page(self.page1);
        let layer = &current_page.get_layer(self.layer1);
        layer.set_outline_thickness(thickness);
        let bezier_handle_offset = radius * 0.55228;
        let points = vec![
            // Start from the top-left corner after the curve
            (Point::new(x + radius, y + height), true),

            // Top-left corner curve
            (Point::new(x + radius - bezier_handle_offset, y + height), true),
            (Point::new(x, y + height - bezier_handle_offset), false),
            (Point::new(x, y + height - radius), false),

            // Move down to bottom-left corner before the curve
            (Point::new(x, y + radius), true),

            // Bottom-left corner curve
            (Point::new(x, y + bezier_handle_offset), true),
            (Point::new(x + radius - bezier_handle_offset, y), false),
            (Point::new(x + radius, y), false),

            // Move right to bottom-right corner before the curve
            (Point::new(x + width - radius, y), true),

            // Bottom-right corner curve
            (Point::new(x + width - bezier_handle_offset, y), true),
            (Point::new(x + width, y + bezier_handle_offset), false),
            (Point::new(x + width, y + radius), false),

            // Move up to top-right corner before the curve
            (Point::new(x + width, y + height - radius), true),

            // Top-right corner curve
            (Point::new(x + width, y + height - bezier_handle_offset), true),
            (Point::new(x + width - bezier_handle_offset, y + height), false),
            (Point::new(x + width - radius, y + height), false),
        ];

        let rounded_rect = Polygon {
            rings: vec![points],
            mode: PolygonMode::Stroke,
            winding_order: WindingOrder::NonZero,
        };

        layer.add_polygon(rounded_rect);
    }
    pub fn draw_lines(&self, lines: &Vec<LineDim>) {
        let current_layer = self.doc.get_page(self.page1).get_layer(self.layer1);

        for line in lines {
            current_layer.set_outline_thickness(line.thickness);

            let start_point = (Point::new(Mm(line.x), Mm(line.y)), true);
            let end_point = if line.width == 0.0 {
                // Linia pionowa
                (Point::new(Mm(line.x), Mm(line.y + line.height)), false)
            } else if line.height == 0.0 {
                // Linia pozioma
                (Point::new(Mm(line.x + line.width), Mm(line.y)), false)
            } else {
                // Nie rysuj linii, jeżeli nie jest ani pionowa ani pozioma
                continue;
            };

            let line = Line {
                points: vec![start_point, end_point],
                is_closed: false,
            };

            current_layer.add_line(line);
        }
    }
    pub fn save(self, buffer: &mut Vec<u8>) -> Result<(), Error> {
        let cursor = Cursor::new(buffer);
        let mut buf_writer = BufWriter::new(cursor);
        self.doc.save(&mut buf_writer)

    }
}
