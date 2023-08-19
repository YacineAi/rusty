#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::{Rocket, Data};
use rocket::data::ByteUnit;
use rocket::response::Stream;
use std::io::Cursor;
use reqwest::blocking::get;
use pdf2image::convert_from_bytes;

#[get("/convert?<pdf_url>&<page_number>")]
fn convert(pdf_url: String, page_number: u32) -> Result<Stream<Cursor<Vec<u8>>>, reqwest::Error> {
    // Fetch the PDF from the provided URL
    let pdf_response = get(&pdf_url)?;

    // Check if the response status is OK
    if !pdf_response.status().is_success() {
        return Err(reqwest::Error::from("Failed to fetch the PDF"));
    }

    // Convert the PDF response into bytes
    let pdf_bytes = pdf_response.bytes()?;

    // Convert the specific page of the PDF to an image
    let images = convert_from_bytes(pdf_bytes.as_ref(), page_number, 100);

    // Check if the page exists in the PDF
    if let Some(image) = images.first() {
        // Convert the image to a byte vector
        let mut image_bytes: Vec<u8> = Vec::new();
        image.write_to(&mut image_bytes, image::ImageOutputFormat::JPEG)
            .expect("Failed to write image to buffer");

        // Create a stream for the image bytes
        let image_cursor = Cursor::new(image_bytes);
        Ok(Stream::from(image_cursor))
    } else {
        Err(reqwest::Error::from("Page not found in the PDF"))
    }
}

fn rocket() -> Rocket {
    rocket::ignite()
        .mount("/", routes![convert])
        .keep_alive(0)
        .manage(rocket::Data::configure(|limit| limit.bytes(ByteUnit::default())));
}

fn main() {
    rocket().launch();
}
