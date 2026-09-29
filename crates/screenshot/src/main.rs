use malus_de_common::logging::init_logging;
use tracing::info;

fn main() {
    init_logging("malus_de_screenshot");
    info!("Iniciando Malus DE Screenshot & OCR Tool (zwlr_screencopy_v1 / PipeWire / Tesseract)...");
}
