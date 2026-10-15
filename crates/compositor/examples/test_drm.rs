use smithay::backend::{
    session::{libseat::LibSeatSession, Session},
    udev::{all_gpus, primary_gpu},
};

fn main() {
    println!("=== Probando detección DRM y LibSeat ===");
    match LibSeatSession::new() {
        Ok((session, _notifier)) => {
            let seat = session.seat();
            println!("✅ LibSeatSession activa en asiento: '{}'", seat);
            match primary_gpu(&seat) {
                Ok(Some(gpu)) => println!("✅ Primary GPU encontrada: {:?}", gpu),
                Ok(None) => println!("⚠️ No primary GPU encontrada para seat '{}'", seat),
                Err(e) => println!("❌ Error buscando primary GPU: {:?}", e),
            }
            match all_gpus(&seat) {
                Ok(gpus) => println!("✅ Todas las GPUs disponibles: {:?}", gpus),
                Err(e) => println!("❌ Error listando GPUs: {:?}", e),
            }
        }
        Err(e) => {
            println!("❌ LibSeatSession::new() falló: {:?}", e);
            println!("ℹ️ (Es normal si se ejecuta dentro de una sesión gráfica anidada que ya tiene tomado el VT/seat0)");
        }
    }
}
