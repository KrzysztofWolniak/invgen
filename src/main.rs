mod utils;
mod models;
mod schema;
extern crate actix_web;
extern crate serde;
extern crate serde_json;
extern crate actix_cors;
use actix_cors::Cors;
use actix_web::{post, web, App, HttpResponse, HttpServer, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Cursor;
use crate::utils::pdf_utils::*
;

// Zaimportuj swoje moduły, takie jak pdf_utils
// use pdf_utils::PdfDocumentWrapper;

use std::time::Instant;
use printpdf::Mm;

use crate::models::user::{NewUser, User};
use crate::schema::users;

#[derive(Serialize, Deserialize)]
struct Payload {
    texts_arr: Vec<TextInfo>,
    borders_arr: Vec<Rectangle>,
    lines_arr: Vec<LineDim>
}

#[post("/generate_pdf")]
async fn generate_pdf(payload: web::Json<Payload>) -> Result<HttpResponse> {
    // Przetwarzanie danych i generowanie PDF
    // Na przykład:
    let start = Instant::now();
     let pdf_doc = PdfDocumentWrapper::new("Faktura Vat","Montserrat.ttf");
    let items = vec![
        WrappedTextParams {
            text: "To jest krótki tekst.".to_string(),
            max_width: Mm(50.0), // Szerokość, która na pewno nie spowoduje zawinięcia
            font_size: 12.0,
            line_height: Mm(5.0),
            alignment: TextAlignment::Left,
        },
        WrappedTextParams {
            text: "To jest dłuższy tekst, który zostanie zawinięty, ponieważ jego szerokość przekracza maksymalną szerokość kolumny.".to_string(),
            max_width: Mm(80.0), // Szerokość, która spowoduje zawinięcie tekstu
            font_size: 12.0,
            line_height: Mm(5.0),
            alignment: TextAlignment::Left,
        },
        // Możesz dodać więcej elementów, jeśli chcesz przetestować różne przypadki
    ];

// Początkowa pozycja X i Y dla rysowania
    let start_x = Mm(10.0);
    let start_y = Mm(100.0);
    let items2 = vec![items.clone(),items.clone()];
// Rysowanie elementów
    pdf_doc.draw_items(items2,Mm(4.0) ,start_x, start_y);
    pdf_doc.draw_texts(&payload.texts_arr);
    pdf_doc.draw_rectangles(&payload.borders_arr);
    pdf_doc.draw_lines(&payload.lines_arr);
    let mut buffer = Vec::new();
    pdf_doc.save(&mut buffer).map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Failed to save PDF: {}", e))
    })?;
    let duration = start.elapsed();
	println!("{:?}",duration);
    Ok(HttpResponse::Ok()
        .header("Content-Disposition", "attachment; filename=generated.pdf")
        .content_type("application/pdf")
        .body(buffer))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
	let cors = Cors::permissive();
        App::new()
	    .wrap(cors)
            .service(generate_pdf)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
