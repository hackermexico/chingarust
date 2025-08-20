# chingarust
Herramienta para d0s escrita en rust para máximo rendimiento. 

ChingaRUST HTTP Stress Tool 🚀
Un simple pero potente script en Rust para realizar pruebas de estrés (HTTP Flood) en servidores web. Esta herramienta es capaz de generar un alto volumen de peticiones HTTP utilizando múltiples hilos para simular tráfico masivo.

Propósito principal: Educativo y para la evaluación de la resiliencia de infraestructuras de red en entornos controlados y autorizados.

⚠️ Descargo de Responsabilidad
El uso de esta herramienta es exclusivamente para fines educativos y de prueba de seguridad. No utilices este script contra ningún servidor o servicio sin el consentimiento explícito del propietario. Realizar un ataque de denegación de servicio contra sistemas sin autorización es ilegal y puede tener graves consecuencias. El autor no se hace responsable del mal uso de esta herramienta.

✨ Características
Multi-hilo: Lanza ataques desde múltiples hilos para maximizar la carga.

Peticiones Aleatorias: Genera peticiones HTTP con métodos (GET, POST, etc.), rutas y User-Agents aleatorios para simular un tráfico más realista y evitar cachés simples.

Estadísticas en Tiempo Real: Muestra en la consola el número de peticiones enviadas, la tasa de peticiones por segundo (RPS) y el ancho de banda utilizado en Mbps.

Configurable: Permite definir fácilmente el objetivo (host), puerto, duración y número de hilos.

Parada Segura: Utiliza un manejador para la señal Ctrl+C, permitiendo detener todos los hilos de forma limpia y segura.

Eficiente y Seguro: Escrito en Rust, aprovechando su seguridad en concurrencia y su alto rendimiento para generar carga de manera efectiva.

🛠️ Requisitos Previos
Para compilar y ejecutar este proyecto, necesitas tener instalado el toolchain de Rust (incluyendo rustc y cargo).

Clona o descarga el repositorio:
# Si usas git
git clone <URL_DEL_REPOSITORIO>
cd rust_http_flood

Compila el proyecto:
Cargo se encargará de descargar las dependencias (rand y ctrlc) y compilar el proyecto. Para obtener el mejor rendimiento, compila en modo release.

cargo build --release
El ejecutable se encontrará en ./target/release/rust_http_flood.
